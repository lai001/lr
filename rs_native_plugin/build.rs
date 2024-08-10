fn main() {
    #[cfg(feature = "plugin_shared_lib")]
    let search_dir_name = "";
    #[cfg(feature = "plugin_shared_crate_import")]
    let search_dir_name = "deps";

    #[cfg(any(feature = "plugin_shared_lib", feature = "plugin_shared_crate_import"))]
    {
        #[cfg(debug_assertions)]
        let build_type = "debug";
        #[cfg(not(debug_assertions))]
        let build_type = "release";
        use rs_core_minimal::path_ext::CanonicalizeSlashExt;
        use std::path::Path;
        let search_dir = Path::new(file!())
            .canonicalize_slash()
            .map(|x| {
                x.join(format!(
                    "../../rs_editor/target/{}/{}",
                    build_type, search_dir_name
                ))
            })
            .map(|x| x.canonicalize_slash())
            .unwrap()
            .unwrap();
        let search_dir = search_dir.to_str().unwrap();
        println!("cargo:rustc-link-search={}", search_dir);
    }
}
