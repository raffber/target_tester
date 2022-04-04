use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::iter::repeat;
use std::thread::sleep;
use std::time::{Duration, Instant, SystemTime};

use byteorder::{ByteOrder, LittleEndian};
use object::{Object, ObjectSymbol};
use object::elf::PT_LOAD;
use object::Endianness;
use regex::Regex;

use crate::connection::Connection;
use crate::crc::crc32;
use crate::ElfFile;

pub struct TestBinary<'data> {
    pub file: ElfFile<'data>,
}

impl<'data> TestBinary<'data> {
    pub fn new(file: ElfFile<'data>) -> Self {
        Self { file }
    }
}

pub struct LoadSegment {
    addr: u32,
    data: Vec<u8>,
}

impl Debug for LoadSegment {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "LoadSegment(0x{:x}, 0x{:x})",
            self.addr,
            self.data.len()
        ))
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
                addr: physical_address,
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

    pub fn start_addr(&self) -> u32 {
        self.addr
    }

    pub fn end_addr(&self) -> u32 {
        self.addr + (self.data.len() as u32)
    }
}

#[derive(Clone)]
pub struct TestCase {
    pub suite_name: String,
    pub test_name: String,
    pub addr: u32,
}

impl TestCase {
    pub fn symbol_name(&self) -> String {
        format!(
            "__test_{}__target_test__{}",
            self.suite_name, self.test_name
        )
    }
}

#[derive(Clone)]
pub struct FailedAssert {
    pub lineno: Option<u32>,
    pub file_name: Option<String>,
    pub assertion: Option<TargetAssertion>,
}

#[derive(Clone)]
pub struct TestResult {
    pub case: TestCase,
    pub timestamp: SystemTime,
    pub error: Option<FailedAssert>,
}

enum TargetState {
    Idle,
    Ready,
    Started,
    Passed,
    Failed,
}

#[derive(Clone)]
pub enum TargetAssertion {
    Equal,
    True,
    False,
    Other(u32),
}

struct TargetData {
    state: Option<TargetState>,
    executed_function: Option<u32>,
    fail_reason: Option<TargetAssertion>,
    file_path: Option<String>,
    lineno: Option<u32>,
}

impl TargetData {
    fn crc32(data: &[u8]) -> u32 {
        crc32(data)
    }

    fn fetch(connection: &mut impl Connection, addr: u32) -> Result<TargetData, String> {
        let data = connection.read_ram(addr, 6 * 4)?;
        let ref_crc = Self::crc32(&data[0..data.len() - 4]);
        let crc = LittleEndian::read_u32(&data[20..data.len()]);
        if crc != ref_crc {
            return Err(format!("Connection failed to read from RAM: Invalid CRC"));
        }
        let state = LittleEndian::read_u32(&data[0..4]);
        let executed_function = LittleEndian::read_u32(&data[4..8]);
        let fail_reason = LittleEndian::read_u32(&data[8..12]);
        let fail_reason = match fail_reason {
            0 => None,
            1 => Some(TargetAssertion::Equal),
            2 => Some(TargetAssertion::True),
            3 => Some(TargetAssertion::False),
            x => Some(TargetAssertion::Other(x)),
        };

        let state = match state {
            0 => None,
            0x8C3F82FA => Some(TargetState::Idle),
            0xD79A2E5F => Some(TargetState::Ready),
            0xCD833CB7 => Some(TargetState::Started),
            0xBAF2C481 => Some(TargetState::Passed),
            0xCA83D14E => Some(TargetState::Failed),
            x => { return Err(format!("Invalid data on target: State was: {}", x)); }
        };


        let file_path = LittleEndian::read_u32(&data[12..16]);
        let lineno = LittleEndian::read_u32(&data[16..20]);
        let file_path = if file_path != 0 {
            Some(connection.read_utf8_string(file_path)?)
        } else {
            None
        };

        Ok(TargetData {
            state,
            executed_function: if executed_function != 0 { Some(executed_function) } else { None },
            fail_reason,
            file_path,
            lineno: if lineno != 0 { Some(lineno) } else { None },
        })
    }
}

pub struct Runner<T: Connection> {
    data: LoadSegment,
    stack_pointer: u32,
    entry_point: u32,
    run_test_addr: u32,
    test_data_addr: u32,
    tests: Vec<TestCase>,
    connection: T,
}

impl<T: Connection> Runner<T> {
    pub fn new(
        binary: &TestBinary,
        vector_table_addr: u32,
        connection: T,
    ) -> Result<Self, String> {
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
            symbols.insert(symbol.name().unwrap().to_string(), symbol.address() as u32);
        }

        let run_test_addr = Self::retrieve_symbol(&symbols, "target_test_fun_to_run")?;
        let test_data_addr = Self::retrieve_symbol(&symbols, "target_test_data")?;

        Ok(Self {
            data: segment,
            stack_pointer,
            entry_point,
            run_test_addr,
            tests: Self::enumerate_tests(binary),
            test_data_addr,
            connection,
        })
    }

    fn retrieve_symbol(symbols: &HashMap<String, u32>, name: &str) -> Result<u32, String> {
        match symbols.get(name) {
            None => {
                return Err(format!(
                    "Did not find test runner in binary (symbol `{}` missing). Did you link it?",
                    name
                ));
            }
            Some(x) => Ok(*x),
        }
    }

    fn enumerate_tests(binary: &TestBinary) -> Vec<TestCase> {
        let test_re =
            Regex::new(r"^target_test_test_(?P<suite_name>.*?)__target_test__(?P<test_name>.*?)$")
                .unwrap();
        let mut tests = Vec::new();
        for symbol in binary.file.symbols() {
            let name = symbol.name().unwrap();
            if let Some(captures) = test_re.captures(name) {
                let suite_name = captures.name("suite_name").unwrap().as_str().to_string();
                let test_name = captures.name("test_name").unwrap().as_str().to_string();
                tests.push(TestCase {
                    suite_name,
                    test_name,
                    addr: symbol.address() as u32,
                })
            }
        }
        tests
    }

    pub fn download(&mut self) -> Result<(), String> {
        self.connection.download(self.data.addr, &self.data.data)
    }

    pub fn run_test(&mut self, test_case: &TestCase) -> Result<TestResult, String> {
        log::debug!("Running test: {} @ {:X}", test_case.test_name, test_case.addr);
        self.connection.reset_run(self.stack_pointer, self.entry_point)?;
        log::debug!("Waiting for device to boot and enter test framework");
        self.wait_for_ready(Duration::from_millis(500))?;

        log::debug!("Halting device again to write test function pointer");
        self.connection.halt(Duration::from_millis(100))?;
        let mut fun_ptr = [0_u8; 4];
        LittleEndian::write_u32(&mut fun_ptr, test_case.addr as u32);
        self.connection.write_ram(self.run_test_addr as u32, &fun_ptr)?;
        self.connection.run()?;

        let data = self.wait_for_done(Duration::from_millis(500))?;
        let test_passed = matches!( &data.state, Some(TargetState::Passed));

        if let Some(executed_function) = data.executed_function {
            if executed_function != test_case.addr {
                return Err(format!("Test framework executed wrong test function. Expected function @{:X} but executed @{:X}", test_case.addr, executed_function));
            }
        } else {
            return Err(format!("Test framework did not execute a test function. Expected function @{:X}", test_case.addr));
        }

        let error = if !test_passed {
            Some(FailedAssert {
                lineno: data.lineno,
                file_name: data.file_path,
                assertion: data.fail_reason,
            })
        } else {
            None
        };

        Ok(TestResult {
            case: test_case.clone(),
            timestamp: SystemTime::now(),
            error,
        })
    }

    pub fn run_all_tests(&mut self) -> Result<Vec<TestResult>, String> {
        let mut ret = Vec::new();
        let tests = self.tests.clone();
        for test in tests {
            println!("Running test: {} -- {}", test.suite_name, test.test_name);
            let result = self.run_test(&test)?;
            if let Some(error) = &result.error {
                let file_name = match &error.file_name {
                    None => "".to_string(),
                    Some(x) => x.to_string()
                };
                println!("Test failed at: {}:{}", file_name, error.lineno.unwrap_or_default());
            } else {
                println!("Test Passed\n");
            }
            ret.push(result);
        }
        Ok(ret)
    }

    fn wait_for_done(&mut self, timeout: Duration) -> Result<TargetData, String> {
        let now = Instant::now();
        while now.elapsed().as_millis() < timeout.as_millis() {
            sleep(Duration::from_millis(10));
            let addr = self.test_data_addr;
            let data = TargetData::fetch(&mut self.connection, addr)?;
            let done = match data.state {
                Some(TargetState::Passed) | Some(TargetState::Failed) => true,
                _ => false,
            };
            if done {
                return Ok(data);
            }
        }
        Err(format!("Timeout while waiting for test to finish"))
    }

    pub fn wait_for_ready(&mut self, timeout: Duration) -> Result<(), String> {
        let now = Instant::now();
        while now.elapsed().as_millis() < timeout.as_millis() {
            sleep(Duration::from_millis(10));
            let addr = self.test_data_addr;
            let data = self.connection.read_ram(addr, 6 * 4)?;
            let crc = LittleEndian::read_u32(&data[20..data.len()]);
            if crc == 0 {
                continue;
            }

            let data = TargetData::fetch(&mut self.connection, addr)?;
            let done = match data.state {
                Some(TargetState::Ready) => true,
                _ => false,
            };
            if done {
                return Ok(());
            }
        }
        Err(format!("Timeout while waiting for test suite to start up"))
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_crc() {
        let crc = TargetData::crc32(&[0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39]);
        assert_eq!(crc, 0xCBF43926);
    }
}