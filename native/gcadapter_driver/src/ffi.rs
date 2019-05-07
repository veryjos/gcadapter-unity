use std::mem;
use std::ptr;
use std::sync::Arc;
use std::ffi::c_void;

use crate::adapter::ControllerState;
use crate::context::Context;


pub struct ControllerPtr(());
pub type ControllerHandle = *mut ControllerPtr;

/// Creates a new [`Context`] and returns a pointer to the object.
///
/// This should be called once when your program starts, and the
/// resulting pointer can be used with gc_context_* calls.
///
/// The pointer is owned by the caller and must be freed with
/// [`gc_delete_context`].
#[no_mangle]
extern "C" fn gc_context_create(
    controller_plug_callback: extern "C" fn(i32, i32) -> ControllerHandle,
    controller_unplug_callback: extern "C" fn(ControllerHandle) -> (),
    controller_state_callback: extern "C" fn(ControllerHandle, *const ControllerState) -> ()
) -> *mut Context {
    match Context::new(
        Arc::new(|adapter_id, port| {
            println!("controller plugged in");
        }),

        Arc::new(|controller_handle| {
            println!("controller unplugged");
        }),

        Arc::new(|controller_handle| {
            println!("controller state updated");
        })
    ) {
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
extern "C" fn gc_context_get_active_controllers(ptr: &mut Context) {
}
