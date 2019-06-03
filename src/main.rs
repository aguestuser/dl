use dl::Config;

use std::env;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    let cfg = Config::new(args).unwrap_or_else(|err| {
        eprintln!("Could not parse arguments: {}", err);
        process::exit(1);
    });

    // if let Err(e) = dl::run(cfg) {
    //     eprintln!("Application error: {}", e);
    //     process::exit(1);
    // }
}
