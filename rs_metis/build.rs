use std::env;
use std::path::PathBuf;

fn is_debug() -> bool {
    if Ok("release".to_owned()) == env::var("PROFILE") {
        false
    } else {
        true
    }
}

fn main() {
    let mut include_dirs: Vec<String> = Vec::new();
    include_dirs.push("../.xmake/deps/METIS/include".to_owned());
    include_dirs.push("../.xmake/deps/GKlib".to_owned());

    let mut defines: Vec<String> = Vec::new();
    defines.push("USE_GKREGEX".to_owned());
    defines.push("IDXTYPEWIDTH=32".to_owned());
    defines.push("REALTYPEWIDTH=32".to_owned());

    #[cfg(target_os = "windows")]
    {
        defines.push("__thread=__declspec(thread)".to_owned());
        defines.push("MSC".to_owned());
        defines.push("WIN32".to_owned());
        defines.push("_CRT_SECURE_NO_DEPRECATE".to_owned());
    }

    if is_debug() {
        println!("cargo:rustc-link-search=../build/windows/x64/debug");
        defines.push("DEBUG".to_owned());
    } else {
        println!("cargo:rustc-link-search=../build/windows/x64/release");
        defines.push("NDEBUG".to_owned());
    }

    println!("cargo:rustc-link-lib=metis");
    let bindings = bindgen::Builder::default()
        .header("../.xmake/deps/METIS/include/metis.h")
        .header("../.xmake/deps/METIS/libmetis/metislib.h")
        .allowlist_function("METIS_.*")
        .allowlist_type("idx_t")
        .allowlist_type("real_t")
        .allowlist_type("graph_t")
        .allowlist_type("rstatus_et")
        .allowlist_type("m.*_et")
        .allowlist_var("METIS_.*")
        .clang_args(defines.iter().map(|x| format!("-D {}", x)).collect::<Vec<String>>())
        .clang_args(include_dirs.iter().map(|x| format!("-I{}", x)).collect::<Vec<String>>())
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
