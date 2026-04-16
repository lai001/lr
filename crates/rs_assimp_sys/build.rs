use rs_core_minimal::path_ext::CanonicalizeSlashExt;
use std::{env, ffi::OsStr, path::PathBuf};

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    let engine_root_dir = rs_core_minimal::file_manager::get_engine_root_dir()
        .canonicalize_slash()
        .expect("The folder should exist");

    let assimp_root_dir = engine_root_dir.join(".xmake/deps/assimp");

    let headers = vec![
        "assimp/aabb.h",
        "assimp/anim.h",
        "assimp/camera.h",
        "assimp/cexport.h",
        "assimp/cfileio.h",
        "assimp/cimport.h",
        "assimp/color4.h",
        "assimp/importerdesc.h",
        "assimp/light.h",
        "assimp/material.h",
        "assimp/matrix3x3.h",
        "assimp/matrix4x4.h",
        "assimp/mesh.h",
        "assimp/metadata.h",
        "assimp/postprocess.h",
        "assimp/scene.h",
        "assimp/texture.h",
        "assimp/types.h",
        "assimp/vector2.h",
        "assimp/vector3.h",
        "assimp/version.h",
    ];
    let headers = headers.iter().map(|x| {
        assimp_root_dir
            .join("build/include")
            .join(x)
            .to_string_lossy()
            .to_string()
    });

    bindgen::builder()
        .clang_arg(format!(
            "-I{}",
            assimp_root_dir.join("build/include").display()
        ))
        .headers(headers)
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .allowlist_type("ai.*")
        .allowlist_function("ai.*")
        .allowlist_var("ai.*")
        .allowlist_var("AI_.*")
        .derive_partialeq(true)
        .derive_eq(true)
        .derive_hash(true)
        .derive_debug(true)
        .generate()
        .unwrap()
        .write_to_file(out_dir.join("bindings.rs"))
        .unwrap();

    println!(
        "cargo:rustc-link-search=native={}",
        assimp_root_dir.join("build/lib").display()
    );

    for entry in walkdir::WalkDir::new(assimp_root_dir.join("build/lib")).max_depth(1) {
        if let Ok(entry) = entry {
            if entry.path().extension() == Some(OsStr::new("lib")) {
                let name = entry
                    .path()
                    .file_stem()
                    .map(|x| x.to_str().unwrap())
                    .unwrap();
                println!("cargo:rustc-link-lib={}={}", "static", name);
            }
        }
    }
}
