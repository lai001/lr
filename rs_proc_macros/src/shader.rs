use crate::{
    debug_log, get_engine_root_dir_at_compile_time, string_extension::StringExtension,
    token_stream_extension::PrettyPrintStream,
};
use anyhow::anyhow;
use path_slash::PathBufExt;
use proc_macro::TokenStream;
use quote::quote;
use spanned::Spanned;
use syn::*;

#[derive(Debug)]
struct FileParams {
    file: std::path::PathBuf,
}

impl parse::Parse for FileParams {
    fn parse(input: parse::ParseStream) -> Result<Self> {
        let args_parsed =
            syn::punctuated::Punctuated::<syn::LitStr, syn::Token![,]>::parse_terminated(input)?;
        if args_parsed.len() < 1 {
            let token: Ident = input.parse()?;
            return Err(Error::new(token.span(), "Too few arguments specified."));
        }
        let mut args_parsed = args_parsed
            .into_iter()
            .map(|x| x.token().to_string().trim_quote())
            .collect::<Vec<String>>();
        let file = args_parsed.remove(0);

        Ok(FileParams {
            file: get_engine_root_dir_at_compile_time().join(file),
        })
    }
}

#[derive(Debug)]
struct DefinesParams {
    defines: Vec<String>,
}

impl parse::Parse for DefinesParams {
    fn parse(input: parse::ParseStream) -> Result<Self> {
        let args_parsed =
            syn::punctuated::Punctuated::<syn::LitStr, syn::Token![,]>::parse_terminated(input)?;
        let defines = args_parsed
            .into_iter()
            .map(|x| x.token().to_string().trim_quote())
            .collect::<Vec<String>>();
        Ok(DefinesParams { defines })
    }
}

#[derive(Debug)]
struct IncludeDirsParams {
    dirs: Vec<std::path::PathBuf>,
}

impl parse::Parse for IncludeDirsParams {
    fn parse(input: parse::ParseStream) -> Result<Self> {
        let args_parsed =
            syn::punctuated::Punctuated::<syn::LitStr, syn::Token![,]>::parse_terminated(input)?;
        let dirs = args_parsed
            .into_iter()
            .map(|x| x.token().to_string().trim_quote())
            .collect::<Vec<String>>();
        let dirs = dirs
            .iter()
            .map(|x| std::path::Path::new(x).to_path_buf())
            .collect();
        Ok(IncludeDirsParams { dirs })
    }
}

fn pre_process(
    file_parameters: &FileParams,
    defines_parameters: &DefinesParams,
    include_dirs_params: &IncludeDirsParams,
) -> anyhow::Result<String> {
    let mut clang = std::process::Command::new("clang");
    clang.arg("-E");
    clang.arg("-P");
    clang.arg("-x");
    clang.arg("c");
    clang.arg("-std=c11");
    for include_dir in include_dirs_params.dirs.iter() {
        let resolve_dir = get_engine_root_dir_at_compile_time().join(include_dir);
        let resolve_dir = dunce::canonicalize(resolve_dir)?;
        let resolve_dir = resolve_dir.to_slash_lossy();
        clang.arg(format!("-I{}", resolve_dir));
    }
    for definition in defines_parameters.defines.iter() {
        if definition.contains("=") {
            clang.arg(format!("-D{}", definition));
        } else {
            clang.arg(format!("-D{}", definition));
        }
    }
    let file_path = &file_parameters.file;
    let path_arg = file_path
        .to_slash()
        .ok_or(anyhow!("Not a valid path, {file_path:?}"))?
        .to_string();
    clang.arg(path_arg);
    let output = clang.output()?;
    let stderr = String::from_utf8(output.stderr)?;
    let stdout = String::from_utf8(output.stdout)?;
    if output.status.success() {
        Ok(stdout.to_string())
    } else {
        return Err(anyhow!(stderr));
    }
}

pub(crate) fn global_shader_macro_derive_impl(input: TokenStream) -> TokenStream {
    let ast: DeriveInput = syn::parse(input).unwrap();
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
        .expect("file attribute required for deriving GlobalShader!");
    let file_parameters = file_attribute.parse_args::<FileParams>().unwrap();

    let defines_parameters: DefinesParams = match ast.attrs.iter().find(|x| {
        x.path()
            .segments
            .iter()
            .find(|seg| seg.ident == "defines")
            .is_some()
    }) {
        Some(attribute) => attribute.parse_args::<DefinesParams>().unwrap(),
        None => DefinesParams { defines: vec![] },
    };

    let include_dirs_params: IncludeDirsParams = match ast.attrs.iter().find(|x| {
        x.path()
            .segments
            .iter()
            .find(|seg| seg.ident == "include_dirs")
            .is_some()
    }) {
        Some(attribute) => attribute.parse_args::<IncludeDirsParams>().unwrap(),
        None => IncludeDirsParams { dirs: vec![] },
    };

    let shader_source = pre_process(&file_parameters, &defines_parameters, &include_dirs_params);
    let shader_source = match shader_source {
        Ok(shader_source) => shader_source,
        Err(err) => {
            return syn::Error::new(ast.span(), err.to_string())
                .to_compile_error()
                .into();
        }
    };

    let module = naga::front::wgsl::parse_str(&shader_source).unwrap();

    let output_stream = quote! {};
    debug_log("shader_module", &format!("module: \n{:#?}", module));
    {
        let pretty_string = output_stream.to_pretty_string();
        let path =
            get_engine_root_dir_at_compile_time().join("rs_proc_macros/target/global_shader.txt");
        if path.exists() {
            let _ = std::fs::remove_file(path.clone());
        }
        let _ = std::fs::write(path, pretty_string);
    }
    output_stream.into()
}
