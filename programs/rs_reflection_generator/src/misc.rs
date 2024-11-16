use ra_ap_ide::*;
use ra_ap_ide_db::*;
use ra_ap_syntax::*;
use ra_ap_vfs::*;
use std::path::PathBuf;

pub fn get_cargo_toml(manifest_dir: &PathBuf) -> AbsPathBuf {
    AbsPathBuf::try_from(manifest_dir.join("Cargo.toml").to_str().unwrap()).unwrap()
}

pub fn struct_reflection(reflect_ty_name: &str) -> proc_macro2::TokenStream {
    let struct_name = quote::format_ident!("{}Reflection", reflect_ty_name);
    let output_stream = quote::quote! {
        #[derive(Clone)]
        pub struct #struct_name {
            pub struct_meta: StructMeta,
        }
    };
    output_stream.into()
}

pub fn struct_reflection_new(
    reflect_ty_name: &str,
    fields_token_stream: proc_macro2::TokenStream,
    functions_token_stream: proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    let struct_name = quote::format_ident!("{}Reflection", reflect_ty_name);
    let output_stream = quote::quote! {
        impl #struct_name {
            pub fn new() -> Self {
                let struct_meta = StructMeta {
                    name: #reflect_ty_name.to_string(),
                    fields: vec![
                        #fields_token_stream
                    ],
                    functions: vec![
                        #functions_token_stream
                    ]
                };
                Self { struct_meta }
            }
        }
    };
    output_stream.into()
}

pub fn struct_field_meta_token_stream(name: &str, ty: &str) -> proc_macro2::TokenStream {
    let output_stream = quote::quote! {
        {
            StructFieldMeta {
                name: #name.to_string(),
                type_meta: TypeMeta {
                    name: #ty.to_string(),
                }
            }
        },
    };
    output_stream.into()
}

pub fn function_meta_token_stream(
    name: &str,
    return_ty: &str,
    params_token_streams: proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    let output_stream = quote::quote! {
        {
            FunctionMeta {
                name: #name.to_string(),
                params: vec![
                    #params_token_streams
                ],
                return_ty: TypeMeta {
                    name: #return_ty.to_string(),
                }
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

pub fn get_struct_name_of_impl(struct_impl: ra_ap_hir::Impl, db: &RootDatabase) -> Option<String> {
    let self_ty = struct_impl.self_ty(db);
    if let Some(adt) = self_ty.as_adt() {
        if let Some(s) = adt.as_struct() {
            return Some(s.name(db).as_str().to_string());
        }
    }
    return None;
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
        import_path_config(),
    );
    return use_path;
}

pub fn import_path_config() -> ra_ap_hir::ImportPathConfig {
    ra_ap_hir::ImportPathConfig {
        prefer_no_std: true,
        prefer_prelude: false,
        prefer_absolute: true,
    }
}

pub fn impl_struct_meta_container(
    reflect_ty_name: &str,
    exec_without_self_token_stream: proc_macro2::TokenStream,
    exec_with_mut_self_token_stream: proc_macro2::TokenStream,
    exec_with_self_token_stream: proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    let struct_name = quote::format_ident!("{}Reflection", reflect_ty_name);
    let output_stream = quote::quote! {
        impl StructMetaContainer for #struct_name {
            fn exec_without_self(
                &mut self,
                name: &str,
                mut params: Vec<Box<dyn std::any::Any>>,
            ) -> Option<Box<dyn std::any::Any>> {
                let _ = name;
                let _ = params;
                match name {
                    #exec_without_self_token_stream
                    _ => return None,
                }
                return None;
            }

            fn exec_with_mut_self(
                &mut self,
                name: &str,
                self_param: &mut dyn std::any::Any,
                mut params: Vec<Box<dyn std::any::Any>>,
            ) -> Option<Box<dyn std::any::Any>> {
                let _ = name;
                let _ = self_param;
                let _ = params;
                match name {
                    #exec_with_mut_self_token_stream
                    _ => return None,
                }
                return None;
            }

            fn exec_with_self(
                &mut self,
                name: &str,
                self_param: &dyn std::any::Any,
                mut params: Vec<Box<dyn std::any::Any>>,
            ) -> Option<Box<dyn std::any::Any>> {
                let _ = name;
                let _ = self_param;
                let _ = params;
                match name {
                    #exec_with_self_token_stream
                    _ => return None,
                }
                return None;
            }

            fn get_struct_meta(&self) -> &StructMeta {
                &self.struct_meta
            }
        }
    };
    output_stream.into()
}
