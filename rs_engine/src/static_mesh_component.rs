use crate::{
    content::content_file_type::EContentFileType, drawable::EDrawObjectType, engine::Engine,
    resource_manager::ResourceManager,
};
use rs_artifact::static_mesh::StaticMesh;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Clone, Debug)]
struct StaticMeshComponentRuntime {
    draw_objects: EDrawObjectType,
    _mesh: Arc<StaticMesh>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StaticMeshComponent {
    pub name: String,
    pub static_mesh: Option<url::Url>,
    pub transformation: glam::Mat4,
    pub material_url: Option<url::Url>,

    #[serde(skip)]
    run_time: Option<StaticMeshComponentRuntime>,
}

impl StaticMeshComponent {
    pub fn get_interactive_transformation(&mut self) -> &mut glam::Mat4 {
        &mut self.transformation
    }

    pub fn new(
        name: String,
        static_mesh_url: Option<url::Url>,
        material_url: Option<url::Url>,
        transformation: glam::Mat4,
    ) -> StaticMeshComponent {
        StaticMeshComponent {
            name,
            transformation,
            material_url,
            run_time: None,
            static_mesh: static_mesh_url,
        }
    }

    pub fn initialize(
        &mut self,
        resource_manager: ResourceManager,
        engine: &mut Engine,
        files: &[EContentFileType],
    ) {
        let mut find_static_mesh: Option<Arc<StaticMesh>> = None;

        for file in files {
            if let EContentFileType::StaticMesh(mesh) = file {
                let mesh = mesh.borrow();
                if Some(mesh.url.clone()) == self.static_mesh {
                    find_static_mesh = resource_manager
                        .get_static_mesh(&mesh.asset_info.get_url())
                        .ok();
                    break;
                }
            }
        }

        let material = if let Some(material_url) = &self.material_url {
            files.iter().find_map(|x| {
                if let EContentFileType::Material(content_material) = x {
                    if &content_material.borrow().url == material_url {
                        return Some(content_material.clone());
                    }
                }
                None
            })
        } else {
            None
        };

        if let Some(find_static_mesh) = find_static_mesh {
            let draw_object: EDrawObjectType;
            if let Some(material) = material.clone() {
                draw_object = engine.create_material_draw_object_from_static_mesh(
                    &find_static_mesh.vertexes,
                    &find_static_mesh.indexes,
                    Some(find_static_mesh.name.clone()),
                    material,
                );
            } else {
                draw_object = engine.create_draw_object_from_static_mesh(
                    &find_static_mesh.vertexes,
                    &find_static_mesh.indexes,
                    Some(find_static_mesh.name.clone()),
                );
            }

            self.run_time = Some(StaticMeshComponentRuntime {
                draw_objects: draw_object,
                _mesh: find_static_mesh,
            })
        }
    }

    pub fn update(&mut self, time: f32, engine: &mut Engine) {
        let _ = time;
        let _ = engine;
        let Some(run_time) = &mut self.run_time else {
            return;
        };
        match &mut run_time.draw_objects {
            EDrawObjectType::Static(draw_object) => {
                draw_object.constants.model = self.transformation.clone();
            }
            EDrawObjectType::Skin(_) => unimplemented!(),
            EDrawObjectType::SkinMaterial(_) => unimplemented!(),
            EDrawObjectType::StaticMeshMaterial(draw_object) => {
                draw_object.constants.model = self.transformation.clone();
            }
        }
        engine.update_draw_object(&mut run_time.draw_objects);
    }

    pub fn get_draw_objects(&self) -> Vec<&EDrawObjectType> {
        match &self.run_time {
            Some(x) => vec![&x.draw_objects],
            None => vec![],
        }
    }

    pub fn set_material(
        &mut self,
        engine: &mut Engine,
        new_material_url: Option<url::Url>,
        files: &[EContentFileType],
    ) {
        self.material_url = new_material_url;
        let material = if let Some(material_url) = &self.material_url {
            files.iter().find_map(|x| {
                if let EContentFileType::Material(content_material) = x {
                    if &content_material.borrow().url == material_url {
                        return Some(content_material.clone());
                    }
                }
                None
            })
        } else {
            None
        };

        if let Some(run_time) = self.run_time.as_mut() {
            let static_mesh = run_time._mesh.clone();
            let draw_object: EDrawObjectType;
            if let Some(material) = material.clone() {
                draw_object = engine.create_material_draw_object_from_static_mesh(
                    &static_mesh.vertexes,
                    &static_mesh.indexes,
                    Some(static_mesh.name.clone()),
                    material,
                );
            } else {
                draw_object = engine.create_draw_object_from_static_mesh(
                    &static_mesh.vertexes,
                    &static_mesh.indexes,
                    Some(static_mesh.name.clone()),
                );
            }
            run_time.draw_objects = draw_object;
        }
    }
}
