#![feature(maybe_uninit)]

use std::thread;
use std::cell::RefCell;

use libusb;

mod sync_cell;

mod context;
mod adapter;

mod ffi;

/// Vendor ID for the GameCube adapter.
pub const vendor_id: u16 = 0x057E;

/// Device ID for the GameCube adapter.
pub const device_id: u16 = 0x0337;
