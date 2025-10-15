use crate::{
    analyzer,
    generated_module_partion::GeneratedModulePartion,
    misc::{
        make_api_code, make_assign_function, make_bind_function, make_param, make_param_list,
        make_return_value_expr, make_unwrap_object, resolve_struct_import_path_ident,
        EWrappedStructType,
    },
    register_function_maker::RegisterFunctionMaker,
};
use anyhow::anyhow;
use convert_case::Casing;
use proc_macro2::TokenStream;
use ra_ap_hir::{HasVisibility, Semantics, Visibility};
use ra_ap_ide_db::base_db::RootQueryDb;
use ra_ap_syntax::{
    ast::{self, HasModuleItem},
    SourceFile,
};
use ra_ap_vfs::AbsPathBuf;
use rs_core_minimal::path_ext::CanonicalizeSlashExt;
use std::{collections::HashMap, path::Path};
use std::{io::Write, path::PathBuf, str::FromStr};

pub struct EngineApiGenerator {}

impl EngineApiGenerator {
    pub fn new() -> EngineApiGenerator {
        EngineApiGenerator {}
    }

    pub fn run(&mut self, analyzer: &analyzer::Analyzer) -> anyhow::Result<()> {
        // if self.output_dir.is_dir() {
        // return Err(anyhow!(
        //     "{:?} {}",
        //     self.output_dir,
        //     std::io::ErrorKind::AlreadyExists
        // ));
        // }
        let output_root_dir =
            rs_core_minimal::file_manager::get_engine_generated_dir().join("v8_binding_api");
        if !output_root_dir.exists() {
            std::fs::create_dir_all(&output_root_dir)?;
        }

        let crates = analyzer.root_database.all_crates().clone();
        log::trace!("Num of crates: {}", crates.len());
        for db_krate in crates.iter() {
            let db = &analyzer.root_database;
            let sema: Semantics<'_, ra_ap_ide::RootDatabase> = Semantics::new(db);
            let db_krate_data = db_krate.data(db);
            let krate: ra_ap_hir::Crate = (*db_krate).into();
            let Some(crate_name) = krate.display_name(db).map(|x| x.canonical_name().clone())
            else {
                continue;
            };

            let env = db_krate.env(db);
            let Some(crate_cargo_manifest_dir) = env
                .get("CARGO_MANIFEST_DIR")
                .map(|x| AbsPathBuf::assert_utf8(Path::new(&x).to_path_buf()))
            else {
                continue;
            };
            let Some(_) = crate_cargo_manifest_dir.file_name() else {
                continue;
            };

            match &db_krate_data.origin {
                ra_ap_ide_db::base_db::CrateOrigin::Local { repo, name } => {
                    let _ = repo;
                    let Some(display_name) = name else {
                        continue;
                    };
                    let display_name = display_name.as_str();
                    if !display_name.starts_with("rs_")
                        || display_name.eq_ignore_ascii_case("rs_reflection_core")
                    {
                        continue;
                    }
                }
                _ => {
                    continue;
                }
            }

            let generated_crate_dir = output_root_dir.join(crate_name.as_str());
            if !generated_crate_dir.exists() {
                std::fs::create_dir_all(&generated_crate_dir)?;
            }

            let modules = krate.modules(db);
            if modules.is_empty() {
                continue;
            } else {
                log::trace!("Crate name: {}", crate_name);
            }

            let mut register_function_maker = RegisterFunctionMaker::new();
            for module_data in modules {
                let module_visibility = module_data.visibility(db);
                if module_visibility != Visibility::Public {
                    continue;
                }
                let definition_source_file_id = module_data.definition_source_file_id(db);
                let Some(editioned_file_id) = definition_source_file_id.file_id() else {
                    continue;
                };
                let source_file: SourceFile = sema.parse(editioned_file_id);
                let vfs = &analyzer.vfs;
                let file_path = vfs
                    .file_path(editioned_file_id.file_id(db))
                    .as_path()
                    .expect("Not null");
                let Some(relative_path) = file_path.strip_prefix(&crate_cargo_manifest_dir) else {
                    continue;
                };
                let target_output_suffix = Path::new(crate_name.as_str()).join(relative_path);

                let components: Vec<String> =
                    file_path.components().map(|x| x.to_string()).collect();
                if components.contains(&"ffi".to_string()) {
                    continue;
                }
                let mut find_rs_structs = vec![];
                let mut find_rs_struct_impls = vec![];
                let mut struct_map: HashMap<ra_ap_hir::Struct, Vec<ra_ap_hir::Impl>> =
                    HashMap::new();
                for item in source_file.items() {
                    match item {
                        ast::Item::Impl(rs_impl) => {
                            if let Some(rs_impl) = sema.to_impl_def(&rs_impl) {
                                find_rs_struct_impls.push(rs_impl);
                            }
                        }
                        ast::Item::Struct(rs_struct) => {
                            if let Some(rs_struct) = sema.to_struct_def(&rs_struct) {
                                let struct_visibility = rs_struct.visibility(db);
                                if struct_visibility != Visibility::Public {
                                    continue;
                                }
                                find_rs_structs.push(rs_struct);
                            }
                        }
                        _ => {}
                    }
                }

                for find_rs_struct in find_rs_structs {
                    for find_rs_struct_impl in find_rs_struct_impls.clone() {
                        let lhs = find_rs_struct_impl.self_ty(db).as_adt();
                        let rhs = find_rs_struct.ty(db).as_adt();
                        if lhs == rhs {
                            struct_map
                                .entry(find_rs_struct)
                                .or_default()
                                .push(find_rs_struct_impl);
                        }
                    }
                }

                let mut contents = quote::quote! {
                    /// THIS IS AN AUTOGENERATED FILE. DO NOT EDIT THIS FILE DIRECTLY.
                    #[allow(warnings)]
                    use std::{cell::RefCell, rc::Rc};
                    use anyhow::anyhow;
                    use rs_engine::input_mode::EInputMode;
                    use rs_render::{global_uniform::EDebugShadingType, view_mode::EViewModeType};
                    use rs_v8_host::{error::Error, util::return_exception, v8_runtime::{Constructible, Constructor, Newable, CPPGC_TAG}};
                    use winit::{event::ElementState, keyboard::KeyCode};
                };
                let mut is_write = false;
                for (rs_struct, rs_struct_impls) in struct_map {
                    for wrap_type in EWrappedStructType::all() {
                        if let Ok(GeneratedModulePartion {
                            code,
                            mut binding_api_types,
                        }) =
                            Self::code_gen(rs_struct, rs_struct_impls.clone(), analyzer, wrap_type)
                        {
                            contents.extend(code);
                            is_write = true;

                            let key = PathBuf::from_str(
                                relative_path
                                    .as_str()
                                    .strip_prefix("src")
                                    .unwrap_or(relative_path.as_str()),
                            )?;
                            register_function_maker
                                .type_map_mut()
                                .entry(key)
                                .or_default()
                                .append(&mut binding_api_types);
                        }
                    }
                }
                if is_write {
                    let output_path = output_root_dir.join(target_output_suffix);
                    let parent_dir = output_path.parent().expect("A valid path");
                    if !parent_dir.exists() {
                        let _ = std::fs::create_dir_all(&parent_dir);
                    }
                    log::debug!("{:?}", &output_path);
                    std::fs::write(&output_path, contents.to_string())?;
                }
            }

            let generated_manifest_file_path = generated_crate_dir.join("Cargo.toml");
            std::fs::write(
                &generated_manifest_file_path,
                Self::manifest_content(&format!("{}_v8_binding_api", crate_name.as_str())),
            )?;
            if generated_crate_dir.join("src").exists() {
                EngineApiGenerator::create_module_files(&generated_crate_dir)?;
                if generated_crate_dir.join("src/lib.rs").exists() {
                    let register_function_code = register_function_maker.make().to_string();
                    let mut file = std::fs::OpenOptions::new()
                        .append(true)
                        .open(generated_crate_dir.join("src/lib.rs"))?;
                    writeln!(&mut file, "{}", register_function_code)?;
                }
            }
        }

        Ok(())
    }

    fn create_module_files(generated_crate_dir: &Path) -> anyhow::Result<()> {
        let src_dir = generated_crate_dir.join("src");
        if !src_dir.is_dir() {
            return Err(anyhow!("src folder is not exists"));
        }
        for entry in walkdir::WalkDir::new(src_dir) {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                let mut contents = String::new();
                for entry in std::fs::read_dir(path)? {
                    let entry = entry?;
                    let file_stem = entry
                        .path()
                        .file_stem()
                        .ok_or(anyhow!(""))?
                        .to_string_lossy()
                        .to_string();
                    if !(file_stem == "main" || file_stem == "lib" || file_stem == "mod") {
                        contents += &format!("#[allow(warnings)]\npub mod {};\n", file_stem);
                    }
                }
                let filename = if path.file_name() == Some(std::ffi::OsStr::new("src")) {
                    "lib.rs"
                } else {
                    "mod.rs"
                };
                log::debug!("{:?}", &path.join(filename));
                std::fs::write(&path.join(filename), contents)?;
            }
        }
        Ok(())
    }

    pub fn manifest_content(crate_name: &str) -> String {
        let v8_version = {
            let mut cargo_manifest = rs_manifest::CargoManifest::new(
                rs_core_minimal::file_manager::get_engine_root_dir().join("rs_v8_host/Cargo.toml"),
            )
            .unwrap();
            let mut versions = HashMap::from([("v8", "".to_string())]);
            cargo_manifest.read_create_version(&mut versions);
            versions["v8"].clone()
        };

        let (log_version, anyhow_version) = {
            let mut cargo_manifest = rs_manifest::CargoManifest::new(
                rs_core_minimal::file_manager::get_engine_root_dir()
                    .join("programs/rs_v8_binding_api_generator/Cargo.toml"),
            )
            .unwrap();
            let mut versions = HashMap::from([("log", "".to_string()), ("anyhow", "".to_string())]);
            cargo_manifest.read_create_version(&mut versions);
            (versions["log"].clone(), versions["anyhow"].clone())
        };

        let winit_version = {
            let mut cargo_manifest = rs_manifest::CargoManifest::new(
                rs_core_minimal::file_manager::get_engine_root_dir().join("rs_engine/Cargo.toml"),
            )
            .unwrap();
            let mut versions = HashMap::from([("winit", "".to_string())]);
            cargo_manifest.read_create_version(&mut versions);
            versions["winit"].clone()
        };

        let engine_root_dir = rs_core_minimal::file_manager::get_engine_root_dir()
            .canonicalize_slash()
            .expect("Success");
        let mut template = r#"[package]
name = "@crate_name@"
version = "0.1.0"
edition = "2021"

[dependencies]
v8 = "@v8_version@"
log = "@log_version@"
anyhow = { version = "@anyhow_version@" }
winit = { version = "@winit_version@" }
rs_engine = { path = "@engine_dir@/rs_engine" }
rs_render = { path = "@engine_dir@/rs_render" }
rs_v8_host = { path = "@engine_dir@/rs_v8_host" }
rs_core_minimal = { path = "@engine_dir@/rs_core_minimal" }
"#
        .to_string();
        template = template.replace("@engine_dir@", engine_root_dir.to_str().expect("Success"));
        template = template.replace("@v8_version@", &v8_version);
        template = template.replace("@log_version@", &log_version);
        template = template.replace("@anyhow_version@", &anyhow_version);
        template = template.replace("@crate_name@", &crate_name);
        template = template.replace("@winit_version@", &winit_version);
        template
    }

    fn code_gen(
        rs_struct: ra_ap_hir::Struct,
        rs_struct_impls: Vec<ra_ap_hir::Impl>,
        analyzer: &analyzer::Analyzer,
        wrap_type: EWrappedStructType,
    ) -> anyhow::Result<GeneratedModulePartion> {
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

        let mut binding_api_type_name = TokenStream::new();
        let code = make_api_code(
            &wrapped_struct_name,
            rs_struct.name(db).as_str(),
            quote::quote! { #struct_import_path },
            wrap_type,
            set_function_bindings,
            function_bindings,
            &mut binding_api_type_name,
        );

        Ok(GeneratedModulePartion {
            code,
            binding_api_types: vec![binding_api_type_name],
        })
    }
}
