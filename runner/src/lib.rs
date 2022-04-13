use object::elf::FileHeader32;
use object::Endianness;

pub mod config;

pub mod report;
pub mod connection;
pub mod runner;
mod crc;
pub mod probers;

pub type ElfFile<'data> = object::read::elf::ElfFile<'data, FileHeader32<Endianness>>;

