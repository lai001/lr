use crate::ui::material_view::MaterialNode;
use egui_snarl::Snarl;
use rs_artifact::material_paramenters::{BaseDataValueType, StructField};
use rs_core_minimal::name_generator::make_unique_name;
use rs_engine::url_extension::UrlExtension;
use rs_foundation::new::SingleThreadMutType;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug)]
struct MaterialRuntime {
    material: SingleThreadMutType<rs_engine::content::material::Material>,
}

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct Paramenters {
    fields: Vec<StructField>,
}

impl Paramenters {
    pub fn empty() -> Self {
        Self { fields: vec![] }
    }

    pub fn add(&mut self, name: String, data_type: BaseDataValueType) -> bool {
        if name.is_empty() {
            return false;
        }
        let name = make_unique_name(self.fields.iter().map(|x| x.name.clone()).collect(), name);
        let field = StructField { name, data_type };
        self.fields.push(field);
        true
    }

    pub fn fields(&self) -> &[StructField] {
        &self.fields
    }

    pub fn remove(&mut self, name: &str) -> bool {
        let num = self.fields.len();
        self.fields.retain(|x| x.name != name);
        num != self.fields.len()
    }

    pub fn fields_iter_mut(&mut self) -> std::slice::IterMut<'_, StructField> {
        self.fields.iter_mut()
    }

    pub fn change_type(&mut self, name: &str, new_type: BaseDataValueType) -> bool {
        for field in &mut self.fields {
            if field.name == name {
                field.data_type = new_type;
                return true;
            }
        }
        return false;
    }

    pub fn change_name(&mut self, old_name: &str, new_name: &str) -> bool {
        let unique_name = make_unique_name(
            self.fields.iter().map(|x| x.name.clone()).collect(),
            new_name,
        );
        for field in &mut self.fields {
            if field.name == old_name {
                field.name = unique_name;
                return true;
            }
        }
        return false;
    }

    pub fn is_valid(&self) -> bool {
        !self.fields.is_empty()
    }
}

#[derive(Serialize, Deserialize)]
pub struct Material {
    pub url: url::Url,
    pub snarl: Snarl<MaterialNode>,
    pub paramenters: Paramenters,

    #[serde(skip)]
    run_time: Option<MaterialRuntime>,
}

impl Material {
    pub fn new(url: url::Url, snarl: Snarl<MaterialNode>) -> Material {
        Material {
            url,
            snarl,
            run_time: None,
            paramenters: Paramenters::empty(),
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
