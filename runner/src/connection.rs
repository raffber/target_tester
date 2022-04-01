use std::ffi::CString;
use std::time::Duration;

pub trait Connection {
    fn read_ram(&mut self, addr: u32, length: usize) -> Result<Vec<u8>, String>;

    fn write_ram(&mut self, addr: u32, data: &[u8]) -> Result<(), String>;

    fn halt(&mut self, timeout: Duration) -> Result<(), String>;

    fn run(&mut self) -> Result<(), String>;

    fn reset_run(&mut self, stack_ptr: u32, entry_point: u32) -> Result<(), String>;

    fn download(&mut self, addr: u32, data: &[u8]) -> Result<(), String>;

    fn read_utf8_string(&mut self, addr: u32) -> Result<String, String> {
        const MAX_STRLEN: usize = 256;
        let data = self.read_ram(addr, MAX_STRLEN)?;
        if let Some(pos) = data.iter().position(|&x| x == 0) {
            Ok(CString::new(&data[0..pos])
                .unwrap()
                .to_str()
                .map_err(|_| format!("Could not decode C string."))?
                .to_string())
        } else {
            Ok("".to_string())
        }
    }
}