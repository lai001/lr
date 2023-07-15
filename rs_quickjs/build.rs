use std::env;
use std::path::PathBuf;

fn main() {
    if Ok("release".to_owned()) == env::var("PROFILE") {
        println!("cargo:rustc-link-search=../build/windows/x64/release");
    } else {
        println!("cargo:rustc-link-search=../build/windows/x64/debug");
    }
    println!("cargo:rustc-link-lib=quickjs");
    let bindings = bindgen::Builder::default()
        .header("../.xmake/deps/quickjs/quickjs.h")
        .header("../.xmake/deps/quickjs/quickjs-libc.h")
        .header("./src/QuickjsHelper.h")
        .clang_arg("-I../.xmake/deps/quickjs")
        .clang_arg("-D CONFIG_BIGNUM")
        .clang_arg("-D JS_STRICT_NAN_BOXING")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
