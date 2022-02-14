mod config;
mod jlink;
mod bindings;

#[allow(non_upper_case_globals)]
#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
#[allow(dead_code)]
pub mod jlink_sys;

use object::read::File;
use crate::config::{Interface, Speed};
use crate::jlink_sys::{JLINKARM_SPEED_ADAPTIVE, JLINKARM_SPEED_AUTO, JLINKARM_TIF_JTAG, JLINKARM_TIF_SWD};


pub struct TestBinary<'data> {
    pub file: File<'data>,
}

impl<'data> TestBinary<'data> {
    pub fn new(file: File<'data>) -> Self {
        Self {
            file
        }
    }
}


pub struct Runner {}

impl Runner {
    fn use_batch_mode() -> Result<(), String> {
        jlink::exec_command("SilentUpdateFW")?;
        jlink::exec_command("SuppressInfoUpdateFW")?;
        jlink::exec_command("SetBatchMode = 1")?;
        jlink::exec_command("HideDeviceSelection = 1")?;
        jlink::exec_command("SuppressControlPanel")?;
        jlink::exec_command("DisableInfoWinFlashDL")?;
        jlink::exec_command("DisableInfoWinFlashBPs").map(|_| ())
    }

    pub fn connect(device: &str, speed: Speed, interface: Interface) -> Result<Self, String> {
        jlink::open(None)?;
        Self::use_batch_mode();
        jlink::exec_command(&format!("device = {}", device)).map(|_| ())?;
        jlink::set_tif(interface)?;
        jlink::set_speed(speed)?;
        jlink::connect()?;
        Ok(Runner {})
    }

    pub fn reset(&mut self) {
        jlink::reset_device();
    }
}



