
use std::io::Write;
use once_cell::sync::Lazy;
use tempfile::NamedTempFile;
use crate::jlink_sys;

cfg_if::cfg_if! {
    if #[cfg(unix)] {
        const JLINK_LIB: &[u8] = include_bytes!("../3rdparty/libjlinkarm.so");
    } else {
        const JLINK_LIB: &[u8] = include_bytes!("../3rdparty/TODO.dll");
    }
}


pub static JLINK_API: Lazy<jlink_sys::JLink> = Lazy::new(|| {
    let mut tmpfile = NamedTempFile::new().unwrap();
    tmpfile.write_all(JLINK_LIB).unwrap();
    unsafe {
        jlink_sys::JLink::new(tmpfile.path()).unwrap()
    }
});
