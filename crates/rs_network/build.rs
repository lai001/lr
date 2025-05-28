use std::env;
use std::path::PathBuf;

fn main() {
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").expect("A valid environment variable");

    if target_os == "windows" {
        if Ok("release".to_owned()) == env::var("PROFILE") {
            println!("cargo:rustc-link-search=../../build/windows/x64/release");
        } else {
            println!("cargo:rustc-link-search=../../build/windows/x64/debug");
        }
    } else if target_os == "android" {
        if Ok("release".to_owned()) == env::var("PROFILE") {
            println!("cargo:rustc-link-search=../../build/android/x64/release");
        } else {
            println!("cargo:rustc-link-search=../../build/android/x64/debug");
        }
    } else {
        panic!("Not support");
    }

    println!("cargo:rustc-link-lib=kcp");
    let bindings = bindgen::Builder::default()
        .header("../../.xmake/deps/kcp/ikcp.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
