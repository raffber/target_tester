use std::fs::File;
use clap::{app_from_crate, arg};
use object::{Object, ObjectSymbol, ObjectSymbolTable};
use std::process::exit;
use target_tester::config::{Interface, Speed};
use target_tester::{Connection, LoadSegment, Runner, TestBinary, TestCase};
use target_tester::report::xml_dump_result;

fn main() {
    env_logger::init();

    let matches = app_from_crate!()
        .arg(arg!(-c --config <CONFIG> "Config file"))
        .arg(arg!(<BINARY> "Binary file used for testing"))
        .get_matches();
    let binary = matches.value_of("BINARY").unwrap();
    let config = matches.value_of("config").unwrap();

    let binary = match std::fs::read(binary) {
        Ok(x) => x,
        Err(err) => {
            println!("Cannot read binary: {}", err);
            exit(1);
        }
    };
    let file = target_tester::ElfFile::parse(binary.as_slice()).expect("Could not read elf-file");
    let binary = TestBinary::new(file);

    let connection = Connection::connect("S32K148", Speed::KHz(4000), Interface::SWD).unwrap();

    let tests = Runner::enumerate_tests(&binary);
    let mut runner = Runner::new(&binary, 0x10028, connection).unwrap();


    println!("Downloading test binary....");
    runner.download().unwrap();
    println!("done\n");

    let results= runner.run_all_tests().unwrap();

    let mut file = File::create("my-junit.xml").unwrap();
    xml_dump_result(results, file).unwrap();
}
