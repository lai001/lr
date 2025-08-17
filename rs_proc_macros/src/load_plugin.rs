use anyhow::anyhow;
use proc_macro::TokenStream;
use quote::quote;
use std::path::Path;
use toml_edit::DocumentMut;

fn is_plugin(crate_manifest_path: &Path) -> anyhow::Result<bool> {
    let contents = std::fs::read_to_string(crate_manifest_path)?;
    let doc = contents.parse::<DocumentMut>()?;
    let (_, value) = doc
        .get_key_value("features")
        .ok_or(anyhow!("No features"))?;
    let table = value.as_table().ok_or(anyhow!(""))?;
    Ok(table.contains_key("lr_plugin"))
}

pub fn load_static_plugins_macro_impl(input: TokenStream) -> TokenStream {
    let mut create_plugins = String::new();
    let program = input.to_string();
    match program.as_str() {
        "rs_editor" | "rs_desktop_standalone" | "rs_android" => {
            let manifest_path = Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("..")
                .join(program.as_str())
                .join("Cargo.toml");
            let contents = std::fs::read_to_string(manifest_path).unwrap();
            let doc = contents.parse::<DocumentMut>().unwrap();
            let table = doc["dependencies"].as_table().unwrap();
            for (crate_name, crate_properties) in table.iter() {
                let Some(crate_properties) = crate_properties.as_value() else {
                    continue;
                };
                let Some(inline_table) = crate_properties.as_inline_table() else {
                    continue;
                };
                let Some((_, path)) = inline_table.get_key_value("path") else {
                    continue;
                };
                let Some(Ok(path)) = path.as_str().map(|x| Path::new(x).canonicalize()) else {
                    continue;
                };
                let crate_manifest_path = path.join("Cargo.toml");
                let is_plugin = is_plugin(&crate_manifest_path).unwrap_or(false);
                if is_plugin {
                    let crate_name = quote::format_ident!("{}", crate_name);
                    let stream = quote! { #crate_name::create_plugin(), };
                    create_plugins += &stream.to_string();
                }
            }
        }
        _ => {}
    }
    let stream_str = format!("vec![{}]", create_plugins);
    stream_str.parse::<TokenStream>().unwrap()
}
