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

pub struct Adapter<'a> {
    thread_handle: thread::JoinHandle<()>,
    pub device: libusb::Device<'a>,

    pub controllers: SyncCell<[ControllerState; 4]>
}

impl<'a> Adapter<'a> {
    pub fn new(device: libusb::Device<'a>) -> Adapter {
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

        Adapter {
            thread_handle,
            device,

            controllers
        }
    }
}
