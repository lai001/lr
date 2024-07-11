fn main() {
    use rs_core_minimal::path_ext::CanonicalizeSlashExt;
    use std::path::Path;
    let search_dir = Path::new(file!())
        .canonicalize_slash()
        .map(|x| x.join("../../.xmake/deps/dotnetSDK/host/fxr/8.0.6"))
        .map(|x| x.canonicalize_slash())
        .unwrap()
        .unwrap();
    let search_dir = search_dir.to_str().unwrap();
    println!("cargo:rustc-link-search={}", search_dir);

    let search_dir = Path::new(file!())
        .canonicalize_slash()
        .map(|x| x.join("../../.xmake/deps/dotnetSDK/packs/Microsoft.NETCore.App.Host.win-x64/8.0.6/runtimes/win-x64/native"))
        .map(|x| x.canonicalize_slash())
        .unwrap()
        .unwrap();
    let search_dir = search_dir.to_str().unwrap();
    println!("cargo:rustc-link-search={}", search_dir);
}
