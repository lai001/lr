use anyhow::anyhow;
use convert_case::Casing;
use proc_macro2::TokenStream;
use ra_ap_hir::{DisplayTarget, HasCrate, HirDisplay};
use ra_ap_hir_def::builtin_type;
use ra_ap_ide::RootDatabase;

#[derive(Clone, Copy)]
pub enum EWrappedStructType {
    RcRefCell,
    StaticLifeTimeNotNullPtr,
}

impl EWrappedStructType {
    pub fn all() -> Vec<EWrappedStructType> {
        vec![
            EWrappedStructType::RcRefCell,
            EWrappedStructType::StaticLifeTimeNotNullPtr,
        ]
    }
}

pub fn convet_builtin(
    builtin_type: ra_ap_hir::BuiltinType,
) -> ra_ap_hir_def::builtin_type::BuiltinType {
    if builtin_type.is_bool() {
        ra_ap_hir_def::builtin_type::BuiltinType::Bool
    } else if builtin_type.is_int() {
        let builtin_int = ra_ap_hir_ty::primitive::BuiltinInt::from_suffix(
            &builtin_type.name().symbol().to_string(),
        )
        .expect("Valid");
        ra_ap_hir_def::builtin_type::BuiltinType::Int(builtin_int)
    } else if builtin_type.is_uint() {
        let builtin_uint = ra_ap_hir_ty::primitive::BuiltinUint::from_suffix(
            &builtin_type.name().symbol().to_string(),
        )
        .expect("Valid");
        ra_ap_hir_def::builtin_type::BuiltinType::Uint(builtin_uint)
    } else if builtin_type.is_float() {
        let builtin_float = ra_ap_hir_ty::primitive::BuiltinFloat::from_suffix(
            &builtin_type.name().symbol().to_string(),
        )
        .expect("Valid");
        ra_ap_hir_def::builtin_type::BuiltinType::Float(builtin_float)
    } else if builtin_type.is_char() {
        ra_ap_hir_def::builtin_type::BuiltinType::Char
    } else if builtin_type.is_str() {
        ra_ap_hir_def::builtin_type::BuiltinType::Str
    } else {
        unreachable!()
    }
}

pub fn make_bind_function(
    function_name: &str,
    function_body: &TokenStream,
    params_len: usize,
) -> TokenStream {
    let function_name = function_name.parse::<TokenStream>().unwrap();
    let params_len = params_len as i32;
    if params_len == 0 {
        quote::quote! {
            pub fn #function_name(
                scope: &mut v8::PinScope,
                args: v8::FunctionCallbackArguments,
                mut ret_val: v8::ReturnValue,
            ) {
                #function_body
            }
        }
    } else {
        quote::quote! {
            pub fn #function_name(
                scope: &mut v8::PinScope,
                args: v8::FunctionCallbackArguments,
                mut ret_val: v8::ReturnValue,
            ) {
                if args.length() < #params_len {
                    return_exception(scope, &mut ret_val, "Too few parameters");
                    return;
                }
                #function_body
            }
        }
    }
}

pub fn make_unwrap_object(native_struct_name: &str, wrap_type: EWrappedStructType) -> TokenStream {
    let prefix = prefix(wrap_type);
    let expect_variable_name = "unwrapped_value";
    let name = expect_variable_name.parse::<TokenStream>().unwrap();
    let native_struct_name = format!("{}{}", prefix, native_struct_name)
        .parse::<TokenStream>()
        .unwrap();
    quote::quote! {
        let #name = unsafe {
            let #name = v8::Object::unwrap::<CPPGC_TAG, #native_struct_name>(scope, args.this())
                .expect("Not null");
            #name.as_ref().value.as_ptr().as_mut().expect("Not null")
        };
    }
}

pub fn make_assign_function(
    rust_function_name: &str,
    native_struct_name: &str,
    wrap_type: EWrappedStructType,
) -> TokenStream {
    let prefix = prefix(wrap_type);
    let native_struct_name = format!("{}{}", prefix, native_struct_name)
        .parse::<TokenStream>()
        .unwrap();
    let bind_function_name = rust_function_name.to_case(convert_case::Case::Camel);
    let rust_function_name = rust_function_name.parse::<TokenStream>().unwrap();
    quote::quote! {
        {
            let name = v8::String::new(scope, #bind_function_name).ok_or(Error::Other("Failed to create string".to_string()))?;
            let function = v8::FunctionTemplate::new(scope, #native_struct_name::#rust_function_name);
            prototype_template.set(name.into(), function.into());
        }
    }
}

pub fn make_param_list(params_len: usize) -> TokenStream {
    let mut stream: TokenStream = TokenStream::default();
    for index in 0..params_len {
        let variable_name = format!("arg_{index}").parse::<TokenStream>().unwrap();
        stream.extend(quote::quote! {
            #variable_name,
        });
    }
    stream
}

pub fn make_param(
    index: usize,
    param: &ra_ap_hir::Param,
    db: &RootDatabase,
) -> anyhow::Result<TokenStream> {
    let stream: TokenStream;
    let ty = param.ty();
    let variable_name = format!("arg_{index}").parse::<TokenStream>().unwrap();
    let arg_stream = format!("let arg_{index} = args.get({index});")
        .parse::<TokenStream>()
        .unwrap();
    match ty.as_builtin().map(|x| convet_builtin(x)) {
        Some(builtin) => match builtin {
            builtin_type::BuiltinType::Char => {
                return Err(anyhow!("Not support"));
            }
            builtin_type::BuiltinType::Bool => {
                stream = quote::quote! {
                    #arg_stream
                    if !#variable_name.is_boolean() {
                        return_exception(scope, &mut ret_val, &format!("args[{}] is not a boolean", #index));
                        return;
                    }
                    let #variable_name = #variable_name.to_boolean(scope).boolean_value(scope);
                };
            }
            builtin_type::BuiltinType::Str => {
                stream = quote::quote! {
                    #arg_stream
                    let Some(#variable_name) = #variable_name.to_string(scope).map(|x| x.to_rust_string_lossy(&mut scope)) else {
                        return_exception(scope, &mut ret_val, &format!("args[{}] is not a string", #index));
                        return;
                    };
                    let #variable_name = &#variable_name;
                };
            }
            builtin_type::BuiltinType::Int(builtin_int) => match builtin_int {
                builtin_type::BuiltinInt::Isize => {
                    stream = quote::quote! {
                        #arg_stream
                        let Some(#variable_name) = #variable_name.to_int32(scope).map(|x| x.value()) else {
                            return_exception(scope, &mut ret_val, &format!("args[{}] is not a number", #index));
                            return;
                        };
                        let #variable_name = #variable_name as isize;
                    };
                }
                builtin_type::BuiltinInt::I32 => {
                    stream = quote::quote! {
                        #arg_stream
                        let Some(#variable_name) = #variable_name.to_int32(scope).map(|x| x.value()) else {
                            return_exception(scope, &mut ret_val, &format!("args[{}] is not a number", #index));
                            return;
                        };
                    };
                }
                _ => {
                    return Err(anyhow!("Not support"));
                }
            },
            builtin_type::BuiltinType::Uint(builtin_uint) => match builtin_uint {
                builtin_type::BuiltinUint::Usize => {
                    stream = quote::quote! {
                        #arg_stream
                        let Some(#variable_name) = #variable_name.to_uint32(scope).map(|x| x.value()) else {
                            return_exception(scope, &mut ret_val, &format!("args[{}] is not a number", #index));
                            return;
                        };
                        let #variable_name = #variable_name as usize;
                    };
                }
                builtin_type::BuiltinUint::U32 => {
                    stream = quote::quote! {
                        #arg_stream
                        let Some(#variable_name) = #variable_name.to_uint32(scope).map(|x| x.value()) else {
                            return_exception(scope, &mut ret_val, &format!("args[{}] is not a number", #index));
                            return;
                        };
                    };
                }
                _ => {
                    return Err(anyhow!("Not support"));
                }
            },
            builtin_type::BuiltinType::Float(builtin_float) => match builtin_float {
                builtin_type::BuiltinFloat::F32 => {
                    stream = quote::quote! {
                        #arg_stream
                        let Some(#variable_name) = #variable_name.to_number(scope).map(|x| x.number_value(scope)).flatten() else {
                            return_exception(scope, &mut ret_val, &format!("args[{}] is not a number", #index));
                            return;
                        };
                        let #variable_name = #variable_name as f32;
                    };
                }
                builtin_type::BuiltinFloat::F64 => {
                    stream = quote::quote! {
                        #arg_stream
                        let Some(#variable_name) = #variable_name.to_number(scope).map(|x| x.number_value(scope)).flatten() else {
                            return_exception(scope, &mut ret_val, &format!("args[{}] is not a number", #index));
                            return;
                        };
                    };
                }
                _ => {
                    return Err(anyhow!("Not support"));
                }
            },
        },
        None => {
            // let krate = param.ty().krate(db);
            // let root_file = krate.root_file(db);
            if let Some(adt) = param.ty().as_adt() {
                if let Some(rs_enum) = adt.as_enum() {
                    let enum_name = rs_enum.name(db);
                    let enum_name = enum_name.symbol().to_string();
                    let enum_name_token = enum_name.parse::<TokenStream>().unwrap();

                    let variants = rs_enum.variants(db);
                    let mut list_token_stream = TokenStream::default();
                    for variant in variants.iter() {
                        match variant.kind(db) {
                            ra_ap_hir::StructKind::Unit => {}
                            _ => {
                                return Err(anyhow!("Not support"));
                            }
                        }
                    }
                    for (index, variant) in variants.iter().enumerate() {
                        let index = index as i32;
                        let variant_name_token =
                            variant.name(db).as_str().parse::<TokenStream>().unwrap();

                        list_token_stream.extend(quote::quote! {
                            #index => Some(#enum_name_token::#variant_name_token),
                        });
                    }
                    stream = quote::quote! {
                        #arg_stream
                        let Some(#variable_name) = #variable_name.to_int32(scope).map(|x| x.value()) else {
                            return_exception(scope, &mut ret_val, &format!("args[{}] is not a number", #index));
                            return;
                        };
                        let #variable_name = match #variable_name {
                            #list_token_stream
                            _ => {
                                None
                            },
                        };
                        let Some(#variable_name) = #variable_name else {
                            return_exception(scope, &mut ret_val, "Not a valid value");
                            return;
                        };
                    };

                    return Ok(stream);
                }
            }
            return Err(anyhow!("Not support"));
        }
    }
    Ok(stream)
}

fn prefix(wrap_type: EWrappedStructType) -> &'static str {
    let prefix = match wrap_type {
        EWrappedStructType::RcRefCell => "RcRef",
        EWrappedStructType::StaticLifeTimeNotNullPtr => "",
    };
    prefix
}

pub fn make_wrapped_struct(
    wrap_struct_name: &str,
    wrapped_value_type: &TokenStream,
    wrap_type: EWrappedStructType,
) -> TokenStream {
    let prefix = prefix(wrap_type);
    let wrap_struct_name = format!("{}{}", prefix, wrap_struct_name)
        .parse::<TokenStream>()
        .unwrap();
    let name = format!("c\"{}\"", wrap_struct_name)
        .parse::<TokenStream>()
        .unwrap();
    match wrap_type {
        EWrappedStructType::RcRefCell => {
            quote::quote! {
                #[derive(Clone)]
                pub struct #wrap_struct_name {
                    pub value: Rc<RefCell<#wrapped_value_type>>,
                }
                unsafe impl v8::cppgc::GarbageCollected for #wrap_struct_name {
                    fn trace(&self, _visitor: &mut v8::cppgc::Visitor) {}

                    fn get_name(&self) -> &'static std::ffi::CStr {
                        #name
                    }
                }
                impl Newable for #wrap_struct_name {
                    type AssociatedType = Rc<RefCell<#wrapped_value_type>>;

                    fn new(value: Self::AssociatedType) -> Self {
                        #wrap_struct_name { value }
                    }
                }
            }
        }
        EWrappedStructType::StaticLifeTimeNotNullPtr => {
            quote::quote! {
                #[derive(Clone)]
                pub struct #wrap_struct_name {
                    pub value: std::ptr::NonNull<#wrapped_value_type>,
                }
                unsafe impl v8::cppgc::GarbageCollected for #wrap_struct_name {
                    fn trace(&self, _visitor: &mut v8::cppgc::Visitor) {}

                    fn get_name(&self) -> &'static std::ffi::CStr {
                        #name
                    }
                }
                impl Newable for #wrap_struct_name {
                    type AssociatedType = std::ptr::NonNull<#wrapped_value_type>;

                    fn new(value: Self::AssociatedType) -> Self {
                        #wrap_struct_name { value }
                    }
                }
            }
        }
    }
}

pub fn make_api_code(
    wrap_struct_name: &str,
    wrapped_value_type: &str,
    wrapped_value_full_type: TokenStream,
    wrap_type: EWrappedStructType,
    set_function_bindings: TokenStream,
    function_bindings: TokenStream,
    out_binding_api_type_name: &mut TokenStream,
) -> TokenStream {
    let prefix = prefix(wrap_type);
    let wrapped_struct = make_wrapped_struct(wrap_struct_name, &wrapped_value_full_type, wrap_type);
    let wrap_struct_name_token_stream = format!("{}{}", prefix, wrap_struct_name)
        .parse::<TokenStream>()
        .unwrap();
    let binding_api_type_name = format!("{}{}BindingApi", prefix, wrapped_value_type)
        .parse::<TokenStream>()
        .unwrap();
    *out_binding_api_type_name = binding_api_type_name.clone();
    let export_wrap_struct_name = format!("{}{}", prefix, wrap_struct_name);
    match &wrap_type {
        EWrappedStructType::RcRefCell => {
            quote::quote! {
                pub struct #binding_api_type_name {
                    function_template: v8::Global<v8::FunctionTemplate>,
                }

                impl Constructible for #binding_api_type_name {
                    type AssociatedType = #wrap_struct_name_token_stream;

                    fn construct(v8_runtime: &mut rs_v8_host::v8_runtime::V8Runtime) -> Result<Constructor, Error> {
                        let main_context = v8_runtime.global_context.clone();
                        let isolate = &mut v8_runtime.isolate;
                        v8::scope_with_context!(context_scope, isolate, &main_context);
                        let scope = context_scope;
                        let native_class_function_template =
                            v8::FunctionTemplate::new(scope, #wrap_struct_name_token_stream::constructor_function);

                        native_class_function_template.set_class_name(
                            v8::String::new(scope, #export_wrap_struct_name)
                                .ok_or(Error::Other("Failed to create string".to_string()))?,
                        );
                        let prototype_template = native_class_function_template.prototype_template(scope);

                        #set_function_bindings

                        let function = native_class_function_template
                            .get_function(scope)
                            .ok_or(Error::Other("Function is null".to_string()))?;
                        let context = v8::Local::new(scope, main_context.clone());
                        let global_this = context.global(scope);
                        let name = v8::String::new(scope, #export_wrap_struct_name)
                            .ok_or(Error::Other("Failed to create string".to_string()))?;
                        global_this.set(scope, name.into(), function.into());

                        let function_template = v8::Global::new(scope, native_class_function_template);

                        Ok(Constructor {
                            name: #export_wrap_struct_name.to_string(),
                            function_template,
                        })
                    }
                }

                #wrapped_struct

                impl #wrap_struct_name_token_stream {
                    pub fn constructor_function(
                        scope: &mut v8::PinScope,
                        args: v8::FunctionCallbackArguments,
                        mut ret_val: v8::ReturnValue,
                    ) {

                    }

                    #function_bindings
                }
            }
        }
        EWrappedStructType::StaticLifeTimeNotNullPtr => {
            quote::quote! {
                pub struct #binding_api_type_name {
                    function_template: v8::Global<v8::FunctionTemplate>,
                }

                impl Constructible for #binding_api_type_name {
                    type AssociatedType = #wrap_struct_name_token_stream;

                    fn construct(v8_runtime: &mut rs_v8_host::v8_runtime::V8Runtime) -> Result<Constructor, Error> {
                        let main_context = v8_runtime.global_context.clone();
                        let isolate = &mut v8_runtime.isolate;
                        v8::scope_with_context!(context_scope, isolate, &main_context);
                        let scope = context_scope;
                        let native_class_function_template =
                            v8::FunctionTemplate::new(scope, #wrap_struct_name_token_stream::constructor_function);

                        native_class_function_template.set_class_name(
                            v8::String::new(scope, #export_wrap_struct_name)
                                .ok_or(Error::Other("Failed to create string".to_string()))?,
                        );
                        let prototype_template = native_class_function_template.prototype_template(scope);

                        #set_function_bindings

                        let function = native_class_function_template
                            .get_function(scope)
                            .ok_or(Error::Other("Function is null".to_string()))?;
                        let context = v8::Local::new(scope, main_context.clone());
                        let global_this = context.global(scope);
                        let name = v8::String::new(scope, #export_wrap_struct_name)
                            .ok_or(Error::Other("Failed to create string".to_string()))?;
                        global_this.set(scope, name.into(), function.into());

                        let function_template = v8::Global::new(scope, native_class_function_template);

                        Ok(Constructor {
                            name: #export_wrap_struct_name.to_string(),
                            function_template,
                        })
                    }
                }

                #wrapped_struct

                impl #wrap_struct_name_token_stream {
                    pub fn constructor_function(
                        scope: &mut v8::PinScope,
                        args: v8::FunctionCallbackArguments,
                        mut ret_val: v8::ReturnValue,
                    ) {

                    }

                    #function_bindings
                }
            }
        }
    }
}

pub fn resolve_struct_import_path(db: &RootDatabase, rs_struct: &ra_ap_hir::Struct) -> String {
    let name = rs_struct.name(db);
    let module = rs_struct.module(db);
    let krate = module.krate(db);
    let mut modules = vec![];
    let mut current_module = Some(module.clone());
    while let Some(module) = current_module {
        modules.push(module);
        current_module = module.parent(db);
    }
    let crate_name = krate
        .display_name(db)
        .map(|x| x.crate_name().symbol().as_str().to_string());
    let mut module_chain = modules
        .iter()
        .flat_map(|x| x.name(db).map(|x| x.as_str().to_string()))
        .collect::<Vec<String>>();
    module_chain.reverse();
    if let Some(crate_name) = crate_name {
        module_chain.insert(0, crate_name);
    }
    module_chain.push(name.as_str().to_string());
    module_chain.join("::")
}

pub fn resolve_struct_import_path_ident(
    db: &RootDatabase,
    rs_struct: &ra_ap_hir::Struct,
) -> TokenStream {
    let path = resolve_struct_import_path(db, rs_struct);
    path.parse().unwrap()
}

pub fn make_return_value_expr(
    db: &RootDatabase,
    return_type: &ra_ap_hir::Type,
) -> anyhow::Result<TokenStream> {
    let _ = db;
    let is_unit = return_type.is_unit();
    if is_unit {
        return Ok(TokenStream::default());
    }
    if let Some(as_builtin) = return_type.as_builtin() {
        match convet_builtin(as_builtin) {
            ra_ap_hir_def::builtin_type::BuiltinType::Char => {}
            ra_ap_hir_def::builtin_type::BuiltinType::Bool => {
                return Ok(quote::quote! {
                    ret_val.set(v8::Boolean::new(scope, return_value).into());
                });
            }
            ra_ap_hir_def::builtin_type::BuiltinType::Str => {
                return Ok(quote::quote! {
                    ret_val.set(v8::String::new(scope, return_value).expect("Not null").into());
                });
            }
            ra_ap_hir_def::builtin_type::BuiltinType::Int(builtin_int) => match builtin_int {
                builtin_type::BuiltinInt::I32 => {
                    return Ok(quote::quote! {
                        ret_val.set(v8::Integer::new(scope, return_value).into());
                    });
                }
                _ => {}
            },
            ra_ap_hir_def::builtin_type::BuiltinType::Uint(builtin_uint) => match builtin_uint {
                builtin_type::BuiltinUint::U32 => {
                    return Ok(quote::quote! {
                        ret_val.set(v8::Integer::new_from_unsigned(scope, return_value).into());
                    });
                }
                _ => {}
            },
            ra_ap_hir_def::builtin_type::BuiltinType::Float(builtin_float) => match builtin_float {
                builtin_type::BuiltinFloat::F64 => {
                    quote::quote! {
                        ret_val.set(v8::Number::new(scope, return_value).into());
                    };
                }
                _ => {}
            },
        }
    } else if let Some(as_adt) = return_type.as_adt() {
        match as_adt {
            ra_ap_hir::Adt::Struct(rs_struct) => {
                if resolve_struct_import_path(db, &rs_struct) == "alloc::string::String" {
                    return Ok(quote::quote! {
                        ret_val.set(v8::String::new(scope, &return_value).expect("Not null").into());
                    });
                }
            }
            _ => {}
        }
    }
    return Err(anyhow!("Not support"));
}

pub fn is_impl_clone(ty: &ra_ap_hir::Type, db: &dyn ra_ap_hir::db::HirDatabase) -> bool {
    let krate = ty.krate(db);
    let interner = ra_ap_hir::next_solver::DbInterner::new_with(db, krate.into());
    let clone_trait = interner.lang_items().Clone.expect("A valid TraitId");
    // let lang_item = db.lang_item(krate.into(), ra_ap_hir::LangItem::Clone);
    // let clone_trait = match lang_item {
    //     Some(ra_ap_hir_def::lang_item::LangItemTarget::Trait(it)) => it,
    //     _ => return false,
    // };
    return ty.impls_trait(db, clone_trait.into(), &[]);
}

pub fn readable_type_description(
    ty: &ra_ap_hir::Type,
    db: &dyn ra_ap_hir::db::HirDatabase,
) -> String {
    let krate = ty.krate(db);
    let display = ty.display(db, DisplayTarget::from_crate(db, krate.into()));
    ra_ap_syntax::ToSmolStr::to_smolstr(&display).to_string()
}
