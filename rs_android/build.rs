fn main() {
    let cargo_cfg_target_os =
        std::env::var("CARGO_CFG_TARGET_OS").expect("CARGO_CFG_TARGET_OS environment variable");
    let cargo_cfg_target_arch =
        std::env::var("CARGO_CFG_TARGET_ARCH").expect("CARGO_CFG_TARGET_ARCH environment variable");
    let profile = std::env::var("PROFILE").expect("PROFILE environment variable");
    if cargo_cfg_target_os == "android" && cargo_cfg_target_arch == "x86_64" {
        println!(
            "cargo:rustc-link-search={}",
            rs_core_minimal::file_manager::get_engine_root_dir()
                .join(".xmake/deps/ffmpeg_android/x86_64/lib")
                .to_string_lossy()
        );
        println!(
            "cargo:rustc-link-search={}",
            rs_core_minimal::file_manager::get_engine_root_dir()
                .join("build/android/x86_64")
                .join(profile)
                .to_string_lossy()
        );
        // https://github.com/RustAudio/cpal/issues/859#issuecomment-2757139534
        println!("cargo:rustc-link-lib=c++_shared");
        println!("cargo:rustc-link-arg=-fexceptions");
        println!("cargo:rustc-link-arg=-frtti");
    }
}
