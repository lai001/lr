use proc_macro::TokenStream;
use quote::quote;

pub fn plugin_project_file_path_macro_impl(input: TokenStream) -> TokenStream {
    let mut project_file_path_str = String::new();
    let program = input.to_string();
    match program.as_str() {
        "rs_editor" => {
            let plugin_infos = crate::load_plugin::fetch_plugin_info(&program);
            for plugin_info in plugin_infos {
                if let Some(project_file_path) =
                    plugin_info.project_path.to_str().map(|x| x.to_string())
                {
                    project_file_path_str = project_file_path;
                }
            }
        }
        _ => {}
    }

    let final_output = if project_file_path_str.is_empty() {
        quote! {
            None::<String>
        }
    } else {
        quote! {
            Some(#project_file_path_str.to_string())
        }
    };

    final_output.into()
}
