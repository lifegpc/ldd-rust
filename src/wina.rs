#![allow(non_snake_case)]
extern crate winapi;
extern crate winsafe;

use winapi::shared::minwindef::MAX_PATH;
use winsafe::WString;

pub fn GetWindowsDirectory() -> Result<String, &'static str> {
    let mut s = WString::new_alloc_buffer(MAX_PATH + 1);
    unsafe {
        use winapi::um::sysinfoapi::GetWindowsDirectoryW;
        let r = GetWindowsDirectoryW(s.as_mut_ptr(), MAX_PATH as u32);
        if r > 0 {
            return Ok(s.to_string());
        } else {
            return Err("Can not get windows directory");
        }
    }
}

#[test]
fn test_GetWindowsDirectory() {
    GetWindowsDirectory().unwrap();
}
