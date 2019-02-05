use clap::{Arg, App};

pub struct Config {
    pub dll_path: String
}

impl Config {
    pub fn from_cli() -> Config {
        let matches = App::new("gcadapter-unity test")
            .version("1.0")
            .author("Joseph Delgado <downsider002@gmail.com>")
            .about("Tests gcadapter-unity dll")

            .arg(Arg::with_name("path")
                .short("p")
                .long("path")
                .value_name("PATH")
                .help("Path to gcadapter-unity dll")
                .required(true)
            )

            .get_matches();

        Config {
            dll_path: matches.value_of("path").unwrap().to_string()
        }
    }
}
