use crate::{
    brigde_data::mesh_vertex::MeshVertex,
    file_manager::FileManager,
    material::Material,
    material_type::EMaterialType,
    resource_manager::ResourceManager,
    static_mesh::{Mesh, StaticMesh},
    util,
};
use russimp::{node::Node, texture::TextureType};
use std::{cell::RefCell, collections::HashMap, rc::Rc, sync::Arc};

pub struct MeshCluster {
    pub vertex_buffer: Vec<MeshVertex>,
    pub index_buffer: Vec<u32>,
    pub textures_dic: HashMap<russimp::texture::TextureType, String>,
}

pub struct ModelLoader {}

impl ModelLoader {
    fn walk_node(node: Rc<RefCell<Node>>) {
        let node = node.borrow();
        let transformation = util::russimp_mat4_to_glam_mat4(&node.transformation);
        let mut parent_node_name = "".to_string();
        if let Some(parent) = &node.parent {
            parent_node_name = parent.borrow().name.to_string();
        }
        log::trace!(
            "\nparent_node.name: {}, node.name: {}.\nnode.transformation: {:?}",
            parent_node_name,
            node.name,
            transformation
        );
        for node in &node.children {
            Self::walk_node(node.to_owned());
        }
    }

    fn get_texture_absolute_path(
        model_file_path: &str,
        texture: &russimp::texture::Texture,
        fm: &FileManager,
    ) -> String {
        let paths = rs_foundation::search_file(
            (&texture.path).into(),
            vec![
                fm.get_resource_dir_path().into(),
                std::path::Path::new(model_file_path)
                    .parent()
                    .unwrap()
                    .into(),
            ],
        );
        if let Some(path) = paths.first() {
            path.to_string_lossy().to_string()
        } else {
            panic!()
        }
    }

    fn collect_textures(
        model_file_path: &str,
        materials: &[russimp::material::Material],
        fm: &FileManager,
    ) -> HashMap<String, TextureType> {
        let mut result = HashMap::new();
        for material in materials {
            for (texture_type, textures) in &material.textures {
                for impoted_texture in textures {
                    let path =
                        Self::get_texture_absolute_path(model_file_path, impoted_texture, fm);
                    result.insert(path, texture_type.clone());
                }
            }
        }
        result
    }

    pub fn load_from_file(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        file_path: &str,
    ) -> Vec<StaticMesh> {
        let mut static_meshs: Vec<StaticMesh> = vec![];

        let scene = russimp::scene::Scene::from_file(
            &file_path,
            vec![
                russimp::scene::PostProcess::Triangulate,
                russimp::scene::PostProcess::CalculateTangentSpace,
            ],
        )
        .unwrap();

        if let Some(node) = scene.root {
            Self::walk_node(node);
        }

        let mut diffuse_textures: HashMap<String, Arc<Option<wgpu::Texture>>> = HashMap::new();
        let mut specular_textures: HashMap<String, Arc<Option<wgpu::Texture>>> = HashMap::new();

        for material in &scene.materials {
            for (texture_type, textures) in &material.textures {
                for impoted_texture in textures {
                    let path = FileManager::default().get_resource_path(&impoted_texture.path);
                    match texture_type {
                        russimp::texture::TextureType::Diffuse => {
                            if let Some((texture, _)) =
                                util::texture2d_from_rgba_image_file(device, queue, true, &path)
                            {
                                log::trace!("Load diffuse texture from {}", &path);

                                diffuse_textures.insert(path, Arc::new(Some(texture)));
                                // diffuse_texture = Some(Arc::new(texs.0));
                            }
                        }
                        russimp::texture::TextureType::Specular => {
                            if let Some((texture, _)) =
                                util::texture2d_from_rgba_image_file(device, queue, true, &path)
                            {
                                log::trace!("Load specular texture from {}", &path);

                                specular_textures.insert(path, Arc::new(Some(texture)));
                                // specular_texture = Some(Arc::new(texs.0));
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        for imported_mesh in &scene.meshes {
            let mut vertex_buffer: Vec<MeshVertex> = vec![];
            let mut index_buffer: Vec<u32> = vec![];
            let mut uv_map: Option<Vec<russimp::Vector3D>> = None;
            let mut diffuse_texture: Arc<Option<wgpu::Texture>> = Arc::new(None);
            let mut specular_texture: Arc<Option<wgpu::Texture>> = Arc::new(None);

            for (texture_type, textures) in &scene
                .materials
                .get(imported_mesh.material_index as usize)
                .unwrap()
                .textures
            {
                let texture = textures.get(0).unwrap();
                let path = FileManager::default().get_resource_path(&texture.path);
                match texture_type {
                    russimp::texture::TextureType::Diffuse => {
                        if let Some(texture) = diffuse_textures.get(&path) {
                            diffuse_texture = texture.clone();
                        }
                    }
                    russimp::texture::TextureType::Specular => {
                        if let Some(texture) = specular_textures.get(&path) {
                            specular_texture = texture.clone();
                        }
                    }
                    _ => {}
                }
                // log::trace!("{:?}", textures);
            }

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

            log::trace!(
                "imported_mesh.name: {}, vertex count: {}",
                &imported_mesh.name,
                vertex_buffer.len()
            );

            let mesh = Mesh::new(vertex_buffer, index_buffer);
            let material = Material::new(diffuse_texture, specular_texture);

            let static_mesh = StaticMesh::new(
                &imported_mesh.name,
                mesh,
                device,
                EMaterialType::Phong(material),
            );
            static_meshs.push(static_mesh);
        }
        static_meshs
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

    pub fn load_from_file2(file_path: &str) -> Vec<MeshCluster> {
        let file_manager = FileManager::default();
        let resource_manager = ResourceManager::default();
        let load_result = russimp::scene::Scene::from_file(
            &file_path,
            vec![
                russimp::scene::PostProcess::Triangulate,
                russimp::scene::PostProcess::CalculateTangentSpace,
            ],
        );
        if let Err(error) = load_result {
            log::warn!("{}", error);
            return Vec::new();
        }
        let scene = load_result.unwrap();
        let mut mesh_clusters: Vec<MeshCluster> = Vec::new();
        let textures = Self::collect_textures(file_path, &scene.materials, &file_manager);
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
            };
            if let Some(material) = scene.materials.get(imported_mesh.material_index as usize) {
                for (texture_type, textures) in &material.textures {
                    for texture in textures {
                        let path =
                            Self::get_texture_absolute_path(file_path, texture, &file_manager);
                        cluster.textures_dic.insert(texture_type.clone(), path);
                    }
                }
            }
            mesh_clusters.push(cluster);
        }
        mesh_clusters
    }
}
