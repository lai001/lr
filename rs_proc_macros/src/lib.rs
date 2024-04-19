mod multiple_thread_functions_generator;
mod shader;
mod string_extension;
mod token_stream_extension;

use multiple_thread_functions_generator::multiple_thread_functions_generator_macro_derive_impl;
use proc_macro::TokenStream;
use shader::global_shader_macro_derive_impl;
use std::io::Write;

pub(crate) fn get_engine_root_dir_at_compile_time() -> std::path::PathBuf {
    std::path::Path::new(file!())
        .join("../../../")
        .to_path_buf()
}

pub(crate) fn debug_log<S: AsRef<str>>(content: S) {
    let path = get_engine_root_dir_at_compile_time().join("rs_proc_macros/target/debug.log");
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
