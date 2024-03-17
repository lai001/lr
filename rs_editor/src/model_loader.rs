use anyhow::anyhow;
use rs_artifact::mesh_vertex::MeshVertex;
use rs_engine::resource_manager::ResourceManager;
use rs_render::vertex_data_type::skin_mesh_vertex::{SkinMeshVertex, INVALID_BONE};
use russimp::material::TextureType;
use std::{collections::HashMap, path::Path};

pub struct MeshCluster {
    pub name: String,
    pub vertex_buffer: Vec<MeshVertex>,
    pub index_buffer: Vec<u32>,
    pub textures_dic: HashMap<TextureType, String>,
}

pub struct SkinMeshCluster {
    pub name: String,
    pub bones: Vec<glam::Mat4>,
    pub vertex_buffer: Vec<SkinMeshVertex>,
    pub index_buffer: Vec<u32>,
    pub textures_dic: HashMap<TextureType, String>,
}

pub struct SkinResource {
    pub clusters: Vec<SkinMeshCluster>,
}

pub struct ModelLoader {}

impl ModelLoader {
    fn get_texture_absolute_path(
        model_file_path: &Path,
        texture: &russimp::material::Texture,
        additional_paths: &[&Path],
    ) -> String {
        let mut dirs: Vec<std::path::PathBuf> = Vec::new();

        if let Some(p) = model_file_path.parent() {
            dirs.push(p.into());
        }
        for p in additional_paths {
            dirs.push(p.into());
        }

        let paths = rs_foundation::search_file((&texture.filename).into(), dirs);
        if let Some(path) = paths.first() {
            path.to_string_lossy().to_string()
        } else {
            panic!()
        }
    }

    fn collect_textures(
        model_file_path: &Path,
        materials: &[russimp::material::Material],
        additional_paths: &[&Path],
    ) -> HashMap<String, TextureType> {
        let mut result = HashMap::new();
        for material in materials {
            for (texture_type, impoted_texture) in &material.textures {
                let path = Self::get_texture_absolute_path(
                    model_file_path,
                    &*impoted_texture.borrow(),
                    additional_paths,
                );
                result.insert(path, texture_type.clone());
            }
        }
        result
    }

    fn make_vertex(
        index: u32,
        imported_mesh: &russimp::mesh::Mesh,
        uv_map: &Option<Vec<russimp::Vector3D>>,
    ) -> MeshVertex {
        let mut texture_coord: glam::Vec2 = glam::vec2(0.0, 0.0);
        if let Some(uv_map) = uv_map {
            let uv = uv_map.get(index as usize).unwrap();
            texture_coord = glam::vec2(uv.x, uv.y);
        }
        let vertex = imported_mesh.vertices.get(index as usize).unwrap();
        let mut vertex_color: russimp::Color4D = russimp::Color4D {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 0.0,
        };
        if let Some(color) = imported_mesh.colors.get(index as usize) {
            if let Some(color) = color {
                if let Some(color) = color.get(0) {
                    vertex_color = *color;
                }
            }
        }
        let normal = imported_mesh
            .normals
            .get(index as usize)
            .unwrap_or(&russimp::Vector3D {
                x: 0.5,
                y: 0.5,
                z: 1.0,
            });
        let tangent = imported_mesh
            .tangents
            .get(index as usize)
            .unwrap_or(&russimp::Vector3D {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            });
        let bitangent =
            imported_mesh
                .bitangents
                .get(index as usize)
                .unwrap_or(&russimp::Vector3D {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                });

        let vertex = MeshVertex {
            vertex_color: glam::vec4(
                vertex_color.r,
                vertex_color.g,
                vertex_color.b,
                vertex_color.a,
            ),
            position: glam::vec3(vertex.x, vertex.y, vertex.z),
            normal: glam::vec3(normal.x, normal.y, normal.z),
            tangent: glam::vec3(tangent.x, tangent.y, tangent.z),
            bitangent: glam::vec3(bitangent.x, bitangent.y, bitangent.z),
            tex_coord: glam::vec2(texture_coord.x, texture_coord.y),
        };
        vertex
    }

    fn make_skin_vertex(
        index: u32,
        imported_mesh: &rs_assimp::mesh::Mesh,
        uv_map: &Option<Vec<glam::Vec3>>,
    ) -> SkinMeshVertex {
        let mut texture_coord: glam::Vec2 = glam::vec2(0.0, 0.0);
        if let Some(uv_map) = uv_map {
            let uv = uv_map.get(index as usize).unwrap();
            texture_coord = glam::vec2(uv.x, uv.y);
        }
        let vertex = imported_mesh.vertices.get(index as usize).unwrap();
        let mut vertex_color: glam::Vec4 = glam::Vec4::default();
        if let Some(color) = imported_mesh.colors.get(index as usize) {
            if let Some(color) = color.get(0) {
                vertex_color = *color;
            }
        }
        let normal = imported_mesh
            .normals
            .get(index as usize)
            .unwrap_or(&glam::Vec3 {
                x: 0.5,
                y: 0.5,
                z: 1.0,
            });
        let tangent = imported_mesh
            .tangents
            .get(index as usize)
            .unwrap_or(&glam::Vec3::X);
        let bitangent = imported_mesh
            .bitangents
            .get(index as usize)
            .unwrap_or(&glam::Vec3::Y);
        let mut vertex_bones: [i32; 4] = [INVALID_BONE, INVALID_BONE, INVALID_BONE, INVALID_BONE];
        let mut weights: [f32; 4] = [0.0, 0.0, 0.0, 0.0];
        for (bone_index, bone) in imported_mesh.bones.iter().enumerate() {
            let bone = bone.borrow();
            for weight in &bone.weights {
                if weight.vertex_id == index {
                    for (vertex_bone_index, vertex_bone) in vertex_bones.iter_mut().enumerate() {
                        if *vertex_bone == INVALID_BONE {
                            *vertex_bone = bone_index as _;
                            weights[vertex_bone_index] = weight.weight;
                            break;
                        }
                    }
                }
            }
        }

        let vertex = SkinMeshVertex {
            vertex_color,
            position: vertex.clone(),
            normal: normal.clone(),
            tangent: tangent.clone(),
            bitangent: bitangent.clone(),
            tex_coord: texture_coord.clone(),
            bones: vertex_bones,
            weights,
        };
        vertex
    }

    pub fn load_from_file(
        file_path: &Path,
        additional_paths: &[&Path],
    ) -> anyhow::Result<Vec<MeshCluster>> {
        let resource_manager = ResourceManager::default();
        let scene = russimp::scene::Scene::from_file(
            &file_path.to_str().unwrap(),
            vec![
                russimp::scene::PostProcess::Triangulate,
                russimp::scene::PostProcess::CalculateTangentSpace,
            ],
        )?;

        let mut mesh_clusters: Vec<MeshCluster> = Vec::new();
        let textures = Self::collect_textures(file_path, &scene.materials, additional_paths);
        {
            let mut load = HashMap::new();
            for (texture_path, texture_type) in &textures {
                log::trace!("texture_path: {texture_path}, texture_type: {texture_type:?}");
                load.insert(texture_path.as_str(), texture_path.as_str());
            }
            resource_manager.load_images_from_disk_and_cache_parallel(load);
        }
        for imported_mesh in &scene.meshes {
            let mut vertex_buffer: Vec<MeshVertex> = vec![];
            let mut index_buffer: Vec<u32> = vec![];
            let mut uv_map: Option<Vec<russimp::Vector3D>> = None;
            if let Some(map) = imported_mesh.texture_coords.get(0) {
                if let Some(map) = map {
                    uv_map = Some(map.to_vec());
                }
            }
            for face in &imported_mesh.faces {
                let indices = &face.0;
                for index in indices {
                    let vertex = Self::make_vertex(*index, imported_mesh, &uv_map);
                    vertex_buffer.push(vertex);
                    index_buffer.push(*index);
                }
            }
            assert_eq!(vertex_buffer.len() % 3, 0);
            let mut cluster = MeshCluster {
                vertex_buffer,
                index_buffer,
                textures_dic: HashMap::new(),
                name: imported_mesh.name.clone(),
            };
            if let Some(material) = scene.materials.get(imported_mesh.material_index as usize) {
                for (texture_type, texture) in &material.textures {
                    let path = Self::get_texture_absolute_path(
                        file_path,
                        &*texture.borrow(),
                        additional_paths,
                    );
                    cluster.textures_dic.insert(texture_type.clone(), path);
                }
            }
            mesh_clusters.push(cluster);
        }
        Ok(mesh_clusters)
    }

    pub fn load_from_file_as_skin(file_path: &Path) -> anyhow::Result<Vec<SkinMeshCluster>> {
        let mut props = rs_assimp::property_store::PropertyStore::new();
        props.set_property_integer(
            &rs_assimp::config::AI_CONFIG_FBX_USE_SKELETON_BONE_CONTAINER,
            1,
        );
        let scene = rs_assimp::scene::Scene::from_file_with_properties(
            file_path,
            rs_assimp::post_process_steps::PostProcessSteps::Triangulate
                | rs_assimp::post_process_steps::PostProcessSteps::PopulateArmatureData,
            props,
        )?;
        if scene.armatures.is_empty() {
            return Err(anyhow!("Not contains any armatures."));
        }
        if scene.armatures.len() > 1 {
            log::warn!("Too many armatures");
        }

        let mut mesh_clusters: Vec<SkinMeshCluster> = Vec::new();

        for imported_mesh in &scene.meshes {
            let mut vertex_buffer: Vec<SkinMeshVertex> = vec![];
            let mut index_buffer: Vec<u32> = vec![];
            let mut uv_map: Option<Vec<glam::Vec3>> = None;
            if let Some(map) = imported_mesh.texture_coords.get(0) {
                uv_map = Some(map.to_vec());
            }
            for face in &imported_mesh.faces {
                let indices = &face.indices;
                for index in indices {
                    let vertex = Self::make_skin_vertex(*index, imported_mesh, &uv_map);
                    vertex_buffer.push(vertex);
                    index_buffer.push(*index);
                }
            }
            assert_eq!(vertex_buffer.len() % 3, 0);
            let bones = imported_mesh
                .bones
                .iter()
                .map(|x| x.borrow().offset_matrix)
                .collect();
            let cluster = SkinMeshCluster {
                vertex_buffer,
                index_buffer,
                textures_dic: HashMap::new(),
                name: imported_mesh.name.clone(),
                bones,
            };
            mesh_clusters.push(cluster);
        }

        Ok(mesh_clusters)
    }
}
