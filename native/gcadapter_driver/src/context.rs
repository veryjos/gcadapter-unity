use std::thread;
use std::mem::{transmute_copy};
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{Sender, Receiver, channel};
use std::collections::HashSet;

use libusb;

use crate::{VENDOR_ID, PRODUCT_ID};

use crate::adapter::Adapter;

/// Context for the GameCube adapter daemon
pub struct Context {
    // Must be a box because must be pinned for an unsafe static reference
    libusb_context: Box<libusb::Context>,
    hotplug_thread_handle: thread::JoinHandle<()>,

    adapter_receiver: Receiver<(u8,Option<Adapter>)>,
    adapters: Vec<Adapter>,
}

fn hotplug_thread(sender: Sender<(u8, Option<Adapter>)>, libusb_context: &'static libusb::Context) {
    let mut plugged_in = HashSet::new();

    loop {
        // TODO: Support true OS hotplug
        let devices = libusb_context.devices().expect("Failed to list USB devices");

        // Send off plug and events
        for device in devices.iter() {
            let desc = device.device_descriptor().unwrap();

            // Check if it's the right device type
            if desc.vendor_id()  != VENDOR_ID ||
               desc.product_id() != PRODUCT_ID {
                continue;
            }

            if !plugged_in.contains(&device.address()) {
                // New device, send it off to the context
                let address = device.address();
                println!("New device plugged in: {:?}", address);
                plugged_in.insert(address);

                let adapter = Adapter::new(device);
                sender.send((address, Some(adapter)));
            }
        }

        // Send off unplug events
        plugged_in.retain(|address| {
            match devices.iter()
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

        thread::sleep(std::time::Duration::from_millis(2500));
    }
}

impl Context {
    /// Creates a new Context and starts an input thread.
    pub fn new() -> Result<Context, libusb::Error> {
        let libusb_context = Box::new(libusb::Context::new()
            .expect("Failed to open libusb context"));

        let (sender, adapter_receiver) = channel();
        let libusb_context_ref = unsafe { transmute_copy(&libusb_context) };
        let hotplug_thread_handle = thread::spawn(move || {
            hotplug_thread(sender, libusb_context_ref);
        });

        Ok(Context {
            libusb_context,
            hotplug_thread_handle,

            adapter_receiver,
            adapters: vec!()
        })
    }

    // Updates each adapter
    pub fn tick(&mut self) {
        // Handle each adapter added/removed event
        while let Ok(event) = self.adapter_receiver.try_recv() {
            match event {
                (address, Some(adapter)) => unsafe {
                    println!("Got adapter at address {}", address);
                    self.adapters.push(adapter);
                },

                (address, None) => {
                    println!("Adapter from address {}", address);
                    self.adapters.retain(|adapter| {
                        adapter.address() != address
                    })
                }
            }
        }
    }
}
