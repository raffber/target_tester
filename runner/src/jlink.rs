use std::ffi::{CStr, CString, OsStr};
use std::net::{IpAddr, SocketAddr};
use std::os::raw::c_char;
use crate::bindings::JLINK_API;

pub struct JLink {


}

impl JLink {
    pub fn open(addr: Option<SocketAddr>) -> Result<JLink, String> {
        if let Some(addr) = addr {
            let ip = CString::new(addr.ip().to_string()).unwrap();
            unsafe  {
                let result = JLINK_API.JLINKARM_SelectIP(ip.as_ptr(), addr.port() as i32);
                if result == 1 {
                    return Err(format!("Could not connect to emulator on {}", addr));
                }
            }
        } else {
            unsafe  {
                let result = JLINK_API.JLINKARM_SelectUSB(0);
                if result != 0 {
                    return Err("Could not connect to default emulator!".to_string());
                }
            }
        }
        let c_str = unsafe {
            let data = JLINK_API.JLINK_Open();
            if !data.is_null() {
                Some(CStr::from_ptr(data))
            } else {
                None
            }
        };
        if let Some(err) = c_str {
            let err = err.to_str().unwrap().to_string();
            return Err(err);
        }
        Ok(JLink {})
    }

}
