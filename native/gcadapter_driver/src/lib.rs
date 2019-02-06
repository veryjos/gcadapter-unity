#![feature(maybe_uninit)]

use std::thread;
use std::cell::RefCell;

use libusb;

mod sync_cell;

mod context;
mod adapter;

mod ffi;

/// Vendor ID for the GameCube adapter.
pub const VENDOR_ID: u16 = 0x057E;

/// Product ID for the GameCube adapter.
pub const PRODUCT_ID: u16 = 0x0337;
