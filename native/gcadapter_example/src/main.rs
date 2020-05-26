use std::collections::HashSet;
use std::sync::{Arc, Mutex};

use lazy_static::lazy_static;
use libloading::{Library, Symbol};

mod config;
use config::Config;

lazy_static! {
    static ref PLUGGED_IN: Arc<Mutex<HashSet<usize>>> = Arc::new(Mutex::new(HashSet::new()));
}

type ControllerId = usize;

#[repr(C)]
#[derive(Debug)]
struct ControllerState {
    plugged_in: bool,

    buttons: u32,
    x: f32,
    y: f32,
    cx: f32,
    cy: f32,
    l: f32,
    r: f32,
}

fn main() {
    // Parse CLI into program config
    let config = Config::from_cli();

    // Load the project library
    let lib = Library::new(config.dll_path).unwrap();

    // Load symbols from the dll and do a bit of test stuff
    macro_rules! load_symbols {
        ($($symbol_name:ident : $ty:ty),*) => {
            $(
                let $symbol_name: $ty = unsafe {
                    lib.get(stringify!($symbol_name).as_bytes())
                        .unwrap()
                };
            )*
        };
    };

    load_symbols! {
        gc_create_context: Symbol<unsafe extern fn(fn(id: ControllerId), fn(id: ControllerId)) -> *const ()>,
        gc_get_latest_controller_state: Symbol<unsafe extern fn(context: *const (), id: ControllerId) -> ControllerState>
    };

    let controller_plugged = |id: ControllerId| {
        PLUGGED_IN.lock().unwrap().insert(id);
        println!("Controller plugged in: {}", id);
    };

    let controller_unplugged = |id: ControllerId| {
        PLUGGED_IN.lock().unwrap().remove(&id);
        println!("Controller unplugged: {}", id);
    };

    let context = unsafe { gc_create_context(controller_plugged, controller_unplugged) };

    loop {
        for controller_id in PLUGGED_IN.lock().unwrap().iter() {
            let controller_state =
                unsafe { gc_get_latest_controller_state(context, *controller_id) };

            println!("{:?}", controller_state);
        }

        std::thread::sleep(std::time::Duration::from_millis(1000));
    }
}
