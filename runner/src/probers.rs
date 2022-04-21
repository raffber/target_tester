use std::time::Duration;

use probe_rs::{CoreRegisterAddress, MemoryInterface, Probe, Session, WireProtocol};
use probe_rs::flashing::DownloadOptions;

use crate::config::{Interface, Speed};
use crate::connection::Connection;

pub struct ProbeRsConnection {
    session: Session,
}

impl ProbeRsConnection {
    pub fn connect(device: &str, interface: Interface, speed: Speed) -> Result<Self, String> {
        let probes = Probe::list_all();
        let mut probe = probes[0].open().map_err(|x| format!("{}", x))?;

        match interface {
            Interface::JTAG => probe.select_protocol(WireProtocol::Jtag).map_err(|x| format!("{}", x))?,
            Interface::SWD => probe.select_protocol(WireProtocol::Swd).map_err(|x| format!("{}", x))?,
        }
        match speed {
            Speed::KHz(x) => {
                probe.set_speed(x as u32).map_err(|x| format!("{}", x))?;
            }
            Speed::Auto => {
                probe.set_speed(4000).map_err(|x| format!("{}", x))?;
            }
            _ => {}
        }

        let session = probe.attach(device).map_err(|x| format!("{}", x))?;

        Ok(Self { session })
    }
}

impl Connection for ProbeRsConnection {
    fn read_ram(&mut self, addr: u32, length: usize) -> Result<Vec<u8>, String> {
        let mut buf = vec![0_u8; length];
        let mut core = self.session.core(0).map_err(|x| format!("{}", x))?;
        core.read_8(addr, &mut buf).map_err(|x| format!("{}", x))?;
        Ok(buf)
    }

    fn write_ram(&mut self, addr: u32, data: &[u8]) -> Result<(), String> {
        let mut core = self.session.core(0).map_err(|x| format!("{}", x))?;
        core.write_8(addr, data).map_err(|x| format!("{}", x))?;
        Ok(())
    }

    fn halt(&mut self, timeout: Duration) -> Result<(), String> {
        let mut core = self.session.core(0).map_err(|x| format!("{}", x))?;
        let _ = core.halt(timeout).map_err(|x| format!("{}", x))?;
        Ok(())
    }

    fn run(&mut self) -> Result<(), String> {
        let mut core = self.session.core(0).map_err(|x| format!("{}", x))?;
        core.run().map_err(|x| format!("{}", x))?;
        Ok(())
    }

    fn reset_run(&mut self, stack_ptr: u32, entry_point: u32) -> Result<(), String> {
        let mut core = self.session.core(0).map_err(|x| format!("{}", x))?;
        core.reset_and_halt(Duration::from_millis(300)).map_err(|x| format!("{}", x))?;
        core.write_core_reg(CoreRegisterAddress(13), stack_ptr).map_err(|x| format!("{}", x))?;
        core.write_core_reg(CoreRegisterAddress(15), entry_point).map_err(|x| format!("{}", x))?;
        core.run().map_err(|x| format!("{}", x))?;
        Ok(())
    }

    fn download(&mut self, addr: u32, data: &[u8]) -> Result<(), String> {
        {
            let mut buf = vec![0_u8; data.len()];
            let mut core = self.session.core(0).map_err(|x| format!("{}", x))?;
            core.read_8(addr, &mut buf).map_err(|x| format!("{}", x))?;
            if data == &buf {
                return Ok(());
            }
        }
        let mut loader = self.session.target().flash_loader();
        loader.add_data(addr, data).expect("Invalid flash regions in binary");
        loader.commit(&mut self.session, DownloadOptions::default()).map_err(|x| format!("{}", x))?;
        Ok(())
    }
}
