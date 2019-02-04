use libloading::{ Library, Symbol };

fn main() {
    let lib = Library::new("./target/debug/deps/libgcadapter_driver.dylib").unwrap();

    let list_devices: Symbol<unsafe extern fn() -> ()> = unsafe {
        lib.get(b"list_devices\0").unwrap()
    };

    unsafe {
        list_devices();
    }
}
