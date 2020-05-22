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
        gc_create_context: Symbol<unsafe extern fn() -> usize>
    };

    unsafe {
       gc_create_context();
    }

    std::thread::sleep(std::time::Duration::from_millis(10000));
}
