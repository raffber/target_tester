use object::elf::FileHeader32;
use object::Endianness;

mod bindings;
pub mod config;
pub mod jlink;

#[allow(non_upper_case_globals)]
#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
#[allow(dead_code)]
pub mod jlink_sys;
pub mod report;
pub mod connection;
pub mod runner;
mod crc;
pub mod probers;

pub type ElfFile<'data> = object::read::elf::ElfFile<'data, FileHeader32<Endianness>>;

