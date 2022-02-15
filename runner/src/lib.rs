mod config;
mod jlink;
mod bindings;

#[allow(non_upper_case_globals)]
#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
#[allow(dead_code)]
pub mod jlink_sys;

use std::cmp::Ordering;
use std::fmt::{Debug, Formatter};
use std::iter::repeat;
use object::{Object, ObjectSection, ObjectSegment, SectionFlags, SegmentFlags};
use object::elf::PT_LOAD;
use object::read::File;
use object::elf::FileHeader32;
use object::Endianness;

use crate::config::{Interface, Speed};
use crate::jlink_sys::{JLINKARM_SPEED_ADAPTIVE, JLINKARM_SPEED_AUTO, JLINKARM_TIF_JTAG, JLINKARM_TIF_SWD};

pub type ElfFile<'data> = object::read::elf::ElfFile<'data, FileHeader32<Endianness>>;

pub struct TestBinary<'data> {
    pub file: ElfFile<'data>,
}

impl<'data> TestBinary<'data> {
    pub fn new(file: ElfFile<'data>) -> Self {
        Self {
            file
        }
    }
}

pub struct Connection {}

impl Connection {
    pub fn connect(device: &str, speed: Speed, interface: Interface) -> Result<Self, String> {
        jlink::open(None)?;
        Self::use_batch_mode();
        jlink::exec_command(&format!("device = {}", device)).map(|_| ())?;
        jlink::set_tif(interface)?;
        jlink::set_speed(speed)?;
        jlink::connect()?;
        Ok(Connection {})
    }

    fn use_batch_mode() -> Result<(), String> {
        jlink::exec_command("SilentUpdateFW")?;
        jlink::exec_command("SuppressInfoUpdateFW")?;
        jlink::exec_command("SetBatchMode = 1")?;
        jlink::exec_command("HideDeviceSelection = 1")?;
        jlink::exec_command("SuppressControlPanel")?;
        jlink::exec_command("DisableInfoWinFlashDL")?;
        jlink::exec_command("DisableInfoWinFlashBPs").map(|_| ())
    }
}

pub struct LoadSegment {
    addr: u64,
    data: Vec<u8>,
}

impl Debug for LoadSegment {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("LoadSegment(0x{:x}, 0x{:x})", self.addr, self.data.len()))
    }
}

impl LoadSegment {
    pub fn get_from_file(binary: &TestBinary) -> Vec<LoadSegment> {
        let mut segments_to_load = Vec::new();
        let file_data = binary.file.data();
        for header in binary.file.raw_segments().iter() {
            let p_type = header.p_type.get(Endianness::Little);
            if p_type & PT_LOAD == 0 {
                continue;
            }
            let file_size = header.p_filesz.get(Endianness::Little) as usize;
            if file_size == 0 {
                continue;
            }
            let offset = header.p_offset.get(Endianness::Little) as usize;
            let physical_address = header.p_paddr.get(Endianness::Little);
            let load_data = &file_data[offset..offset + file_size];
            let segment = LoadSegment {
                addr: physical_address as u64,
                data: load_data.to_vec(),
            };
            segments_to_load.push(segment);
        }
        segments_to_load
    }

    pub fn collapse_segments(mut segments: Vec<LoadSegment>) -> Option<LoadSegment> {
        if segments.len() < 1 {
            return None;
        }
        segments.sort_by(|x, y| {
            if x.addr > y.addr {
                Ordering::Greater
            } else if x.addr < y.addr {
                Ordering::Less
            } else {
                Ordering::Equal
            }
        });
        let first = &segments[0];
        let mut binary = first.data.to_vec();
        let mut current_addr = first.addr as usize + first.data.len();
        for segment in &segments[1 .. segments.len()] {
            let segment_addr = segment.addr as usize;
            if segment_addr > current_addr {
                binary.extend(repeat(0xFF).take(segment_addr - current_addr));
            }
            binary.extend(&segment.data);
            current_addr = segment_addr + segment.data.len();
        }
        Some(LoadSegment {
            addr: first.addr,
            data: binary
        })
    }
}

pub struct Runner {
    segments_to_load: Vec<LoadSegment>,
}

impl Runner {
    pub fn new(binary: &TestBinary) -> Self {
        let segments_to_load = LoadSegment::get_from_file(binary);

        Self {
            segments_to_load
        }
    }

    pub fn reset(&mut self, pc: u64, sp: u64) {
        // jlink::halt();
        jlink::reset_device();
        // jlink::set
    }

    // pub fn upload(binary: &TestBinary, vector_table: usize) -> Result<(), String> {}
}



