mod dep;
mod find_dumpbin;
mod opts;
mod wina;

struct Main {
    o: opts::Opts,
}

impl Main {
    pub fn new() -> Self {
        Self {
            o: opts::Opts::new(),
        }
    }

    pub fn run(&self) -> i32 {
        let p = find_dumpbin::get_dumpbin_path();
        if p.is_none() {
            return -1;
        }
        let p = p.unwrap();
        if self.o.verbose() {
            println!("{}", p);
        }
        let mut d = dep::Dep::new(p.as_str(), self.o.f.as_str(), &self.o);
        let r = d.find();
        if !r {
            return 1;
        }
        return 0;
    }
}

fn main() {
    let m = Main::new();
    std::process::exit(m.run());
}
