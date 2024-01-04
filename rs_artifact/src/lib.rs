pub mod artifact;
pub mod asset;
pub mod error;
pub mod file_header;
pub mod image;
pub mod mesh_vertex;
pub mod mesh_vertex_visitor;
pub mod resource_info;
pub mod resource_type;
pub mod shader_source_code;
pub mod static_mesh;
pub mod type_expected;

#[cfg(target_os = "android")]
pub mod java_input_stream;

pub fn default_url() -> &'static url::Url {
    static URL: std::sync::OnceLock<url::Url> = std::sync::OnceLock::new();
    URL.get_or_init(|| url::Url::parse("rs://").unwrap())
}

pub fn build_asset_url(
    name: &str,
    resource_type: resource_type::EResourceType,
) -> Result<url::Url, url::ParseError> {
    url::Url::parse(&format!("rs://Asset/{:?}/{}", resource_type, name))
}

#[derive(Debug, Clone, Copy)]
pub enum EEndianType {
    Big,
    Little,
    Native,
}
