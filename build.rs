use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo:rustc-link-search=/usr/lib/x86_64-linux-gnu");
    println!("cargo:rustc-link-lib=ibumad");
    println!("cargo:rustc-link-lib=ibmad");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());

    //UMAD
    let umad_bindings = bindgen::Builder::default()
        .header("src/umad/wrapper.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Unable to generate bindings");

    umad_bindings
        .write_to_file(out_path.join("umad_bindings.rs"))
        .expect("Couldn't write umad bindings!");

    //IBMAD
    let ibmad_bindings = bindgen::Builder::default()
        .header("src/ibmad/wrapper.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Unable to generate bindings");

    ibmad_bindings
        .write_to_file(out_path.join("ibmad_bindings.rs"))
        .expect("Couldn't write ibmad bindings!");

}