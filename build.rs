use std::env;
use std::path::PathBuf;

fn main() {
    let bindings = bindgen::Builder::default()
        // Wrapper for actual header files in the system include path
        .header("wrapper.h")
        // Tell cargo to invalidate the built crate whenever any of the included header files changed
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        // Suppress all those warnings... (TODO: make this actually work)
        .raw_line("#[allow(non_camel_case_types)]")
        // Derive std::default::Default implementations
        .derive_default(true)
        // Generate the bindings
        .generate().expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
