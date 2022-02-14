use std::io::Write;
use libloading::Library;
use once_cell::sync::Lazy;
use tempfile::NamedTempFile;
use cfg_if;


cfg_if::cfg_if! {
    if #[cfg(unix)] {
        const JLINK_LIB: &[u8] = include_bytes!("../3rdparty/libjlinkarm.so");
    } else {
        const JLINK_LIB: &[u8] = include_bytes!("../3rdparty/TODO.dll");
    }
}


static GLOBAL_DATA: Lazy<libloading::Library> = Lazy::new(|| {
    let mut tmpfile = NamedTempFile::new().unwrap();
    tmpfile.write_all(JLINK_LIB).unwrap();
    unsafe {
        Library::new(tmpfile.path()).unwrap()
    }
});

pub struct JLink {


}

impl JLink {
    pub fn scan() -> Vec<JLink> {
        todo!()
    }
}
