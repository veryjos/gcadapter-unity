use std::collections::{HashMap, HashSet};
use std::convert::TryInto;
use std::time::Duration;
use std::sync::{Arc, Mutex, mpsc};

use crate::controller::{ControllerState, ControllerId};
use crate::ffi::{ControllerPluggedCallback, ControllerUnpluggedCallback};
use crate::sync_cell::{SyncCell, SyncCellWriter};

pub const VENDOR_ID: u16 = 0x057E;
pub const PRODUCT_ID: u16 = 0x0337;

enum AdapterEvent {
    Added(rusb::Device<rusb::GlobalContext>),
    Removed(u8),
}

/// Represents the state of a Context.
struct ContextState {
    controllers: HashMap<ControllerId, SyncCell<ControllerState>>,
}

/// A `Context` holds on to global resources for users of `gcadapter-unity`. Every integration with
/// `gcadapter-unity` should create a `Context` before performing any other work.
///
/// You should only have one instance of this in your program.
pub struct Context {
    context_state: Arc<Mutex<ContextState>>,
    _write_thread_handle: std::thread::JoinHandle<()>,
    _hotplug_thread_handle: std::thread::JoinHandle<()>,
    _usb_context: rusb::Context,
}

impl Context {
    pub fn new(
        plugged_callback: ControllerPluggedCallback,
        unplugged_callback: ControllerUnpluggedCallback
    ) -> Context {
        let usb_context = rusb::Context::new()
            .expect("Failed to initialize libusb");

        let mut context_state = Arc::new(Mutex::new(ContextState {
            controllers: HashMap::new(),
        }));

        let (writer, reader) = mpsc::channel();

        // Spawn write thread, responsible for pushing updates for controller input.
        let write_thread_handle = {
            let context_state = context_state.clone();

            std::thread::spawn(move || {
                let mut adapters: HashMap<u8, rusb::DeviceHandle<rusb::GlobalContext>> = HashMap::new();
                let mut writers: HashMap<ControllerId, /*SyncCellWriter<ControllerState>*/ bool> = HashMap::new();
                let mut payload = [0u8; 37];

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

                                adapters.insert(device.address(), device_handle);
                            },

                            AdapterEvent::Removed(address) => {
                                adapters.remove(&address);
                            },
                        }
                    }

                    // Read controller input from each adapter.
                    for (base_idx, device_handle) in adapters.values_mut().enumerate() {
                        device_handle.read_interrupt(0x81, &mut payload, Duration::new(1, 0))
                            .expect("Failed to read from GameCube adapter.");

                        for i in 0..4 {
                            let controller_id = (base_idx + i) as ControllerId;
                            let ofs = 1 + i * 9;
                            let data: &[u8; 9] = payload[ofs..ofs + 9].try_into().unwrap();

                            if ControllerState::is_plugged(data) {
                                if !writers.contains_key(&controller_id) {
                                    writers.insert(controller_id, true);
                                    println!("Controller {} plugged in.", controller_id);
                                }
                            } else {
                                if writers.contains_key(&controller_id) {
                                    writers.remove(&controller_id);
                                    println!("Controller {} unplugged in.", controller_id);
                                }
                            }

                            /*
                            let writer_idx = base_idx * 4 + i;
                            writers[writer_idx].write(|state: &mut ControllerState| {
                                if old_state.plugged_in && !state.plugged_in {
                                    plugged_callback(i);
                                } else if !old_state.plugged_in && state.plugged_in {
                                    unplugged_callback(i);
                                }

                                state.read_slice(payload[ofs..ofs + 9]
                                    .try_into().unwrap());
                            });
                            */
                        }
                    }
                }

                // Request data from each device.
                /*

                // Read from device and write values out to SyncCellWriter.
                let mut payload = [0u8; 37];
                loop {
                    device_handle.read_interrupt(0x81, &mut payload, Duration::new(1, 0))
                        .expect("Failed to read from GameCube adapter");

                }
                */
            })
        };

        // Spawn hotplug thread (Windows doesn't support hotplug, so we have to do this).
        let hotplug_thread_handle = {
            let context_state = context_state.clone();

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
                        println!("closed {}", address);

                        writer.send(AdapterEvent::Removed(address)).unwrap();
                    }

                    // Add new adapters.
                    for device in adapters {
                        if opened.contains(&device.address()) {
                            continue
                        }

                        opened.insert(device.address());
                        println!("opened {}", device.address());

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
}


/*
        let controllers = [
            SyncCell::new(ControllerState::default()),
            SyncCell::new(ControllerState::default()),
            SyncCell::new(ControllerState::default()),
            SyncCell::new(ControllerState::default()),
        ];

        let writers: ArrayVec<[SyncCellWriter<ControllerState>; 4]> = controllers.iter()
            .map(|sync_cell| sync_cell.create_writer())
            .collect();

        let adapter = match writers.into_inner() {
            Ok(writers) => Adapter::new(
                device,
                writers,
                self.plugged_callback,
                self.unplugged_callback
            ),
            Err(_) => panic!("smallvec exceeded size of fixed-size array"),
        };

        unsafe {
            self.context_state.as_mut().adapters.push(adapter);
        }
        */