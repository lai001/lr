use rs_build_util::copy_to_output;
use std::env;
use std::fs;

fn main() {
    if env::var("CARGO_CFG_TARGET_OS").unwrap() == "windows" {
        let dll_dir = rs_core_minimal::file_manager::get_deps_dir()
            .join("ffmpeg-n7.1.4-win64-gpl-shared-7.1/bin")
            .canonicalize()
            .unwrap();
        for entry in fs::read_dir(dll_dir).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("dll") {
                copy_to_output(&path);
            }
        }
    }
}
