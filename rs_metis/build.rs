use std::env;
use std::path::PathBuf;

use rs_core_minimal::path_ext::CanonicalizeSlashExt;

fn is_debug() -> bool {
    if Ok("release".to_owned()) == env::var("PROFILE") {
        false
    } else {
        true
    }
}

fn main() {
    let target = env::var("TARGET").expect("TARGET environment variable is not set");

    let mut include_dirs: Vec<String> = Vec::new();
    let engine_root_dir = rs_core_minimal::file_manager::get_engine_root_dir()
        .canonicalize_slash()
        .expect("The folder should exist");
    include_dirs.push(
        engine_root_dir
            .join(".xmake/deps/METIS/include")
            .to_string_lossy()
            .to_string(),
    );
    include_dirs.push(
        engine_root_dir
            .join(".xmake/deps/GKlib")
            .to_string_lossy()
            .to_string(),
    );

    let mut defines: Vec<String> = Vec::new();
    defines.push("USE_GKREGEX".to_owned());
    defines.push("IDXTYPEWIDTH=32".to_owned());
    defines.push("REALTYPEWIDTH=32".to_owned());

    if target.contains("windows") {
        defines.push("__thread=__declspec(thread)".to_owned());
        defines.push("MSC".to_owned());
        defines.push("WIN32".to_owned());
        defines.push("_CRT_SECURE_NO_DEPRECATE".to_owned());
    }

    if target.contains("android") {
        defines.push("__thread=".to_owned());
    }

    if is_debug() {
        println!(
            "cargo:rustc-link-search={}",
            engine_root_dir
                .join("build/windows/x64/debug")
                .to_string_lossy()
                .to_string()
        );
        defines.push("DEBUG".to_owned());
    } else {
        println!(
            "cargo:rustc-link-search={}",
            engine_root_dir
                .join("build/windows/x64/release")
                .to_string_lossy()
                .to_string()
        );
        defines.push("NDEBUG".to_owned());
    }

    println!("cargo:rustc-link-lib=metis");
    println!("cargo:rustc-link-lib=GKlib");
    let bindings = bindgen::Builder::default()
        .header(
            engine_root_dir
                .join(".xmake/deps/METIS/include/metis.h")
                .to_string_lossy()
                .to_string(),
        )
        .header(
            engine_root_dir
                .join(".xmake/deps/METIS/libmetis/metislib.h")
                .to_string_lossy()
                .to_string(),
        )
        .header(
            engine_root_dir
                .join(".xmake/deps/METIS/libmetis/rename.h")
                .to_string_lossy()
                .to_string(),
        )
        .header(
            engine_root_dir
                .join(".xmake/deps/METIS/programs/struct.h")
                .to_string_lossy()
                .to_string(),
        )
        .allowlist_function("METIS_.*")
        .allowlist_function("libmetis__CreateGraph")
        .allowlist_function("libmetis__FreeGraph")
        .allowlist_function("libmetis__InitGraph")
        .allowlist_function("libmetis__imalloc")
        .allowlist_function("libmetis__ismalloc")
        .allowlist_function("gk_free")
        .allowlist_function("libmetis__rsmalloc")
        .allowlist_type("idx_t")
        .allowlist_type("real_t")
        .allowlist_type("graph_t")
        .allowlist_type("params_t")
        .allowlist_type("rstatus_et")
        .allowlist_type("m.*_et")
        .allowlist_var("METIS_.*")
        .allowlist_var("KMETIS_DEFAULT_UFACTOR")
        .allowlist_var("MCPMETIS_DEFAULT_UFACTOR")
        .allowlist_var("PMETIS_DEFAULT_UFACTOR")
        .clang_args(
            defines
                .iter()
                .map(|x| format!("-D {}", x))
                .collect::<Vec<String>>(),
        )
        .clang_args(
            include_dirs
                .iter()
                .map(|x| format!("-I{}", x))
                .collect::<Vec<String>>(),
        )
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
