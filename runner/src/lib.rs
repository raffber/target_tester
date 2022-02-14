mod config;
mod jlink;
mod bindings;

#[allow(non_upper_case_globals)]
#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
#[allow(dead_code)]
pub mod jlink_sys;

use object::read::File;
use crate::config::Interface;
use crate::jlink::JLink;


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
    jlink: JLink
}

impl Runner {
    pub fn initialize() -> Result<Self, String> {
        Ok(Runner {
            jlink: JLink::open(None)?
        })
    }

    pub fn configure(&mut self, interface: Interface, speed_khz: u16) -> Result<(), String> {
        // self.link.select_interface(interface.into()).map_err(|x| x.to_string())?;
        // let speed_config = SpeedConfig::khz(speed_khz).ok_or(format!("Invalid speed"))?;
        // self.link.set_speed(speed_config).map_err(|x| x.to_string())
        return Err(format!("Invalid Config"));
    }
}



