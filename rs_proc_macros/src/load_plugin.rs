use proc_macro::TokenStream;
use quote::quote;
use std::path::Path;
use toml_edit::DocumentMut;

fn is_plugin(doc: &DocumentMut) -> bool {
    let is_plugin = doc
        .get("package")
        .and_then(|pkg| pkg.as_table_like())
        .and_then(|pkg_table| pkg_table.get("metadata"))
        .and_then(|meta| meta.as_table_like())
        .and_then(|meta_table| meta_table.get("lr"))
        .and_then(|lr| lr.as_table_like())
        .and_then(|lr_table| lr_table.get("is_plugin"))
        .and_then(|item| item.as_bool());
    is_plugin.unwrap_or(false)
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
                let Ok(Ok(doc)) =
                    std::fs::read_to_string(&crate_manifest_path).map(|x| x.parse::<DocumentMut>())
                else {
                    continue;
                };
                let is_plugin = is_plugin(&doc);
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

#[cfg(test)]
mod test {
    use crate::load_plugin::is_plugin;
    #[test]
    fn test_case() {
        let contents = r#"[package.metadata.lr]
is_plugin = true"#;
        let doc = contents.parse::<toml_edit::DocumentMut>().unwrap();
        assert!(is_plugin(&doc));
    }

    #[test]
    fn test_case1() {
        let contents = r#"[package.metadata]
is_plugin = true"#;
        let doc = contents.parse::<toml_edit::DocumentMut>().unwrap();
        assert_eq!(is_plugin(&doc), false);
    }

    #[test]
    fn test_case2() {
        let contents = r#"[package.metadata]
is_plugin = false"#;
        let doc = contents.parse::<toml_edit::DocumentMut>().unwrap();
        assert_eq!(is_plugin(&doc), false);
    }

    #[test]
    fn test_case3() {
        let contents = r#"[package.metadata.lr]
is_plugin = 1"#;
        let doc = contents.parse::<toml_edit::DocumentMut>().unwrap();
        assert_eq!(is_plugin(&doc), false);
    }

    #[test]
    fn test_case4() {
        let contents = r#"[package]
metadata = { lr = { is_plugin = true } }"#;
        let doc = contents.parse::<toml_edit::DocumentMut>().unwrap();
        assert_eq!(is_plugin(&doc), true);
    }
}
