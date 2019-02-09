use std::thread;
use std::cell::RefCell;

use libusb;

use crate::{ VENDOR_ID, PRODUCT_ID };

use crate::adapter::Adapter;

/// Context for the GameCube adapter daemon. Owns all the
/// adapters plugged into the system.
pub struct Context {
    join_handle: thread::JoinHandle<()>
}

fn hotplug_thread(libusb_context: libusb::Context) {
    loop {
        // TODO: Support true hotplug
        // Poll for adapters
        let all_adapters: Vec<_> = match libusb_context.devices() {
            Ok(device_list) => {
                device_list.iter()
                    .filter(|device| {
                        match device.device_descriptor() {
                            Ok(desc) => {
                                desc.vendor_id()  == VENDOR_ID &&
                                desc.product_id() == PRODUCT_ID
                            },

                            _ => false
                        }
                    })
                    .collect()
            },

            Err(_) => panic!("Failed to list USB devices")
        };

        println!("{:?}", all_adapters.iter().map(|a| { a.address() }).collect::<Vec<_>>());

        thread::sleep(std::time::Duration::from_millis(1000));
    }
}

impl Context {
    /// Creates a new Context and starts an input thread.
    pub fn new() -> Result<Context, libusb::Error> {
        let libusb_context = libusb::Context::new()?;
        let join_handle = thread::spawn(move || {
            hotplug_thread(libusb_context);
        });

        Ok(Context {
            join_handle
        })
    }
}
