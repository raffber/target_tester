pub mod config;
mod jlink;
mod bindings;

#[allow(non_upper_case_globals)]
#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
#[allow(dead_code)]
pub mod jlink_sys;

use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::iter::repeat;
use std::net::SocketAddr;
use std::str::FromStr;
use std::thread::sleep;
use std::time::{Duration, Instant};
use object::{Object, ObjectSymbol};
use object::elf::PT_LOAD;
use object::elf::FileHeader32;
use object::Endianness;
use byteorder::{ByteOrder, LittleEndian};
use regex::Regex;

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
        jlink::open(Some(SocketAddr::from_str("127.0.0.1:19020").unwrap()))?;
        Self::use_batch_mode()?;
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
        jlink::exec_command("DisableInfoWinFlashBPs")?;
        Ok(())
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
        for segment in &segments[1..segments.len()] {
            let segment_addr = segment.addr as usize;
            if segment_addr > current_addr {
                binary.extend(repeat(0xFF).take(segment_addr - current_addr));
            }
            binary.extend(&segment.data);
            current_addr = segment_addr + segment.data.len();
        }
        Some(LoadSegment {
            addr: first.addr,
            data: binary,
        })
    }

    pub fn start_addr(&self) -> u64 {
        self.addr
    }

    pub fn end_addr(&self) -> u64 {
        self.addr + (self.data.len() as u64)
    }
}

pub struct TestCase {
    pub suite_name: String,
    pub test_name: String,
    pub addr: u64,
}

impl TestCase {
    pub fn symbol_name(&self) -> String {
        format!("__test_{}__target_test__{}", self.suite_name, self.test_name)
    }
}

pub struct FailedAssert {
    lhs: u32,
    rhs: u32,
    line: u32,
    file_name: String,
}

pub struct TestResult {
    case: TestCase,
    error: FailedAssert,
}

pub struct Runner {
    data: LoadSegment,
    stack_pointer: u32,
    entry_point: u32,
    run_test_addr: u64,
    test_done_addr: u64,
    symbols: HashMap<String, u64>,
    tests: Vec<TestCase>,
    connection: Connection,
}

impl Runner {
    pub fn new(binary: &TestBinary, vector_table_addr: u64, connection: Connection) -> Result<Self, String> {
        let segments_to_load = LoadSegment::get_from_file(binary);
        let segment = LoadSegment::collapse_segments(segments_to_load);
        let segment = segment.ok_or(format!("Binary does not contain a loadable segment."))?;
        if vector_table_addr < segment.addr || vector_table_addr + 8 > segment.end_addr() {
            return Err(format!("Vector table not in binary."));
        }
        let addr = (vector_table_addr - segment.addr) as usize;
        let stack_pointer = LittleEndian::read_u32(&segment.data[addr..addr + 4]);
        let entry_point = LittleEndian::read_u32(&segment.data[addr + 4..addr + 8]);

        let mut symbols = HashMap::new();
        for symbol in binary.file.symbols() {
            symbols.insert(symbol.name().unwrap().to_string(), symbol.address());
        }

        let run_test_addr = Self::retrieve_symbol(&symbols, "target_test_fun_to_run")?;
        let test_done_addr = Self::retrieve_symbol(&symbols, "target_test_done")?;

        Ok(Self {
            data: segment,
            stack_pointer,
            entry_point,
            symbols,
            run_test_addr,
            tests: Self::enumerate_tests(binary),
            test_done_addr,
            connection
        })
    }

    fn retrieve_symbol(symbols: &HashMap<String, u64>, name: &str) -> Result<u64, String> {
        match symbols.get(name) {
            None => return Err(format!("Did not find test runner in binary (symbol `{}` missing). Did you link it?", name)),
            Some(x) => Ok(*x),
        }
    }

    pub fn enumerate_tests(binary: &TestBinary) -> Vec<TestCase> {
        let test_re = Regex::new(r"^target_test_test_(?P<suite_name>.*?)__target_test__(?P<test_name>.*?)$").unwrap();
        let mut tests = Vec::new();
        for symbol in binary.file.symbols() {
            let name = symbol.name().unwrap();
            if let Some(captures) = test_re.captures(name) {
                let suite_name = captures.name("suite_name").unwrap().as_str().to_string();
                let test_name = captures.name("test_name").unwrap().as_str().to_string();
                tests.push(TestCase {
                    suite_name,
                    test_name,
                    addr: symbol.address(),
                })
            }
        }
        tests
    }

    pub fn reset(&mut self) -> Result<(), String> {
        jlink::halt()?;
        jlink::reset_device()?;
        sleep(Duration::from_millis(50));
        jlink::set_stack_pointer_and_program_counter(self.stack_pointer, self.entry_point)?;
        Ok(())
    }

    pub fn run(&mut self) -> Result<(), String> {
        jlink::run()
    }

    pub fn reset_run(&mut self) -> Result<(), String> {
        self.reset()?;
        self.run()
    }

    pub fn download(&mut self) -> Result<(), String> {
        jlink::download(self.data.addr, &self.data.data)
    }

    pub fn run_test(&mut self, test_case: &TestCase) -> Result<TestResult, String> {
        jlink::clear_all_breakpoints()?;
        jlink::set_breakpoint(0, self.run_test_addr)?;
        self.reset_run()?;
        self.wait_for_target_halted(Duration::from_millis(200))?;


        todo!()
    }

    pub fn wait_for_target_halted(&mut self, timeout: Duration) -> Result<(), String> {
        let now = Instant::now();
        while now.elapsed().as_millis() < timeout.as_millis() {
            sleep(Duration::from_millis(10));
            if jlink::is_target_halted()? {
                return Ok(())
            }
        }
        Err(format!("Timeout while waiting for target to halt"))
    }
}


