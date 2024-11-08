use crate::{
    engine::Engine,
    handle::BufferHandle,
    uniform_map::{StructField, UniformMap},
    url_extension::UrlExtension,
};
use rs_artifact::{asset::Asset, resource_type::EResourceType};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct MaterialParamentersCollection {
    pub url: url::Url,
    pub fields: Vec<StructField>,

    #[serde(skip)]
    uniform_map: Option<UniformMap>,
    #[serde(skip)]
    buffer_handle: Option<crate::handle::BufferHandle>,
}

impl MaterialParamentersCollection {
    pub fn new(url: url::Url) -> MaterialParamentersCollection {
        MaterialParamentersCollection {
            url,
            fields: vec![],
            uniform_map: None,
            buffer_handle: None,
        }
    }

    pub fn get_name(&self) -> String {
        self.url.get_name_in_editor()
    }

    pub fn replace_field_name_with(&mut self, old_name: &str, new_name: &str) -> bool {
        if !rs_core_minimal::misc::is_valid_name(new_name) {
            return false;
        }
        let find_field = self.fields.iter_mut().find(|x| x.name == old_name);
        let Some(find_field) = find_field else {
            return false;
        };
        find_field.name = new_name.to_string();
        true
    }

    pub fn add_field_force(&mut self, field: StructField) {
        self.fields.push(field);
    }

    pub fn add_field(&mut self, field: StructField, engine: &mut Engine) {
        self.fields.push(field);
        self.initialize(engine);
    }

    pub fn delete_field_by_index(&mut self, index: usize, engine: &mut Engine) {
        if self.fields.get(index).is_none() {
            return;
        }
        self.fields.remove(index);
        self.initialize(engine);
    }

    pub fn initialize(&mut self, engine: &mut Engine) {
        self.uniform_map = Some(UniformMap::new(&self.fields));
        let uniform_map = self.uniform_map.as_ref().unwrap();
        let name = self.get_name();
        self.buffer_handle = Some(engine.create_buffer(
            uniform_map.get_data(),
            wgpu::BufferUsages::all(),
            Some(name.as_ref()),
        ));
    }

    pub fn update(&self, engine: &mut Engine) {
        let Some(buffer_handle) = &self.buffer_handle else {
            return;
        };
        let Some(uniform_map) = &self.uniform_map else {
            return;
        };
        engine.update_buffer(buffer_handle.clone(), uniform_map.get_data());
    }

    pub fn get_field_value_as_f32(&mut self, name: &str) -> Option<f32> {
        let Some(uniform_map) = self.uniform_map.as_mut() else {
            return None;
        };
        uniform_map.get_field_value_as_f32(name)
    }

    pub fn get_field_value_as_vec2(&mut self, name: &str) -> Option<glam::Vec2> {
        let Some(uniform_map) = self.uniform_map.as_mut() else {
            return None;
        };
        uniform_map.get_field_value_as_vec2(name)
    }

    pub fn get_field_value_as_vec3(&mut self, name: &str) -> Option<glam::Vec3> {
        let Some(uniform_map) = self.uniform_map.as_mut() else {
            return None;
        };
        uniform_map.get_field_value_as_vec3(name)
    }

    pub fn get_field_value_as_vec4(&mut self, name: &str) -> Option<glam::Vec4> {
        let Some(uniform_map) = self.uniform_map.as_mut() else {
            return None;
        };
        uniform_map.get_field_value_as_vec4(name)
    }

    pub fn set_field_f32_value(&mut self, name: &str, value: f32) -> bool {
        let Some(uniform_map) = self.uniform_map.as_mut() else {
            return false;
        };
        uniform_map.set_field_f32_value(name, value)
    }

    pub fn set_field_vec2_value(&mut self, name: &str, value: glam::Vec2) -> bool {
        let Some(uniform_map) = self.uniform_map.as_mut() else {
            return false;
        };
        uniform_map.set_field_vec2_value(name, value)
    }

    pub fn set_field_vec3_value(&mut self, name: &str, value: glam::Vec3) -> bool {
        let Some(uniform_map) = self.uniform_map.as_mut() else {
            return false;
        };
        uniform_map.set_field_vec3_value(name, value)
    }

    pub fn set_field_vec4_value(&mut self, name: &str, value: glam::Vec4) -> bool {
        let Some(uniform_map) = self.uniform_map.as_mut() else {
            return false;
        };
        uniform_map.set_field_vec4_value(name, value)
    }

    pub fn get_buffer_handle(&self) -> Option<BufferHandle> {
        match &self.buffer_handle {
            Some(buffer_handle) => Some(buffer_handle.clone()),
            None => None,
        }
    }
}

impl Asset for MaterialParamentersCollection {
    fn get_url(&self) -> url::Url {
        self.url.clone()
    }

    fn get_resource_type(&self) -> EResourceType {
        EResourceType::Content(
            rs_artifact::content_type::EContentType::MaterialParamentersCollection,
        )
    }
}
