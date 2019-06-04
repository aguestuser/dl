use dl::Config;
use futures::future::Future;
use hyper::rt;
use std::env;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    let cfg = Config::new(args).unwrap_or_else(|err| {
        eprintln!("{}", err);
        process::exit(1);
    });

    rt::run(rt::lazy(|| {
        dl::run(cfg).map_err(|err| {
            eprintln!("> Errror: {}", err);
            process::exit(1);
        })
    }));
}
