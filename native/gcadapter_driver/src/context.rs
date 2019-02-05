use std::thread;
use std::cell::RefCell;

use libusb;

/// Context for the GameCube adapter daemon. Owns all the
/// adapters plugged into the system.
struct Context {
    join_handle: thread::JoinHandle<()>,

    libusb_context: libusb::Context,
    adapters: RefCell<Vec<Adapter>>
}

impl Context {
    /// Creates a new Context and starts an input thread.
    fn new() -> Result<Context, libusb::Error> {
        let libusb_context = libusb::Context::new()?;
        thread::spawn(|| {
        })

        Ok(Context {
            libusb_context,
            adapters: RefCell::new(vec!())
        })
    }

    fn start_thread(&mut self) {
        loop {
            // Poll for new adapters
            for mut device in context.devices().unwrap().iter() {
                let device_desc = device.device_descriptor().unwrap();

                if device_desc.vendor_id()  == vendor_id &&
                   device_desc.product_id() == product_id {
                    // Found a new GameCube adapter.
                    self.adapters.push(Adapter::new(device));
                }
            }

            // Work each adapter
            for adapter in self.adapters {
            }
        }
    }
}
