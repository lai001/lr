fn main() {
    #[cfg(feature = "plugin_shared_lib")]
    {
        use rs_core_minimal::path_ext::CanonicalizeSlashExt;
        use std::path::Path;
        let search_dir = Path::new(file!())
            .canonicalize_slash()
            .map(|x| x.join("../../rs_editor/target/debug"))
            .map(|x| x.canonicalize_slash())
            .unwrap()
            .unwrap();
        let search_dir = search_dir.to_str().unwrap();
        println!("cargo:rustc-link-search={}", search_dir);
    }
}
