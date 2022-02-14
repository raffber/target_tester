mod config;
mod jlink;
mod bindings;

use object::read::File;
use crate::config::Interface;


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


pub struct Runner {
}

impl Runner {
    pub fn initialize() -> Result<Self, String> {
        // for device in jaylink::scan_usb().unwrap() {
        //     let device = device.open().map_err(|x| format!("{}", x))?;
        //     return Ok(Self {
        //         link: device
        //     });
        // }
        return Err(format!("No J-Link device found."));
    }

    pub fn configure(&mut self, interface: Interface, speed_khz: u16) -> Result<(), String> {
        // self.link.select_interface(interface.into()).map_err(|x| x.to_string())?;
        // let speed_config = SpeedConfig::khz(speed_khz).ok_or(format!("Invalid speed"))?;
        // self.link.set_speed(speed_config).map_err(|x| x.to_string())
        return Err(format!("Invalid Config"));
    }
}



