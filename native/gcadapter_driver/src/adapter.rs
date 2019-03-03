use std::thread;
use std::sync::Arc;
use std::default::Default;

use crate::sync_cell::SyncCell;

use libusb;

#[derive(Default, Clone, Copy)]
pub struct ControllerState {
    plugged_in: bool,

    buttons: u32,
    x: f32,
    y: f32,
    cx: f32,
    cy: f32
}

pub struct Adapter {
    thread_handle: thread::JoinHandle<()>,

    pub controllers: SyncCell<[ControllerState; 4]>,

    device: libusb::Device<'static>,
    device_handle: libusb::DeviceHandle<'static>
}

impl Adapter {
    pub fn new(device: libusb::Device<'static>) -> Adapter {
        let controllers = SyncCell::new();

        let thread_handle = {
            let writer = controllers.create_writer();

            thread::spawn(move || {
                // Write some values to the controller
                // TODO: Actually poll the values out of libusb
                let data = [ControllerState::default(); 4];
                writer.write(data);
            })
        };

        let device_handle = device.open().unwrap();

        Adapter {
            thread_handle,

            controllers,

            device,
            device_handle
        }
    }

    pub fn address(&self) -> u8 {
        self.device.address()
    }
}
