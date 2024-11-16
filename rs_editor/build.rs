fn copy_std_lib() {
    let Ok(rustc_toolchain) = std::env::var("RUSTC_TOOLCHAIN") else {
        return;
    };
    let Ok(out_dir) = std::env::var("OUT_DIR") else {
        return;
    };

    let lib_folder =
        std::path::Path::new(&rustc_toolchain).join("lib/rustlib/x86_64-pc-windows-msvc/lib");
    let mut paths = glob::glob(&format!("{}/std-*.dll", lib_folder.display().to_string())).unwrap();
    let path = paths.find(|_| true).unwrap().unwrap();
    let filename = path.file_name().unwrap().to_str().unwrap();
    let to = std::path::Path::new(&out_dir)
        .join("../../../")
        .join(filename);
    if to.exists() {
        return;
    }
    std::fs::copy(&path, &to).unwrap();
}

fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap() == "windows" {
        let input = rs_core_minimal::file_manager::get_engine_resource("Editor/icon.svg");
        let output = rs_core_minimal::file_manager::get_engine_output_target_dir()
            .join("editor.ico");
        svg_to_ico::svg_to_ico(&input, 96.0, &output, &[256, 256])
            .expect("failed to convert svg to ico");
        let mut res = winresource::WindowsResource::new();
        res.set_icon(output.to_str().unwrap());
        res.compile().unwrap();

        #[cfg(feature = "plugin_shared_crate")]
        {
            copy_std_lib();
        }
    }
}
