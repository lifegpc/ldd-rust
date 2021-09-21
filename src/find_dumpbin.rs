extern crate json;
extern crate subprocess;

use json::JsonValue;
use std::fs::read_dir;
use std::path::PathBuf;
use std::time::Duration;
use subprocess::Popen;
use subprocess::PopenConfig;
use subprocess::Redirection;

/// Get vs information by using `vswhere.exe`
/// * `p` - the path to `vswhere.exe`
pub fn get_vs_info(p: &str) -> Option<JsonValue> {
    let li = vec![p, "-legacy", "-prerelease", "-format", "json"];
    let p = Popen::create(
        &li,
        PopenConfig {
            stdin: Redirection::Pipe,
            stdout: Redirection::Pipe,
            stderr: Redirection::Pipe,
            ..Default::default()
        },
    );
    if p.is_err() {
        return None;
    }
    let mut p = p.unwrap();
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
    let j = json::parse(so.as_str());
    if j.is_err() {
        println!("{}", j.unwrap_err());
        return None;
    }
    let j = j.unwrap();
    Some(j)
}

pub fn get_dumpbin_path() -> Option<String> {
    let mut r = get_vs_info("vswhere.exe");
    if r.is_none() {
        r = get_vs_info("C:\\Program Files (x86)\\Microsoft Visual Studio\\Installer\\vswhere.exe");
        if r.is_none() {
            println!("{}", "Can not find vswhere.exe");
            return None;
        }
    }
    let r = r.unwrap();
    for inst in r.members() {
        let ip = inst["installationPath"].as_str();
        if ip.is_none() {
            continue;
        }
        let ip = ip.unwrap();
        let r = find_dumpbin_in_vs_path(ip);
        if r.is_some() {
            return Some(r.unwrap());
        }
    }
    None
}

/// Find dumpbin in vs installation path.
/// * `p` - VS installation path.
pub fn find_dumpbin_in_vs_path(p: &str) -> Option<String> {
    let mut pb = PathBuf::from(p);
    pb.push("VC");
    pb.push("Tools");
    pb.push("MSVC");
    let r = read_dir(pb);
    if r.is_err() {
        println!("{}", r.unwrap_err());
        return None;
    }
    let r = r.unwrap();
    for p in r {
        if p.is_err() {
            continue;
        }
        let d = p.unwrap();
        let mut pbb = d.path();
        pbb.push("bin");
        let r2 = read_dir(pbb);
        if r2.is_err() {
            continue;
        }
        let r2 = r2.unwrap();
        for p in r2 {
            if p.is_err() {
                continue;
            }
            let d = p.unwrap();
            let pbb = d.path();
            let r3 = read_dir(pbb);
            if r3.is_err() {
                continue;
            }
            let r3 = r3.unwrap();
            for p in r3 {
                if p.is_err() {
                    continue;
                }
                let d = p.unwrap();
                let mut rp = d.path();
                rp.push("dumpbin.exe");
                if rp.exists() {
                    let rp = rp.to_str();
                    if rp.is_some() {
                        return Some(String::from(rp.unwrap()));
                    }
                }
            }
        }
    }
    None
}
