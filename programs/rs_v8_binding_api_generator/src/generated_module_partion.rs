use proc_macro2::TokenStream;

pub struct GeneratedModulePartion {
    pub code: TokenStream,
    pub binding_api_types: Vec<TokenStream>,
}
