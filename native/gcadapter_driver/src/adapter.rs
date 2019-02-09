use std::thread;
use std::sync::Arc;

use crate::sync_cell::SyncCell;

use libusb;

#[derive(Clone, Copy)]
pub struct ControllerState {
    plugged_in: bool,

    buttons: u32,
    x: f32,
    y: f32,
    cx: f32,
    cy: f32
}

pub struct Adapter<'a> {
    pub device: libusb::Device<'a>,

    pub controllers: Vec<Arc<SyncCell<ControllerState>>>
}

impl<'a> Adapter<'a> {
    pub fn new(device: libusb::Device<'a>) -> Adapter {
        Adapter {
            device,

            controllers: vec!()
        }
    }
}
