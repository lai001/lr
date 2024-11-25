use crate::{
    analyzer,
    misc::{
        make_api_code, make_assign_function, make_bind_function, make_param, make_param_list,
        make_return_value_expr, make_unwrap_object, resolve_struct_import_path_ident,
        EWrappedStructType,
    },
};
use anyhow::anyhow;
use convert_case::Casing;
use proc_macro2::TokenStream;
use ra_ap_hir::{HasVisibility, Semantics, Visibility};
use ra_ap_ide_db::EditionedFileId;
use ra_ap_syntax::{
    ast::{self, HasModuleItem},
    SourceFile,
};
use ra_ap_vfs::*;
use rs_core_minimal::path_ext::CanonicalizeSlashExt;
use std::{collections::HashMap, path::PathBuf};

pub struct EngineApiGenerator {
    pub output_dir: PathBuf,
}

impl EngineApiGenerator {
    pub fn new(output_dir: PathBuf) -> Self {
        Self { output_dir }
    }

    pub fn run(&mut self, analyzer: &mut analyzer::Analyzer) -> anyhow::Result<()> {
        // if self.output_dir.is_dir() {
        // return Err(anyhow!(
        //     "{:?} {}",
        //     self.output_dir,
        //     std::io::ErrorKind::AlreadyExists
        // ));
        // }
        let _ = std::fs::create_dir(&self.output_dir);
        let _ = std::fs::create_dir(&self.output_dir.join("src"));
        std::fs::write(
            &self.output_dir.join("Cargo.toml"),
            Self::manifest_content(),
        )?;
        std::fs::write(
            &self.output_dir.join("src/lib.rs"),
            Self::lib_file_content(),
        )?;

        for (_, vfs_path) in analyzer
            .vfs
            .iter()
            .map(|x| (x.0.clone(), x.1.clone()))
            .collect::<Vec<(FileId, VfsPath)>>()
        {
            let db = &analyzer.root_database;
            let Some(path) = vfs_path.as_path() else {
                continue;
            };
            if path.extension() != Some("rs") {
                continue;
            }
            let components: Vec<String> = path.components().map(|x| x.to_string()).collect();
            if !components.contains(&"rs_engine".to_string())
                || components.contains(&"ffi".to_string())
            {
                continue;
            }

            let source_file_id = analyzer
                .vfs
                .file_id(&vfs_path)
                .ok_or(anyhow!("No source file found"))?;
            let editioned_file_id: EditionedFileId =
                EditionedFileId::current_edition(source_file_id);

            let sema: Semantics<'_, ra_ap_ide::RootDatabase> = Semantics::new(db);
            let source_file: SourceFile = sema.parse(editioned_file_id);

            let mut find_rs_structs = vec![];
            let mut find_rs_struct_impls = vec![];
            let mut struct_map: HashMap<ra_ap_hir::Struct, Vec<ra_ap_hir::Impl>> = HashMap::new();
            for item in source_file.items() {
                match item {
                    ast::Item::Impl(rs_impl) => {
                        if let Some(rs_impl) = sema.to_impl_def(&rs_impl) {
                            find_rs_struct_impls.push(rs_impl);
                        }
                    }
                    ast::Item::Struct(rs_struct) => {
                        if let Some(rs_struct) = sema.to_struct_def(&rs_struct) {
                            find_rs_structs.push(rs_struct);
                        }
                    }
                    _ => {}
                }
            }

            for find_rs_struct in find_rs_structs {
                for find_rs_struct_impl in find_rs_struct_impls.clone() {
                    if find_rs_struct_impl.self_ty(db) == find_rs_struct.ty(db) {
                        struct_map
                            .entry(find_rs_struct)
                            .or_default()
                            .push(find_rs_struct_impl);
                    }
                }
            }

            let mut content = quote::quote! {
                #![allow(warnings)]
                use std::{cell::RefCell, rc::Rc};
                use anyhow::anyhow;
                use rs_engine::input_mode::EInputMode;
                use rs_render::{global_uniform::EDebugShadingType, view_mode::EViewModeType};
                use rs_v8_host::{util::return_exception, v8_runtime::CPPGC_TAG};
            };
            let mut is_write = false;
            for (rs_struct, rs_struct_impls) in struct_map {
                for wrap_type in EWrappedStructType::all() {
                    if let Ok(token_stream) =
                        Self::code_gen(rs_struct, rs_struct_impls.clone(), analyzer, wrap_type)
                    {
                        content.extend(token_stream);
                        is_write = true;
                    }
                }
            }
            if is_write {
                let output_file_name =
                    format!("src/native_{}.rs", path.file_stem().expect("Not null"));
                std::fs::write(&self.output_dir.join(output_file_name), content.to_string())?;
            }
        }

        Ok(())
    }

    pub fn manifest_content() -> String {
        let engine_root_dir = rs_core_minimal::file_manager::get_engine_root_dir()
            .canonicalize_slash()
            .expect("Success");
        let mut template = r#"
[package]
name = "rs_v8_engine_binding_api"
version = "0.1.0"
edition = "2021"

[dependencies]
v8 = "130.0.0"
log = "0.4.22"
anyhow = { version = "1.0.92" }
rs_engine = { path = "@engine_dir@/rs_engine" }
rs_render = { path = "@engine_dir@/rs_render" }
rs_v8_host = { path = "@engine_dir@/rs_v8_host" }        
        "#
        .to_string();
        template = template.replace("@engine_dir@", engine_root_dir.to_str().expect("Success"));
        template
    }

    pub fn lib_file_content() -> String {
        let template = r#"
pub mod native_engine;
pub mod native_level;
pub mod native_player_viewport;
        "#
        .to_string();
        template
    }

    fn code_gen(
        rs_struct: ra_ap_hir::Struct,
        rs_struct_impls: Vec<ra_ap_hir::Impl>,
        analyzer: &mut analyzer::Analyzer,
        wrap_type: EWrappedStructType,
    ) -> anyhow::Result<TokenStream> {
        let db = &analyzer.root_database;
        // let sema: Semantics<'_, ra_ap_ide::RootDatabase> = Semantics::new(db);
        let struct_import_path = resolve_struct_import_path_ident(db, &rs_struct);

        let wrapped_struct_name =
            format!("native_{}", rs_struct.name(db).as_str()).to_case(convert_case::Case::Pascal);

        let mut set_function_bindings: TokenStream = TokenStream::default();
        let mut function_bindings: TokenStream = TokenStream::default();

        for struct_impl in rs_struct_impls {
            let items = struct_impl.items(db);
            for item in items {
                match item {
                    ra_ap_hir::AssocItem::Function(function) => {
                        let visibility = function.visibility(db);
                        if visibility != Visibility::Public {
                            continue;
                        }
                        let function_name = function.name(db).as_str().to_string();
                        let function_name_token_stream =
                            function.name(db).as_str().parse::<TokenStream>().unwrap();

                        if let Some(_) = function.self_param(db) {
                            let return_type = function.ret_type(db);
                            let return_value_expr = make_return_value_expr(db, &return_type);
                            let params = function.params_without_self(db);

                            let mut args_token_stream: Option<TokenStream> = None;
                            if params.is_empty() {
                                args_token_stream = Some(TokenStream::default());
                            }
                            for (index, param) in params.iter().enumerate() {
                                if let Ok(stream) = make_param(index, param, &db) {
                                    args_token_stream
                                        .get_or_insert_with(|| TokenStream::default())
                                        .extend(stream);
                                } else {
                                    args_token_stream = None;
                                    break;
                                }
                            }

                            if let (Some(args_token_stream), Ok(return_value_expr)) =
                                (args_token_stream, return_value_expr)
                            {
                                set_function_bindings.extend(make_assign_function(
                                    &function_name,
                                    &wrapped_struct_name,
                                    wrap_type,
                                ));

                                let unwrap_object =
                                    make_unwrap_object(&wrapped_struct_name, wrap_type);

                                let make_param_list = make_param_list(params.len());
                                function_bindings.extend(make_bind_function(
                                    &function_name,
                                    &quote::quote! {
                                        #args_token_stream
                                        #unwrap_object
                                        let return_value = unwrapped_value.#function_name_token_stream(#make_param_list);
                                        #return_value_expr
                                    },
                                    params.len(),
                                ));
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        let code = make_api_code(
            &wrapped_struct_name,
            rs_struct.name(db).as_str(),
            quote::quote! { #struct_import_path },
            wrap_type,
            set_function_bindings,
            function_bindings,
        );
        Ok(code)
    }
}
