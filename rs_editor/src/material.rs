use crate::ui::material_view::MaterialNode;
use egui_snarl::Snarl;
use rs_foundation::new::SingleThreadMutType;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug)]
struct MaterialRuntime {
    material: SingleThreadMutType<rs_engine::content::material::Material>,
}

#[derive(Serialize, Deserialize, Debug)]
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
}
