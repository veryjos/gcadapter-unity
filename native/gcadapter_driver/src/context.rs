use std::thread;
use std::sync::mpsc::{Sender, Receiver, channel};
use std::collections::HashSet;

use libusb;

use crate::{ VENDOR_ID, PRODUCT_ID };

use crate::adapter::Adapter;

/// Context for the GameCube adapter daemon
pub struct Context {
    join_handle: thread::JoinHandle<()>,

    adapter_receiver: Receiver<Adapter>,
    adapters: Vec<Adapter>
}

fn hotplug_thread(sender: Sender<Adapter>, libusb_context: libusb::Context) {
    let mut plugged_in = HashSet::new();

    loop {
        // TODO: Support true hotplug
        // Poll for adapters
        match libusb_context.devices() {
            Ok(device_list) => {
                device_list.iter()
                    .filter(|device| {
                        !plugged_in.contains(device.address())
                    })
                    .filter(|device| {
                        match device.device_descriptor() {
                            Ok(desc) => {
                                desc.vendor_id()  == VENDOR_ID &&
                                desc.product_id() == PRODUCT_ID
                            },

                            _ => false
                        }
                    })
                    .for_each(|device| {
                        // Ship these new devices off to the context for proper handling
                        println!("New device plugged in: {:?}", device.address());
                        plugged_in.insert(device.address());

                        sender.send(Adapter::new());
                    }
            },

            Err(_) => panic!("Failed to list USB devices")
        };

        thread::sleep(std::time::Duration::from_millis(2500));
    }
}

impl Context {
    /// Creates a new Context and starts an input thread.
    pub fn new() -> Result<Context, libusb::Error> {
        let libusb_context = libusb::Context::new()?;
        let (sender, adapter_receiver) = channel();
        let join_handle = thread::spawn(move || {
            hotplug_thread(sender, libusb_context);
        });

        Ok(Context {
            join_handle,

            adapter_receiver,
            adapters: vec!()
        })
    }
}
