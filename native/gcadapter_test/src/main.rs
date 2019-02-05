use libloading::{ Library, Symbol };

mod config;
use config::Config;

fn main() {
    // Parse CLI into program config
    let config = Config::from_cli();

    // Load the project library
    let lib = Library::new(config.dll_path).unwrap();

    let list_devices: Symbol<unsafe extern fn() -> ()> = unsafe {
        lib.get(b"list_devices\0").unwrap()
    };

    unsafe {
        list_devices();
    }
}
