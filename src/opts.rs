extern crate getopts;

use getopts::Matches;
use getopts::Options;
use std::env;

fn create_opts() -> Options {
    let mut o = Options::new();
    o.optflag("h", "help", "Print help message.");
    o.optflag("v", "verbose", "Print verbose message.");
    o.optflag("V", "version", "Print version information.");
    o.optflag("s", "system", "Find system's dlls' dependencies too.");
    o.optflag("d", "delay", "Included delay load dependencies.");
    o
}

fn print_help(o: &Options) {
    let b = format!(
        "{}",
        "Usage: ldd [options] FILE\nA tool to analysis dependents by using dumpbin."
    );
    println!("{}", o.usage(&b));
}

fn print_version() {
    let v = env!("CARGO_PKG_VERSION");
    println!("ldd-rust  v{}  Copyright (C) 2021  lifegpc", v);
    println!("This program comes with ABSOLUTELY NO WARRANTY; for details type `show w'.");
    println!("This is free software, and you are welcome to redistribute it");
    println!("under certain conditions.");
}

#[derive(Clone)]
pub struct Opts {
    pub m: Matches,
    pub f: String,
}

impl Opts {
    pub fn new() -> Self {
        let o = create_opts();
        let args: Vec<String> = env::args().collect();
        let matches = match o.parse(&args[1..]) {
            Ok(m) => m,
            Err(f) => {
                panic!("{}", f.to_string())
            }
        };
        if matches.opt_present("h") {
            print_help(&o);
            std::process::exit(0);
        }
        if matches.opt_present("V") {
            print_version();
            std::process::exit(0);
        }
        let i = if !matches.free.is_empty() {
            matches.free[0].clone()
        } else {
            print_help(&o);
            std::process::exit(1);
        };
        Self { m: matches, f: i }
    }

    /// whether to verbose mode
    pub fn verbose(&self) -> bool {
        self.m.opt_present("v")
    }

    pub fn system(&self) -> bool {
        self.m.opt_present("s")
    }

    pub fn delay(&self) -> bool {
        self.m.opt_present("d")
    }
}
