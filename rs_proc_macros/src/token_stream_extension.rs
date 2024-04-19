pub(crate) trait PrettyPrintStream {
    fn to_pretty_string(&self) -> String;
}

impl PrettyPrintStream for proc_macro2::TokenStream {
    fn to_pretty_string(&self) -> String {
        let file = syn::parse_file(&self.to_string()).unwrap();
        prettyplease::unparse(&file)
    }
}
