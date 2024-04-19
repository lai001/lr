use crate::{get_engine_root_dir_at_compile_time, string_extension::StringExtension};
use proc_macro::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::*;

struct MultipleThreadFunctionsGeneratorFileParams {
    file: std::path::PathBuf,
    target_struct_name: String,
}

impl parse::Parse for MultipleThreadFunctionsGeneratorFileParams {
    fn parse(input: parse::ParseStream) -> Result<Self> {
        let args_parsed =
            syn::punctuated::Punctuated::<syn::LitStr, syn::Token![,]>::parse_terminated(input)?;
        if args_parsed.len() < 2 {
            let token: Ident = input.parse()?;
            return Err(Error::new(token.span(), "Too few arguments specified."));
        }
        let mut args_parsed = args_parsed
            .into_iter()
            .map(|x| x.token().to_string().trim_quote())
            .collect::<Vec<String>>();
        let file = args_parsed.remove(0);
        let target_struct_name = args_parsed.remove(0);
        Ok(MultipleThreadFunctionsGeneratorFileParams {
            file: get_engine_root_dir_at_compile_time().join(file),
            target_struct_name,
        })
    }
}

struct MultipleThreadFunctionsGeneratorIgnoreFunctionsParams {
    functions: Vec<String>,
}

impl parse::Parse for MultipleThreadFunctionsGeneratorIgnoreFunctionsParams {
    fn parse(input: parse::ParseStream) -> Result<Self> {
        let args_parsed =
            syn::punctuated::Punctuated::<syn::LitStr, syn::Token![,]>::parse_terminated(input)?;
        let functions = args_parsed
            .into_iter()
            .map(|x| x.token().to_string().trim_quote())
            .collect::<Vec<String>>();
        Ok(MultipleThreadFunctionsGeneratorIgnoreFunctionsParams { functions })
    }
}

pub fn multiple_thread_functions_generator_macro_derive_impl(input: TokenStream) -> TokenStream {
    let ast: DeriveInput = syn::parse(input).unwrap();
    let self_struct_name = &ast.ident;
    let file_attribute = ast
        .attrs
        .iter()
        .find(|x| {
            x.path()
                .segments
                .iter()
                .find(|seg| seg.ident == "file")
                .is_some()
        })
        .expect("file attribute required for deriving MultipleThreadFunctionsGenerator!");
    let file_parameters = file_attribute
        .parse_args::<MultipleThreadFunctionsGeneratorFileParams>()
        .unwrap();

    let ignore_functions_attribute = ast.attrs.iter().find(|x| {
        x.path()
            .segments
            .iter()
            .find(|seg| seg.ident == "ignore_functions")
            .is_some()
    });
    let mut ignore_functions =
        MultipleThreadFunctionsGeneratorIgnoreFunctionsParams { functions: vec![] };
    if let Some(ignore_functions_attribute) = &ignore_functions_attribute {
        ignore_functions = ignore_functions_attribute
            .parse_args::<MultipleThreadFunctionsGeneratorIgnoreFunctionsParams>()
            .unwrap();
    }

    if !file_parameters.file.exists() {
        panic!("{:?} is not exists.", file_parameters.file);
    }

    let content = std::fs::read_to_string(&file_parameters.file).unwrap();
    let read_ast = syn::parse_file(&content).unwrap();

    let mut func_stream = proc_macro2::TokenStream::default();

    for item in read_ast.items.iter() {
        let Item::Impl(target_impl) = item else {
            continue;
        };
        let Type::Path(path) = &*target_impl.self_ty else {
            continue;
        };
        let Some(segment) = path.path.segments.first() else {
            continue;
        };
        if segment.ident.to_string() != file_parameters.target_struct_name {
            continue;
        }
        for item in target_impl.items.iter() {
            let ImplItem::Fn(method) = item else {
                continue;
            };
            let func_name = method.sig.ident.to_string();
            let generics = method.sig.generics.to_token_stream();

            let input_args = &method.sig.inputs;
            let input_args = input_args
                .iter()
                .filter(|x| match x {
                    FnArg::Receiver(_) => false,
                    FnArg::Typed(_) => true,
                })
                .cloned()
                .collect::<Vec<syn::FnArg>>();

            if ignore_functions.functions.contains(&func_name) {
                continue;
            }
            let output_name = match &method.sig.output {
                ReturnType::Default => proc_macro2::TokenStream::default(),
                ReturnType::Type(_, return_type) => {
                    let t = return_type.to_token_stream();
                    quote! {
                        -> #t
                    }
                }
            };

            let func_name = format_ident!("{}", func_name);
            let mut input_args_token = proc_macro2::TokenStream::default();
            let mut call_input_args_token = proc_macro2::TokenStream::default();
            for input_arg in input_args {
                input_args_token.extend::<proc_macro2::TokenStream>(quote! {
                    #input_arg,
                });
                let FnArg::Typed(typed) = input_arg else {
                    continue;
                };
                let typed = typed.pat.to_token_stream().to_string();
                let typed = format_ident!("{}", typed);
                call_input_args_token.extend::<proc_macro2::TokenStream>(quote! {
                    #typed,
                });
            }

            func_stream.extend::<proc_macro2::TokenStream>(quote! {
                pub fn #func_name #generics(&self, #input_args_token) #output_name {
                    self.inner.lock().unwrap().#func_name(#call_input_args_token)
                }
            });
        }
    }

    let output_stream = quote! {
        impl #self_struct_name {
           #func_stream
        }
    };
    // println!(
    //     "Output token stream: \n{}",
    //     output_stream.to_pretty_string()
    // );

    output_stream.into()
}
