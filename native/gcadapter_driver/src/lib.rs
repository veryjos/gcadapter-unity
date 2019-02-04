mod ffi;

use libusb;

const vendor_id: u16 = 0x057E;
const device_id: u16 = 0x0337;

struct Context {
    libusb_context: libusb::Context,
    adapters: Vec<Adapter>
}

impl Context {
    fn new() -> Result<Context, libusb::Error> {
        let libusb_context = libusb::Context::new()?;

        Ok(Context {
            libusb_context,
            adapters: vec!()
        })
    }

    fn update_adapters(&mut self) {
    }
}

struct Adapter {
    controllers: Vec<Controller>
}

struct Controller {
    plugged_in: bool,

    buttons: u32,
    x: f32,
    y: f32,
    cx: f32,
    cy: f32
}
