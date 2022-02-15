use std::io::Cursor;
use std::process::exit;
use clap::{app_from_crate, arg, App, ArgMatches};
use object::{Object, ObjectSymbol, ObjectSymbolTable};
use target_tester::{LoadSegment, Runner, TestBinary};


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
    // let file = object::read::File::parse(binary.as_slice())
    let binary = TestBinary::new(file);

    let table = binary.file.symbol_table().unwrap();
    for symbol in table.symbols() {
        // println!("{}", symbol.name().unwrap());
    }

    let segments = LoadSegment::get_from_file(&binary);
    LoadSegment::collapse_segments(segments);

    // let runner = Runner::initialize().unwrap();
    // Runner::upload(&binary);

}
