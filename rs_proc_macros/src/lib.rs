mod multiple_thread_functions_generator;
mod shader;
mod string_extension;
mod token_stream_extension;
mod uniform;

use multiple_thread_functions_generator::multiple_thread_functions_generator_macro_derive_impl;
use proc_macro::TokenStream;
use shader::global_shader_macro_derive_impl;
use std::io::Write;
use uniform::shader_uniform_macro_impl;

pub(crate) fn get_engine_root_dir_at_compile_time() -> std::path::PathBuf {
    std::path::Path::new(file!())
        .join("../../../")
        .to_path_buf()
}

pub(crate) fn debug_log<S: AsRef<str>>(file_name: S, content: S) {
    let path = get_engine_root_dir_at_compile_time()
        .join(format!("rs_proc_macros/target/{}.log", file_name.as_ref()));
    let f = std::fs::File::options()
        .create(true)
        .append(true)
        .open(path);
    if let Ok(mut f) = f {
        let _ = f.write_all(format!("{}\n", content.as_ref()).as_bytes());
        let _ = f.flush();
    }
}

#[proc_macro_derive(MultipleThreadFunctionsGenerator, attributes(file, ignore_functions))]
pub fn multiple_thread_functions_generator_macro_derive(input: TokenStream) -> TokenStream {
    multiple_thread_functions_generator_macro_derive_impl(input)
}

#[proc_macro_derive(GlobalShader, attributes(file, defines, include_dirs))]
pub fn global_shader_macro_derive(input: TokenStream) -> TokenStream {
    global_shader_macro_derive_impl(input)
}

#[proc_macro]
pub fn shader_uniform(input: TokenStream) -> TokenStream {
    shader_uniform_macro_impl(input)
}
