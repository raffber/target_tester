use std::fs;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::process::exit;

use clap::{app_from_crate, arg};

use target_tester::config::Config;
use target_tester::probers::ProbeRsConnection;
use target_tester::report::xml_dump_result;
use target_tester::runner::{Runner, TestBinary};

fn main() {
    env_logger::init();

    let matches = app_from_crate!()
        .arg(arg!(-c --config <CONFIG> "Config file"))
        .arg(arg!(-o --output [OUTPUT] "Output file"))
        .arg(arg!(<BINARY> "Binary file used for testing"))
        .get_matches();
    let binary = matches.value_of("BINARY").unwrap();
    let config_path: PathBuf = matches.value_of("config").unwrap().into();
    let config_path = fs::canonicalize(config_path).unwrap();
    let config_dir = config_path.parent().unwrap();

    let mut config_data = String::new();
    let mut config_file = File::open(&config_path).expect(&format!("Could not open config file: {:?}", config_path));
    config_file.read_to_string(&mut config_data).expect(&format!("Could not read from output file: {:?}", config_path));
    let config = match serde_json::from_str::<Config>(&config_data) {
        Ok(x) => x,
        Err(err) => panic!("Invalid config file: {}", err)
    };

    let binary = match std::fs::read(binary) {
        Ok(x) => x,
        Err(err) => {
            println!("Cannot read binary: {}", err);
            exit(1);
        }
    };
    let file = target_tester::ElfFile::parse(binary.as_slice()).expect("Could not read elf-file");
    let binary = TestBinary::new(file);


    if let Some(target_description) = config.target_description {
        let target_description_file = config_dir.join(target_description);
        probe_rs::config::add_target_from_yaml(&target_description_file).unwrap();
    }

    let connection = ProbeRsConnection::connect(&config.target, config.interface, config.speed).unwrap();
    let mut runner = Runner::new(&binary, config.vector_table_offset, connection).unwrap();


    println!("Downloading test binary....");
    runner.download().unwrap();
    println!("done\n");

    let results = runner.run_all_tests().unwrap();

    if let Some(output) = matches.value_of("output") {
        let file = File::create(output).expect(&format!("Could not write to output file: {}", output));
        xml_dump_result(results, file).unwrap();
    }
}
