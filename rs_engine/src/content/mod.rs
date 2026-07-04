pub mod blend_animations;
pub mod content_file_type;
pub mod curve;
pub mod ibl;
pub mod level;
pub mod material;
pub mod material_paramenters_collection;
pub mod media_source;
pub mod particle_system;
pub mod render_target_2d;
pub mod skeleton;
pub mod skeleton_animation;
pub mod skeleton_mesh;
pub mod sound;
pub mod static_mesh;
pub mod texture;

#[macro_export(local_inner_macros)]
macro_rules! impl_content {
    ($type_name:ty) => {
        impl rs_core_minimal::types::HasUrl for $type_name {
            fn get_url(&self) -> url::Url {
                self.url.clone()
            }
        }

        #[typetag::serde(name = ::core::stringify!($type_name))]
        impl rs_content::Content for $type_name {
            fn get_name(&self) -> String {
                crate::url_extension::UrlExtension::get_name_in_editor(&self.url)
            }

            fn set_name(&mut self, new_name: String) {
                crate::url_extension::UrlExtension::set_name_in_editor(&mut self.url, new_name);
            }

            fn get_type_text(&self) -> &'static str {
                ::core::stringify!($type_name)
            }
        }
    };
}
