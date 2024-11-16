pub trait PrettyPrintStream {
    fn to_pretty_string(&self) -> anyhow::Result<String>;
}

impl PrettyPrintStream for proc_macro2::TokenStream {
    fn to_pretty_string(&self) -> anyhow::Result<String> {
        let file = syn::parse_file(&self.to_string())?;
        Ok(prettyplease::unparse(&file))
    }
}
