use std::collections::{HashMap, HashSet};
use std::convert::TryInto;
use std::time::Duration;
use std::sync::mpsc;

use crate::controller::{ControllerState, ControllerId};
use crate::ffi::{ControllerPluggedCallback, ControllerUnpluggedCallback};
use crate::sync_cell::SyncCell;

pub const VENDOR_ID: u16 = 0x057E;
pub const PRODUCT_ID: u16 = 0x0337;

enum AdapterEvent {
    Added(rusb::Device<rusb::GlobalContext>),
    Removed(u8),
}

struct ControllerSlots {
    allocated: Vec<bool>,
}

impl ControllerSlots {
    pub fn new() -> ControllerSlots {
        ControllerSlots {
            allocated: vec!(),
        }
    }

    /// Claims the lowest controller slot available.
    pub fn alloc(&mut self) -> usize {
        match self.allocated.iter().position(|slot| *slot == false) {
            Some(idx) => {
                self.allocated[idx] = true;

                idx
            },

            None => {
                self.allocated.push(true);

                self.allocated.len() - 1
            },
        }
    }

    /// Unclaims the controller slot specified.
    pub fn dealloc(&mut self, idx: usize) {
        assert!(self.allocated[idx] == true);

        self.allocated[idx] = false;
    }
}

struct Adapter {
    device_handle: rusb::DeviceHandle<rusb::GlobalContext>,
    controller_slot: usize,
}

impl Adapter {
    pub fn read(&self, payload: &mut [u8]) {
        match self.device_handle.read_interrupt(0x81, payload, Duration::new(1, 0)) {
            _ => (),
        }
    }
}

#[derive(Default)]
struct ContextState {
    controller_data: Vec<ControllerState>,
}

/// A `Context` holds on to global resources for users of `gcadapter-unity`. Every integration with
/// `gcadapter-unity` should create a `Context` before performing any other work.
///
/// You should only have one instance of this in your program.
pub struct Context {
    context_state: SyncCell<ContextState>,
    _write_thread_handle: std::thread::JoinHandle<()>,
    _hotplug_thread_handle: std::thread::JoinHandle<()>,
    _usb_context: rusb::Context,
}

impl Context {
    pub fn new(
        plugged_callback: ControllerPluggedCallback,
        unplugged_callback: ControllerUnpluggedCallback
    ) -> Context {
        let context_state = SyncCell::new();
        let usb_context = rusb::Context::new()
            .expect("Failed to initialize libusb");

        let (writer, reader) = mpsc::channel();

        // Spawn write thread, responsible for pushing updates for controller input.
        let write_thread_handle = {
            let context_state_writer = context_state.create_writer();

            std::thread::spawn(move || {
                let mut adapters: HashMap<u8, Adapter> = HashMap::new();
                let mut controllers: HashSet<ControllerId> = HashSet::new();
                let mut payload = [0u8; 37];
                let mut slots = ControllerSlots::new();

                loop {
                    // Handle adapter added or removed events.
                    while let Ok(event) = reader.try_recv() {
                        match event {
                            AdapterEvent::Added(device) => {
                                let mut device_handle = device.open()
                                    .expect("Failed to open GameCube adapter.");

                                device_handle.claim_interface(0x00)
                                    .expect("Failed to claim writable interface for GameCube adapter.");

                                device_handle.write_interrupt(0x2, &[0x13], Duration::new(1, 0))
                                    .expect("Failed to initialize GameCube adapter.");

                                adapters.insert(device.address(), Adapter {
                                    device_handle,
                                    controller_slot: slots.alloc(),
                                });
                            },

                            AdapterEvent::Removed(address) => {
                                {
                                    let adapter = adapters.get(&address).unwrap();
                                    slots.dealloc(adapter.controller_slot);

                                    let ofs = adapter.controller_slot * 4;
                                    for i in ofs..ofs + 4 {
                                        if controllers.contains(&i) {
                                            controllers.remove(&i);
                                            unplugged_callback(i);
                                        }
                                    }
                                }

                                adapters.remove(&address);
                            },
                        }
                    }
                
                    // Read controller input from each adapter.
                    let mut new_state = ContextState {
                        controller_data: vec!(),
                    };

                    for adapter in adapters.values_mut() {
                        adapter.read(&mut payload);

                        for i in 0..4 {
                            let controller_id = adapter.controller_slot + i as ControllerId;
                            let ofs = 1 + i * 9;
                            let data: &[u8; 9] = payload[ofs..ofs + 9].try_into().unwrap();

                            let mut controller_state = ControllerState::default();

                            if ControllerState::is_plugged(data) {
                                if !controllers.contains(&controller_id) {
                                    controllers.insert(controller_id);
                                    plugged_callback(controller_id);
                                }

                                controller_state.read_slice(data);
                            } else {
                                if controllers.contains(&controller_id) {
                                    controllers.remove(&controller_id);
                                    unplugged_callback(controller_id);
                                }
                            }

                            new_state.controller_data.push(controller_state);
                        }
                    }

                    context_state_writer.write(new_state);
                }
            })
        };

        // Spawn hotplug thread (Windows doesn't support hotplug, so we have to do this).
        let hotplug_thread_handle = {
            std::thread::spawn(move || {
                let mut opened = HashSet::new();

                loop {
                    // Poll device list every second.
                    std::thread::sleep(Duration::new(1, 0));

                    let devices = rusb::DeviceList::new()
                        .expect("Failed to enumerate USB devices");

                    let adapters: Vec<rusb::Device<rusb::GlobalContext>> = devices.iter()
                        .filter(|device| {
                            match device.device_descriptor() {
                                Ok(descriptor) =>
                                    descriptor.product_id() == PRODUCT_ID && descriptor.vendor_id() == VENDOR_ID,

                                Err(_) => false,
                            }
                        }).collect();

                    // Remove old adapters.
                    let old_adapters: Vec<u8> = opened.iter()
                        .filter(|address| adapters.iter()
                            .find(|device| device.address() == **address)
                            .is_none())
                        .cloned().into_iter().collect();

                    for address in old_adapters {
                        opened.remove(&address);
                        writer.send(AdapterEvent::Removed(address)).unwrap();
                    }

                    // Add new adapters.
                    for device in adapters {
                        if opened.contains(&device.address()) {
                            continue
                        }

                        opened.insert(device.address());
                        writer.send(AdapterEvent::Added(device)).unwrap();
                    }
                }
            })
        };

        let context = Context {
            context_state,
            _write_thread_handle: write_thread_handle,
            _hotplug_thread_handle: hotplug_thread_handle,
            _usb_context: usb_context,
        };

        context
    }

    pub fn get_latest_controller_state(&self, controller_id: ControllerId) -> ControllerState {
        let context_state = self.context_state.read();

        context_state.controller_data[controller_id]
    }
}