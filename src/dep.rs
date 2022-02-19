extern crate subprocess;
extern crate winsafe;

use crate::opts::Opts;
use crate::wina::GetWindowsDirectory;
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;
use subprocess::Popen;
use subprocess::PopenConfig;
use subprocess::Redirection;
use winsafe::GetSystemDirectory;

pub fn get_depend_from_text(inp: &str, edl: bool) -> Vec<String> {
    let ll = inp.split("\n");
    let mut r: Vec<String> = [].to_vec();
    let mut d = false;
    for i in ll {
        let s = i.trim();
        if s == "Image has the following dependencies:" {
            d = true;
        } else if s == "Summary" {
            d = false;
        } else if d && s.len() > 0 {
            if s == "Image has the following delay load dependencies:" {
                if !edl {
                    d = false;
                }
                continue;
            }
            r.push(String::from(s));
        }
    }
    r
}

/// Get dependences with dumpbin.
/// * `dp` - The path to dumpbin.exe
/// * `p` - The path to program
pub fn get_depend(dp: &str, p: &str, edl: bool) -> Option<Vec<String>> {
    let li = vec![dp, "/dependents", p];
    let r = Popen::create(
        &li,
        PopenConfig {
            stdin: Redirection::Pipe,
            stdout: Redirection::Pipe,
            stderr: Redirection::Pipe,
            ..Default::default()
        },
    );
    if r.is_err() {
        println!("{}", r.unwrap_err());
        return None;
    }
    let mut p = r.unwrap();
    let mut so = String::from("");
    loop {
        let t = p.communicate(Some(""));
        if t.is_err() {
            println!("{}", t.unwrap_err());
            return None;
        }
        let t = t.unwrap().0;
        if t.is_some() {
            so += t.unwrap().as_str();
        }
        let r = p.wait_timeout(Duration::new(1, 0));
        if r.is_err() {
            println!("{}", r.unwrap_err());
            return None;
        }
        let r = r.unwrap();
        if r.is_some() {
            let r = r.unwrap();
            if !r.success() {
                println!("{}", so);
                return None;
            }
            break;
        }
    }
    let r = get_depend_from_text(so.as_str(), edl);
    Some(r)
}

pub fn get_path_if_exists(p: &str, n: &str) -> Option<String> {
    let mut bp = PathBuf::from(p);
    bp.push(n);
    if bp.exists() {
        Some(String::from(bp.to_str().unwrap()))
    } else {
        None
    }
}

pub struct Dep {
    /// The path to dumpbin.exe
    dp: String,
    /// The path to program
    p: String,
    map: HashMap<String, String>,
    /// System directory
    sd: String,
    /// Windows directory
    wd: String,
    /// `PATH` enveronment
    path: Vec<String>,
    /// The directory of program
    pd: String,
    opt: Opts,
}

impl Dep {
    pub fn new(dp: &str, p: &str, o: &Opts) -> Self {
        Self {
            dp: String::from(dp),
            p: String::from(p),
            map: HashMap::new(),
            sd: GetSystemDirectory().unwrap(),
            wd: GetWindowsDirectory().unwrap(),
            path: match std::env::var("PATH") {
                Ok(v) => {
                    let s = v.split(";");
                    let mut r = [].to_vec();
                    for i in s {
                        let i = i.trim();
                        if i.len() == 0 {
                            continue;
                        }
                        r.push(String::from(i));
                    }
                    r
                }
                Err(_) => [].to_vec(),
            },
            pd: {
                let mut pb = PathBuf::from(p);
                if !pb.pop() {
                    String::from(".")
                } else {
                    String::from(pb.to_str().unwrap())
                }
            },
            opt: o.clone(),
        }
    }

    pub fn find(&mut self) -> bool {
        let pp = String::from(self.p.as_str());
        self.get_depend(pp.as_str())
    }

    pub fn get_depend(&mut self, p: &str) -> bool {
        let r = get_depend(self.dp.as_str(), p, self.opt.delay());
        if r.is_none() {
            return false;
        }
        let r = r.unwrap();
        if self.opt.verbose() {
            println!("Find follow dependences for {}:", p);
            for i in &r {
                println!("\t{}", &i);
            }
        }
        for i in &r {
            if i.starts_with("LINK : ") {
                continue;
            }
            if !self.search_dll(i.as_str()) {
                return false;
            }
        }
        true
    }

    pub fn search_dll(&mut self, d: &str) -> bool {
        let sd = d.to_lowercase();
        if sd.starts_with("api-ms-win-") {
            return true;
        }
        if sd.starts_with("ext-ms-") {
            return true;
        }
        if self.map.contains_key(sd.as_str()) {
            return true;
        }
        let r = self.search_dll_internal(d);
        if r.is_some() {
            println!("{} -> {}", d, r.as_ref().unwrap().0.as_str());
            self.map.insert(sd, String::from(r.as_ref().unwrap().0.as_str()));
            if !r.as_ref().unwrap().1 {
                return true;
            }
            return self.get_depend(r.as_ref().unwrap().0.as_str());
        }
        println!("Can not find {}", d);
        false
    }

    fn search_dll_internal(&self, d: &str) -> Option<(String, bool)> {
        let r = get_path_if_exists(self.pd.as_str(), d);
        if r.is_some() {
            return Some((r.unwrap(), true));
        }
        let r = get_path_if_exists(self.sd.as_str(), d);
        if r.is_some() {
            return Some((r.unwrap(), self.opt.system()));
        }
        let r = get_path_if_exists(self.wd.as_str(), d);
        if r.is_some() {
            return Some((r.unwrap(), self.opt.system()));
        }
        for i in &self.path {
            let r = get_path_if_exists(i.as_str(), d);
            if r.is_some() {
                return Some((r.unwrap(), true));
            }
        }
        None
    }
}
