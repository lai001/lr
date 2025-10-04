use ra_ap_hir::FindPathConfig;
use ra_ap_ide::*;
use ra_ap_ide_db::*;
use ra_ap_syntax::*;
use ra_ap_vfs::*;
use std::path::PathBuf;

pub fn get_cargo_toml(manifest_dir: &PathBuf) -> AbsPathBuf {
    AbsPathBuf::try_from(manifest_dir.join("Cargo.toml").to_str().unwrap()).unwrap()
}

pub fn find_use_path_helper(
    db: &RootDatabase,
    module: &ra_ap_hir::Module,
    target_file_id: EditionedFileId,
) -> Option<ra_ap_hir::ModPath> {
    let sema = Semantics::new(db);
    let source_file: SourceFile = sema.parse(target_file_id);
    let target_module = sema
        .scope(source_file.syntax())?
        .module()
        .nearest_non_block_module(db);

    let use_path = module.find_use_path(
        db,
        ra_ap_hir::ItemInNs::Types(ra_ap_hir::ModuleDef::Module(target_module)),
        ra_ap_hir::PrefixKind::Plain,
        find_path_config(),
    );
    return use_path;
}

pub fn find_path_config() -> FindPathConfig {
    FindPathConfig {
        prefer_no_std: true,
        prefer_prelude: false,
        prefer_absolute: true,
        allow_unstable: false,
    }
}

pub fn struct_field_meta_token_stream(name: &str, ty: &str) -> proc_macro2::TokenStream {
    let output_stream = quote::quote! {
        StructFieldMeta {
            name: #name.to_string(),
            type_meta: TypeMeta {
                name: #ty.to_string(),
            }
        },
    };
    output_stream.into()
}

pub fn param_meta_token_stream(name: Option<&str>, ty: &str) -> proc_macro2::TokenStream {
    let name_token_stream = match name {
        Some(name) => {
            quote::quote! {
               name: Some(#name.to_string()),
            }
        }
        None => {
            quote::quote! {
               name: None,
            }
        }
    };
    let output_stream = quote::quote! {
        ParamMeta {
            #name_token_stream
            type_meta: TypeMeta {
                name: #ty.to_string(),
            }
        },
    };
    output_stream.into()
}

pub fn reflection_token_stream_template(
    reflect_ty_name: &str,
    fields_token_stream: proc_macro2::TokenStream,
    functions_token_stream: proc_macro2::TokenStream,
    exec_token_stream: proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    let struct_name = quote::format_ident!("{}Reflection", reflect_ty_name);
    quote::quote! {
        pub struct #struct_name {
            pub struct_meta: StructMeta,
        }
        impl #struct_name {
            pub fn new() -> #struct_name {
                let struct_meta = StructMeta {
                    name: #reflect_ty_name.to_string(),
                    fields: vec![
                        #fields_token_stream
                    ],
                    functions: vec![
                        #functions_token_stream
                    ]
                };
                #struct_name { struct_meta }
            }
        }
        impl StructMetaContainer for #struct_name {
            fn get_struct_meta(&self) -> &StructMeta {
                &self.struct_meta
            }
        }
        #exec_token_stream
    }
}
