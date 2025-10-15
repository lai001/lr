use proc_macro2::TokenStream;
use std::{collections::HashMap, path::PathBuf};

pub struct RegisterFunctionMaker {
    type_map: HashMap<PathBuf, Vec<TokenStream>>,
}

impl RegisterFunctionMaker {
    pub fn new() -> Self {
        Self {
            type_map: HashMap::new(),
        }
    }

    pub fn make(&self) -> TokenStream {
        let mut full_types: Vec<TokenStream> = vec![];
        for (prefix, types) in &self.type_map {
            let prefix = prefix
                .components()
                .map(|x| {
                    let mut fix = x.as_os_str().to_string_lossy().to_string();
                    fix = fix.replace("\\", "");
                    fix = fix.replace("/", "");
                    fix = fix.replace(".rs", "");
                    fix
                })
                .collect::<Vec<String>>()
                .join("::");
            let prefix = prefix.strip_prefix("::").unwrap_or(&prefix);
            let prefix = prefix
                .parse::<TokenStream>()
                .expect(&format!("Valid stream, {prefix}"));
            for ty in types {
                full_types.push(quote::quote! {
                    #prefix::#ty
                });
            }
        }
        let mut funciton_body = TokenStream::new();
        for ty in full_types {
            funciton_body.extend(quote::quote! {
                v8_runtime.register_constructor::<#ty>();
            });
        }
        quote::quote! {
            pub fn register(v8_runtime: &mut rs_v8_host::v8_runtime::V8Runtime) {
                #funciton_body
            }
        }
    }

    pub fn type_map_mut(&mut self) -> &mut HashMap<PathBuf, Vec<TokenStream>> {
        &mut self.type_map
    }
}
