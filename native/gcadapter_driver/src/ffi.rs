use std::mem;

use crate::context::Context;

#[no_mangle]
extern "C" fn gc_create_context() -> usize {
    match Context::new() {
        Ok(context) => {
            unsafe { mem::transmute(Box::leak(Box::new(context))) }
        },

        Err(err) => {
            println!("Error creating libusb context: {}", err);

            0
        }
    }
}
