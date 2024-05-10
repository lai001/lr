use crate::{build_content_file_url, property, url_extension::UrlExtension};
use rs_artifact::{
    asset::Asset, property_value_type::EPropertyValueType, resource_type::EResourceType,
};
use serde::{Deserialize, Serialize};
use std::{cell::RefCell, collections::HashMap};

#[derive(Serialize, Deserialize, Debug)]
pub struct Level {
    pub url: url::Url,
    pub actors: Vec<std::rc::Rc<RefCell<crate::actor::Actor>>>,
}

impl Asset for Level {
    fn get_url(&self) -> url::Url {
        self.url.clone()
    }

    fn get_resource_type(&self) -> EResourceType {
        EResourceType::Content(rs_artifact::content_type::EContentType::Level)
    }
}

impl Level {
    pub fn empty_level() -> Self {
        Self {
            actors: vec![],
            url: build_content_file_url("Empty").unwrap(),
        }
    }

    pub fn get_name(&self) -> String {
        self.url.get_name_in_editor()
    }
}

pub fn default_node3d_properties() -> HashMap<String, EPropertyValueType> {
    HashMap::from([
        (
            property::name::TEXTURE.to_string(),
            EPropertyValueType::Texture(None),
        ),
        (
            property::name::SCALE.to_string(),
            EPropertyValueType::Vec3(glam::Vec3::ONE),
        ),
        (
            property::name::TRANSLATION.to_string(),
            EPropertyValueType::Vec3(glam::Vec3::ZERO),
        ),
        (
            property::name::ROTATION.to_string(),
            EPropertyValueType::Quat(glam::Quat::IDENTITY),
        ),
    ])
}
