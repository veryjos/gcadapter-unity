use libloading::{ Library, Symbol };

mod config;
use config::Config;

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
        list_devices: Symbol<unsafe extern fn() -> ()>
    };

    unsafe {
        list_devices();
    }
}
