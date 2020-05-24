use std::mem;

use crate::controller::{ControllerId, ControllerState};
use crate::context::Context;

pub type ControllerPluggedCallback = fn(id: ControllerId);
pub type ControllerUnpluggedCallback = fn(id: ControllerId);

#[no_mangle]
extern "C" fn gc_create_context(
    controller_plugged: ControllerPluggedCallback,
    controller_unplugged: ControllerUnpluggedCallback
) -> usize {
    let context: Context = Context::new(controller_plugged, controller_unplugged);

    unsafe {
        mem::transmute(Box::leak(Box::new(context)))
    }
}

#[no_mangle]
extern "C" fn gc_latest_controller_state(
    context: *const Context,
    id: ControllerId,
) -> *const ControllerState {
    let context = unsafe { &*context };

    context.get_latest_controller_state(id)
}