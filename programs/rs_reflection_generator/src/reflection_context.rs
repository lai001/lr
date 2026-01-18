use crate::misc::param_meta_token_stream;
use crate::misc::reflection_token_stream_template;
use crate::misc::struct_field_meta_token_stream;
use anyhow::anyhow;
use ast::HasModuleItem;
use proc_macro2::TokenStream;
use quote::quote;
use ra_ap_hir::GenericDef;
use ra_ap_hir::HasVisibility;
use ra_ap_hir::HirDisplay;
use ra_ap_ide::*;
use ra_ap_ide_db::base_db::RootQueryDb;
use ra_ap_ide_db::*;
use ra_ap_load_cargo::*;
use ra_ap_proc_macro_api::ProcMacroClient;
use ra_ap_project_model::*;
use ra_ap_syntax::*;
use ra_ap_vfs::*;
use std::fmt::Debug;
use std::path::Path;
use std::{cell::RefCell, rc::Rc};

pub struct ParseResult {
    pub rs_struct: ra_ap_hir::Struct,
    pub impl_defs: Vec<ra_ap_hir::Impl>,
    pub db: Rc<RootDatabase>,
    pub vfs: Rc<RefCell<Vfs>>,
    pub file_path: AbsPathBuf,
    pub target_output_suffix: std::path::PathBuf,
}

impl Debug for ParseResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let names_group = self
            .impl_defs
            .iter()
            .map(|x| {
                let mut names: Vec<String> = vec![];
                let items = x.items(self.db.as_ref());
                for item in items {
                    let name = item
                        .name(self.db.as_ref())
                        .map(|x| x.as_str().to_string())
                        .unwrap_or_default();
                    names.push(name);
                }
                return names;
            })
            .collect::<Vec<Vec<String>>>();
        f.debug_struct("ParseResult")
            .field("rs_struct", &self.rs_struct.name(self.db.as_ref()))
            .field("impl_defs", &names_group)
            .finish()
    }
}

impl ParseResult {
    pub fn generate_reflection_token_stream(&self) -> anyhow::Result<TokenStream> {
        let db = self.db.as_ref();
        // let sema = Semantics::new(db);
        let struct_name = self.rs_struct.name(db).as_str().to_string();
        log::trace!("Generate reflection: {:?}", &struct_name);

        let mut fields_token_stream = TokenStream::new();
        let mut functions_token_stream = TokenStream::new();

        for field in self.rs_struct.fields(db) {
            let name = field.name(db).as_str().to_string();
            log::trace!("Field: {}", name);
            let ty = field.ty(db);
            let module = self.rs_struct.module(db);
            //sema.scope(pat.syntax())?.module();
            let inferred_type = ty
                .display_source_code(db, module.into(), false)
                .map_err(|err| anyhow!("{:?}", err))?;

            fields_token_stream.extend(struct_field_meta_token_stream(&name, &inferred_type));
        }

        let mut exec_token_stream = TokenStream::new();
        for impl_def in self.impl_defs.clone() {
            for assoc_item in impl_def.items(db) {
                match assoc_item {
                    ra_ap_hir::AssocItem::Function(function) => {
                        if !Self::is_supported_function(db, &function) {
                            continue;
                        }
                        let fn_name = function.name(db).as_str().to_string();
                        log::trace!("Function: {}", &fn_name);
                        let function_tokens =
                            generate_exec_function(db, &self.rs_struct, &function);
                        if let Ok(function_tokens) = function_tokens {
                            functions_token_stream.extend(function_tokens.register);
                            exec_token_stream.extend(function_tokens.body);
                        }
                    }
                    _ => {}
                }
            }
        }

        Ok(reflection_token_stream_template(
            &struct_name,
            fields_token_stream,
            functions_token_stream,
            exec_token_stream,
        ))
    }

    fn is_supported_type(db: &RootDatabase, ty: ra_ap_hir::Type, krate: &ra_ap_hir::Crate) -> bool {
        let display_target = krate.to_display_target(db);
        (ty.type_and_const_arguments(db, display_target).count() > 0
            || ty.is_closure()
            || ty.is_fn()
            || ty.as_callable(db).is_some()
            || ty.is_never()
            || ty.as_dyn_trait().is_some()
            || ty.is_unknown()
            || ty.impls_fnonce(db))
            || ty.as_impl_traits(db).is_some() == false
    }

    fn is_supported_function(db: &RootDatabase, function: &ra_ap_hir::Function) -> bool {
        let generics = GenericDef::from(*function);

        if let Some(type_params) = generics
            .type_or_const_params(db)
            .into_iter()
            .map(|it| it.as_type_param(db))
            .collect::<Option<Vec<ra_ap_hir::TypeParam>>>()
        {
            if !type_params.is_empty() {
                return false;
            }
        }

        let krate = function.module(db).krate(db);

        if !Self::is_supported_type(db, function.ty(db), &krate) {
            return false;
        }
        let fn_name = function.name(db).as_str().to_string();
        let return_type = function.ret_type(db);

        if !Self::is_supported_type(db, return_type, &krate) {
            return false;
        }
        if fn_name == "Drop" {
            return false;
        }
        if function.visibility(db) != ra_ap_hir::Visibility::Public {
            return false;
        }
        let params = if let Some(_) = function.self_param(db) {
            function.params_without_self(db)
        } else {
            function.assoc_fn_params(db)
        };
        for param in params {
            let ty = param.ty();
            if !Self::is_supported_type(db, ty.clone(), &krate) {
                return false;
            }
        }
        true
    }
}

pub struct ReflectionContext {
    _manifest_file_path: AbsPathBuf,
    _project_manifest: ProjectManifest,
    _cargo_config: CargoConfig,
    _project_workspace: ProjectWorkspace,
    db: Rc<RootDatabase>,
    vfs: Rc<RefCell<Vfs>>,
    _proc_macro: Option<ProcMacroClient>,
}

impl ReflectionContext {
    pub fn new(manifest_file_path: AbsPathBuf) -> anyhow::Result<Self> {
        let mut cargo_config = CargoConfig::default();
        cargo_config.sysroot = Some(RustLibSource::Discover);
        let project_manifest = ProjectManifest::from_manifest_file(manifest_file_path.clone())?;
        let project_workspace = ProjectWorkspace::load(
            project_manifest.clone(),
            &cargo_config,
            &load_project_workspace_progress,
        )?;
        let load_cargo_config: LoadCargoConfig = LoadCargoConfig {
            load_out_dirs_from_check: true,
            with_proc_macro_server: ProcMacroServerChoice::None,
            prefill_caches: false,
        };
        let (db, vfs, proc_macro) = load_workspace(
            project_workspace.clone(),
            &cargo_config.extra_env,
            &load_cargo_config,
        )?;

        Ok(Self {
            _manifest_file_path: manifest_file_path,
            _project_manifest: project_manifest,
            _cargo_config: cargo_config,
            _project_workspace: project_workspace,
            db: Rc::new(db),
            vfs: Rc::new(RefCell::new(vfs)),
            _proc_macro: proc_macro,
        })
    }

    pub fn parse_crate(&self) -> Vec<ParseResult> {
        let mut parse_results: Vec<ParseResult> = vec![];
        let db = self.db.as_ref();

        let crates = db.all_crates().clone();
        log::trace!("Num of crates: {}", crates.len());
        for db_krate in crates.iter() {
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
                base_db::CrateOrigin::Local { repo, name } => {
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
            let modules = krate.modules(db);
            if !modules.is_empty() {
                log::trace!("Crate name: {}", crate_name);
            }
            for module_data in modules {
                let definition_source_file_id = module_data.definition_source_file_id(db);
                let Some(editioned_file_id) = definition_source_file_id.file_id() else {
                    continue;
                };

                let file_path = self
                    .vfs
                    .borrow()
                    .file_path(editioned_file_id.file_id(db))
                    .as_path()
                    .expect("Not null")
                    .to_path_buf()
                    .normalize();

                let Some(relative_path) = file_path.strip_prefix(&crate_cargo_manifest_dir) else {
                    continue;
                };

                let target_output_suffix = Path::new(crate_name.as_str()).join(relative_path);

                let source_file: SourceFile = sema.parse(editioned_file_id);
                let mut find_rs_structs: Vec<ra_ap_hir::Struct> = vec![];
                let mut find_rs_struct_impls: Vec<ra_ap_hir::Impl> = vec![];

                for item in source_file.items() {
                    match item {
                        ast::Item::Impl(impl_item) => {
                            if let Some(impl_def) = sema.to_impl_def(&impl_item) {
                                find_rs_struct_impls.push(impl_def);
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
                    let mut match_impls = vec![];
                    for struct_impl in &find_rs_struct_impls {
                        let lhs = struct_impl.self_ty(db).as_adt();
                        let rhs = find_rs_struct.ty(db).as_adt();
                        if lhs == rhs {
                            match_impls.push(struct_impl.clone());
                        }
                    }
                    let result = ParseResult {
                        rs_struct: find_rs_struct,
                        impl_defs: match_impls,
                        db: self.db.clone(),
                        vfs: self.vfs.clone(),
                        file_path: file_path.clone(),
                        target_output_suffix: target_output_suffix.clone(),
                    };
                    parse_results.push(result);
                }
            }
        }

        parse_results
    }

    pub fn dump_all_files(&self) {
        for (file_id, path) in self.vfs.borrow().iter() {
            log::trace!("{:?}, {}", file_id, path);
        }
    }

    pub fn project_workspace(&self) -> &ProjectWorkspace {
        &self._project_workspace
    }

    pub fn db(&self) -> &RootDatabase {
        &self.db
    }
}

fn load_project_workspace_progress(message: String) -> () {
    let _ = message;
}

struct FunctionTokens {
    body: TokenStream,
    register: TokenStream,
}

fn generate_exec_function(
    db: &RootDatabase,
    rs_struct: &ra_ap_hir::Struct,
    function: &ra_ap_hir::Function,
) -> anyhow::Result<FunctionTokens> {
    let fn_name = function.name(db).as_str().to_string();
    let struct_name = rs_struct.name(db).as_str().to_string();
    let struct_name_token: TokenStream = struct_name.parse().map_err(|err| anyhow!("{err}"))?;
    let fn_name_token: TokenStream = fn_name.parse().map_err(|err| anyhow!("{err}"))?;
    let return_type = function.ret_type(db);
    let module = rs_struct.module(db);
    let inferred_return_type = return_type
        .display_source_code(db, module.into(), false)
        .map_err(|err| anyhow!("{:?}", err))?;
    let reflected_function_name = format!("exec_{}", fn_name_token)
        .parse::<TokenStream>()
        .map_err(|err| anyhow!("{err}"))?;

    let mut params_token_stream = TokenStream::new();

    let params = if let Some(_) = function.self_param(db) {
        function.params_without_self(db)
    } else if let Some(params) = function.method_params(db) {
        params
    } else {
        function.assoc_fn_params(db)
    };
    for param in &params {
        let name = param.name(db).map(|x| x.as_str().to_string());
        let ty = param.ty();
        let module = rs_struct.module(db);
        let inferred_type = ty
            .display_source_code(db, module.into(), false)
            .map_err(|err| anyhow!("{:?}", err))?;
        params_token_stream.extend(param_meta_token_stream(name.as_deref(), &inferred_type));
    }

    let register: TokenStream;
    if let Some(self_param) = function.self_param(db) {
        let exec_type: TokenStream;
        if self_param.ty(db).is_mutable_reference() {
            exec_type = quote! { ExecMut };
        } else if self_param.ty(db).is_reference() {
            exec_type = quote! { Exec };
        } else {
            return Err(anyhow!("Not support"));
        }
        register = quote! {
            Function {
                meta: FunctionMeta {
                    name: #fn_name.to_string(),
                    params: vec![#params_token_stream],
                    return_ty: TypeMeta { name: #inferred_return_type.to_string() },
                },
                exec_type: FunctionExecType::#exec_type(Box::new(#reflected_function_name)),
            },
        };
    } else {
        let contains_reference = params
            .iter()
            .find(|x| x.ty().contains_reference(db))
            .is_some();
        if contains_reference {
            return Err(anyhow!("Not support"));
        }
        register = quote! {
            Function {
                meta: FunctionMeta {
                    name: #fn_name.to_string(),
                    params: vec![#params_token_stream],
                    return_ty: TypeMeta { name: #inferred_return_type.to_string() },
                },
                exec_type: FunctionExecType::StaticExec(Box::new(#reflected_function_name)),
            },
        };
    }

    let params_len_token_stream: TokenStream = (params.len() as isize - 1)
        .to_string()
        .parse()
        .map_err(|err| anyhow!("{err}"))?;
    let check_len = if params.is_empty() {
        TokenStream::new()
    } else {
        quote! {
            if params.len() <= #params_len_token_stream {
                return Err(rs_reflection_core::error::Error::WrongNumberOfParameters);
            }
        }
    };

    if let Some(self_param) = function.self_param(db) {
        if !(self_param.ty(db).is_mutable_reference() || self_param.ty(db).is_reference()) {
            return Err(anyhow!("Not support, self param is not reference"));
        }
    }

    let mut parameter_list_token_stream = TokenStream::new();
    let mut cast_tokens = TokenStream::new();
    let mut args_token = TokenStream::new();

    for (index, param) in params.iter().enumerate() {
        let var_name = format!("v{}", index)
            .parse::<TokenStream>()
            .map_err(|err| anyhow!("{err}"))?;
        let stream = format!("let {} = params.remove(0);", var_name);
        parameter_list_token_stream.extend(stream.parse::<TokenStream>());

        let mut is_strip_reference = false;
        if param.ty().is_reference() {
            if param.ty().strip_reference().is_array() {
            } else if param.ty().strip_reference().is_str() {
            } else if param.ty().strip_reference().is_slice() {
            } else {
                is_strip_reference = true;
            }
        }

        let inferred_param_type = if is_strip_reference {
            param.ty().strip_reference()
        } else {
            param.ty().clone()
        }
        .display_source_code(db, module.into(), false)
        .map(|x| x.parse::<TokenStream>())
        .map_err(|err| anyhow!("{err:?}"))?
        .map_err(|err| anyhow!("{err}"))?;

        let cast_token = if param.ty().is_mutable_reference() {
            args_token.extend(format!("{},", var_name).parse::<TokenStream>());
            quote! {
                let #var_name = match #var_name {
                    ReflectArg::MutRef(val) => val
                        .downcast_mut::<#inferred_param_type>()
                        .ok_or(rs_reflection_core::error::Error::WrongParameterType)?,
                    _ => return Err(rs_reflection_core::error::Error::WrongOwnership),
                };
            }
        } else if param.ty().is_reference() {
            args_token.extend(format!("{},", var_name).parse::<TokenStream>());
            quote! {
                let #var_name = match #var_name {
                    ReflectArg::Ref(val) => val
                        .downcast_ref::<#inferred_param_type>()
                        .ok_or(rs_reflection_core::error::Error::WrongParameterType)?,
                    _ => return Err(rs_reflection_core::error::Error::WrongOwnership),
                };
            }
        } else {
            args_token.extend(format!("*{},", var_name).parse::<TokenStream>());
            quote! {
                let #var_name = match #var_name {
                    ReflectArg::Owned(val) => val
                        .downcast::<#inferred_param_type>()
                        .map_err(|_| rs_reflection_core::error::Error::WrongParameterType)?,
                    _ => return Err(rs_reflection_core::error::Error::WrongOwnership),
                };
            }
        };
        cast_tokens.extend(cast_token);
    }

    let mut return_value_name = TokenStream::new();
    let mut is_return_value = false;
    let return_token = determine_return(
        db,
        &mut is_return_value,
        &mut return_value_name,
        return_type,
    )?;
    let left_exp = if is_return_value {
        quote! {
            let #return_value_name =
        }
    } else {
        TokenStream::new()
    };

    let invoke_stream: TokenStream;
    if let Some(self_param) = function.self_param(db) {
        let ref_type = if self_param.ty(db).is_mutable_reference() {
            quote! { mut }
        } else if self_param.ty(db).is_reference() {
            quote! { const }
        } else {
            unreachable!()
        };
        invoke_stream = quote! {
            let this = unsafe { (this as *#ref_type dyn std::any::Any as *mut #struct_name_token).as_mut() }
                .ok_or(rs_reflection_core::error::Error::Null)?;
            #left_exp this.#fn_name_token(#args_token);
        };
    } else {
        invoke_stream = quote! {
            #left_exp #struct_name_token::#fn_name_token(#args_token);
        };
    }

    let type_check_stream: TokenStream;
    if function.self_param(db).is_some() {
        type_check_stream = quote! {
            if !this.is::<#struct_name_token>() {
                return Err(rs_reflection_core::error::Error::TypeMismatch(format!(
                    "{:?} != {:?}",
                    rs_reflection_core::get_type_id(this),
                    std::any::TypeId::of::<#struct_name_token>()
                )));
            }
        };
    } else {
        type_check_stream = TokenStream::new();
    }
    let args_stream: TokenStream;
    let params_name: TokenStream = if params.is_empty() {
        quote! {_}
    } else {
        quote! {params}
    };
    if let Some(self_param) = function.self_param(db) {
        if self_param.ty(db).is_mutable_reference() {
            args_stream = quote! {
                this: &'a mut (dyn std::any::Any + 'static), #params_name: &'a mut Vec<ReflectArg<'a>>
            };
        } else if self_param.ty(db).is_reference() {
            args_stream = quote! {
                this: &'a (dyn std::any::Any + 'static), #params_name: &'a mut Vec<ReflectArg<'a>>
            };
        } else {
            unreachable!()
        }
    } else {
        args_stream = quote! {
            #params_name: &mut Vec<ReflectArg>
        };
    }

    let body = quote! {
        pub fn #reflected_function_name<'a>(#args_stream) -> rs_reflection_core::error::Result<Option<ReflectArg<'a>>> {
            #check_len
            #type_check_stream
            #parameter_list_token_stream
            #cast_tokens
            #invoke_stream
            #return_token
        }
    };

    return Ok(FunctionTokens { body, register });
}

fn determine_return(
    db: &RootDatabase,
    is_return_value: &mut bool,
    return_value_name: &mut TokenStream,
    return_type: ra_ap_hir::Type<'_>,
) -> anyhow::Result<TokenStream> {
    *is_return_value = true;
    *return_value_name = quote! { value };
    if return_type.is_unit() {
        *is_return_value = false;
        *return_value_name = quote! { _ };
        return Ok(quote! {
            return Ok(None);
        });
    } else {
        if return_type.is_mutable_reference() {
            return Ok(quote! {
                return Ok(Some(ReflectArg::MutRef(value)));
            });
        } else if return_type.is_reference() {
            return Ok(quote! {
                return Ok(Some(ReflectArg::Ref(value)));
            });
        } else if let Some(adt) = return_type.as_adt() {
            if let Some(as_enum) = adt.as_enum() {
                let name = format!("{:?}", ra_ap_hir::LangItem::Option);
                if as_enum.name(db).as_str().eq_ignore_ascii_case(&name) {
                    if return_type.contains_reference(db) {
                        return Err(anyhow!("Not support"));
                    }
                    if let Some(arg) = return_type.type_arguments().peekable().peek() {
                        if arg.is_mutable_reference() {
                            quote! {
                                return Ok(Some(ReflectArg::OptionMutRef(value)));
                            };
                        } else if arg.is_reference() {
                            quote! {
                                return Ok(Some(ReflectArg::OptionRef(value)));
                            };
                        }
                    }
                }
            }
        }
    }
    return Ok(quote! {
        return Ok(Some(ReflectArg::Owned(Box::new(value))));
    });
}
