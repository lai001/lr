use crate::ui::material_view::MaterialNode;
use egui_snarl::Snarl;
use rs_engine::url_extension::UrlExtension;
use rs_foundation::new::SingleThreadMutType;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug)]
struct MaterialRuntime {
    material: SingleThreadMutType<rs_engine::content::material::Material>,
}

#[derive(Serialize, Deserialize)]
pub struct Material {
    pub url: url::Url,
    pub snarl: Snarl<MaterialNode>,

    #[serde(skip)]
    run_time: Option<MaterialRuntime>,
}

impl Material {
    pub fn new(url: url::Url, snarl: Snarl<MaterialNode>) -> Material {
        Material {
            url,
            snarl,
            run_time: None,
        }
    }

    pub fn set_associated_material(
        &mut self,
        material: SingleThreadMutType<rs_engine::content::material::Material>,
    ) {
        if let Some(runtime) = self.run_time.as_mut() {
            runtime.material = material;
        } else {
            self.run_time = Some(MaterialRuntime { material });
        }
    }

    pub fn get_associated_material(
        &self,
    ) -> Option<SingleThreadMutType<rs_engine::content::material::Material>> {
        if let Some(runtime) = self.run_time.as_ref() {
            Some(runtime.material.clone())
        } else {
            None
        }
    }

    pub fn set_name(&mut self, new_name: String) {
        self.url.set_name_in_editor(new_name);
    }

    pub fn on_url_changed(
        material: &mut rs_engine::content::material::Material,
        asset: &mut Material,
    ) {
        let new_url = Self::make_url(&material.url);
        material.asset_url = new_url.clone();
        asset.url = new_url;
    }

    pub fn make_url(material_url: &url::Url) -> url::Url {
        assert!(material_url.as_str().starts_with(&format!(
            "{}://{}",
            rs_engine::CONTENT_SCHEME,
            rs_engine::CONTENT_ROOT
        )));
        let new = material_url.to_string().replace(
            &format!(
                "{}://{}",
                rs_engine::CONTENT_SCHEME,
                rs_engine::CONTENT_ROOT
            ),
            &format!(
                "{}://{}/material",
                rs_engine::ASSET_SCHEME,
                rs_engine::ASSET_ROOT
            ),
        );
        url::Url::parse(&new).expect("Valid url")
    }
}
