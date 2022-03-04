use std::env::current_dir;

fn main() {
    println!("cargo:rerun-if-changed=3rdparty/");
    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .dynamic_library_name("JLink")
        .generate()
        .expect("Unable to generate bindings");
    let cwd = current_dir().unwrap();
    let out_path = cwd.join("src").join("jlink_sys.rs");
    bindings
        .write_to_file(out_path)
        .expect("Couldn't write bindings!");
}
