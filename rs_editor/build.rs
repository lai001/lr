fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap() == "windows" {
        let input = rs_core_minimal::file_manager::get_engine_resource("Editor/icon.svg");
        let output = rs_core_minimal::file_manager::get_engine_root_dir()
            .join("rs_editor/target/editor.ico");
        svg_to_ico::svg_to_ico(&input, 96.0, &output, &[256, 256])
            .expect("failed to convert svg to ico");
        let mut res = winresource::WindowsResource::new();
        res.set_icon(output.to_str().unwrap());
        res.compile().unwrap();
    }
}
