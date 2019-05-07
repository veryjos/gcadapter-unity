use std::thread;
use std::sync::Arc;
use std::default::Default;

use crate::sync_cell::{SyncCell, SyncCellReader};

use libusb;

#[derive(Default, Clone, Copy)]
pub struct ControllerState {
    plugged_in: bool,

    buttons: u32,

    x: i8,
    y: i8,
    cx: i8,
    cy: i8,

    l: i8,
    r: i8
}

pub struct Adapter {
    controller_plug_callback: Arc<fn(i32, i32) -> ()>,
    controller_unplug_callback: Arc<fn(i32) -> ()>,
    controller_state_callback: Arc<fn(i32) -> ()>,

    thread_handle: thread::JoinHandle<()>,

    controllers: SyncCell<[ControllerState; 4]>,
    controllers_reader: SyncCellReader<[ControllerState; 4]>,

    device: libusb::Device<'static>,
    device_handle: libusb::DeviceHandle<'static>
}

impl Adapter {
    pub fn new(
        controller_plug_callback: Arc<fn(i32, i32) -> ()>,
        controller_unplug_callback: Arc<fn(i32) -> ()>,
        controller_state_callback: Arc<fn(i32) -> ()>,

        device: libusb::Device<'static>
    ) -> Adapter {
        let controllers = SyncCell::new();

        let thread_handle = {
            let controller_plug_callback = controller_plug_callback.clone();
            let writer = controllers.create_writer();
            let address = device.address();

            thread::spawn(move || {
                // Write some values to the controller
                // TODO: Actually poll the values out of libusb
                let data = [ControllerState::default(); 4];
                writer.write(data);

                controller_plug_callback(address.into(), 44);
            })
        };

        let device_handle = device.open().unwrap();
        let controllers_reader = controllers.create_reader();

        Adapter {
            controller_plug_callback,
            controller_unplug_callback,
            controller_state_callback,

            thread_handle,

            controllers,
            controllers_reader,

            device,
            device_handle
        }
    }

    pub fn address(&self) -> u8 {
        self.device.address()
    }

    pub fn update(&self) {
        // Get the last known controller state
        let controller_states = self.controllers_reader.read();

        (self.controller_plug_callback)(0, 0);
    }
}
