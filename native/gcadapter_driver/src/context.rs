use std::thread;
use std::sync::mpsc::{Sender, Receiver, channel};
use std::collections::HashSet;

use libusb;

use crate::{ VENDOR_ID, PRODUCT_ID };

use crate::adapter::Adapter;

/// Context for the GameCube adapter daemon
pub struct Context {
    join_handle: thread::JoinHandle<()>,

    adapter_receiver: Receiver<(u8, Option<Adapter>)>,
    adapters: Vec<Adapter>
}

fn hotplug_thread(sender: Sender<(u8, Option<Adapter>)>, libusb_context: libusb::Context) {
    let mut plugged_in = HashSet::new();

    loop {
        // TODO: Support true OS hotplug
        match libusb_context.devices() {
            Ok(device_list) => {
                // Send off plug and events
                for device in device_list.iter() {
                    let desc = device.device_descriptor().unwrap();

                    // Check if it's the right device type
                    if desc.vendor_id()  != VENDOR_ID ||
                       desc.product_id() != PRODUCT_ID {
                        continue;
                    }

                    if !plugged_in.contains(&device.address()) {
                        // New device, send it off to the context
                        println!("New device plugged in: {:?}", device.address());
                        plugged_in.insert(device.address());

                        sender.send((device.address(), Some(Adapter::new())));
                    }
                }

                // Send off unplug events
                plugged_in.retain(|address| {
                    match device_list.iter()
                        .find(|device| {
                            device.address() == *address
                        }) {
                            None => {
                                println!("Device unplugged from: {:?}", address);
                                sender.send((*address, None));

                                false
                            },
                            _ => true
                        }
                });
            },

            Err(_) => panic!("Failed to list USB devices")
        }

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
