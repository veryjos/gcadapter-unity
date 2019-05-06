use std::mem;
use std::ptr;

use crate::context::Context;

/// Creates a new [`Context`] and returns a pointer to the object.
///
/// This should be called once when your program starts, and the
/// resulting pointer can be used with gc_context_* calls.
///
/// The pointer is owned by the caller and must be freed with
/// [`gc_delete_context`].
#[no_mangle]
extern "C" fn gc_context_create() -> *mut Context {
    match Context::new() {
        Ok(context) => {
            Box::leak(Box::new(context))
        },

        Err(err) => {
            eprintln!("Error creating libusb context: {}", err);

            ptr::null_mut()
        }
    }
}

/// Deletes a previously allocated [`Context`] and all associated resources.
///
/// This should be called once when your program ends.
#[no_mangle]
extern "C" fn gc_context_delete(ptr: *mut Context) {
    unsafe { drop(Box::from_raw(ptr)); };

    eprintln!("Deleted context");
}

/// Updates the context and any associated resources.
///
/// This doesn't have to be called once per frame, this just consumes
/// messages generated from the worker thread.
#[no_mangle]
extern "C" fn gc_context_update(ptr: &mut Context) {
    ptr.update();
}

#[no_mangle]
extern "C" fn gc_context_set_controller_callbacks(
    ptr: &mut Context,
    controller_plug_callback: extern "C" fn(i32) -> (),
    controller_unplug_callback: extern "C" fn(i32) -> (),
    controller_state_callback: extern "C" fn(i32) -> ()
) {
    unsafe {
        controller_plug_callback(1);
        controller_unplug_callback(1);
        controller_state_callback(1);
    }
}

#[no_mangle]
extern "C" fn gc_context_get_active_controllers(ptr: &mut Context) {
}
