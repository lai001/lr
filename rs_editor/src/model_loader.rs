use anyhow::{anyhow, Context};
use glam::Vec3Swizzles;
use rs_artifact::{
    mesh_vertex::MeshVertex,
    skin_mesh::{SkinMesh, SkinMeshVertex},
};
use rs_core_minimal::name_generator::NameGenerator;
use rs_engine::{
    build_content_file_url,
    resource_manager::ResourceManager,
    scene_node::{EComponentType, SceneComponent},
    static_mesh_component::StaticMeshComponent,
};
use rs_foundation::new::{SingleThreadMut, SingleThreadMutType};
use rs_render::vertex_data_type::skin_mesh_vertex::INVALID_BONE;
use russimp::material::TextureType;
use std::{
    cell::RefCell,
    collections::HashMap,
    iter::zip,
    path::{Path, PathBuf},
    rc::Rc,
    sync::Arc,
};

pub struct MeshCluster {
    pub name: String,
    pub vertex_buffer: Vec<MeshVertex>,
    pub index_buffer: Vec<u32>,
    pub textures_dic: HashMap<TextureType, String>,
}

pub struct LoadResult {
    pub asset_reference: String,
    pub static_meshes: Vec<Rc<RefCell<rs_engine::content::static_mesh::StaticMesh>>>,
    pub skeleton_meshes: Vec<Rc<RefCell<rs_engine::content::skeleton_mesh::SkeletonMesh>>>,
    pub skeleton: Option<Rc<RefCell<rs_engine::content::skeleton::Skeleton>>>,
    pub node_animations:
        Vec<Rc<RefCell<rs_engine::content::skeleton_animation::SkeletonAnimation>>>,
    pub actor: SingleThreadMutType<rs_engine::actor::Actor>,
}

pub struct ModelLoader {
    scene_cache: HashMap<PathBuf, Rc<rs_assimp::scene::Scene<'static>>>,
}

impl ModelLoader {
    pub fn new() -> ModelLoader {
        ModelLoader {
            scene_cache: HashMap::new(),
        }
    }

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

    fn make_vertex2(
        index: u32,
        imported_mesh: &rs_assimp::mesh::Mesh,
        uv_map: &Option<Vec<glam::Vec3>>,
        default_normal: &glam::Vec3,
        default_tangent: &glam::Vec3,
        default_bitangent: &glam::Vec3,
        default_vertex_color: &glam::Vec4,
    ) -> MeshVertex {
        let mut texture_coord: glam::Vec2 = glam::vec2(0.0, 0.0);
        if let Some(uv_map) = uv_map {
            if let Some(uv) = uv_map.get(index as usize) {
                texture_coord = uv.xy();
            }
        }
        let vertex = imported_mesh
            .vertices
            .get(index as usize)
            .expect("Should not be null");
        let mut vertex_color: &glam::Vec4 = default_vertex_color;
        if let Some(color) = imported_mesh.colors.get(index as usize) {
            if let Some(color) = color.get(0) {
                vertex_color = color;
            }
        }
        let normal = imported_mesh
            .normals
            .get(index as usize)
            .unwrap_or(default_normal);
        let tangent = imported_mesh
            .tangents
            .get(index as usize)
            .unwrap_or(default_tangent);
        let bitangent = imported_mesh
            .bitangents
            .get(index as usize)
            .unwrap_or(default_bitangent);

        let vertex = MeshVertex {
            vertex_color: vertex_color.clone(),
            position: vertex.clone(),
            normal: normal.clone(),
            tangent: tangent.clone(),
            bitangent: bitangent.clone(),
            tex_coord: texture_coord.clone(),
        };
        vertex
    }

    fn make_skin_vertex(
        index: u32,
        imported_mesh: &rs_assimp::mesh::Mesh,
        uv_map: &Option<Vec<glam::Vec3>>,
        default_normal: &glam::Vec3,
        default_tangent: &glam::Vec3,
        default_bitangent: &glam::Vec3,
        default_vertex_color: &glam::Vec4,
    ) -> SkinMeshVertex {
        let mut texture_coord: glam::Vec2 = glam::vec2(0.0, 0.0);
        if let Some(uv_map) = uv_map {
            let uv = uv_map.get(index as usize).unwrap();
            texture_coord = uv.xy();
        }
        let vertex = imported_mesh.vertices.get(index as usize).unwrap();
        let mut vertex_color: &glam::Vec4 = default_vertex_color;
        if let Some(color) = imported_mesh.colors.get(index as usize) {
            if let Some(color) = color.get(0) {
                vertex_color = color;
            }
        }
        let normal = imported_mesh
            .normals
            .get(index as usize)
            .unwrap_or(default_normal);
        let tangent = imported_mesh
            .tangents
            .get(index as usize)
            .unwrap_or(default_tangent);
        let bitangent = imported_mesh
            .bitangents
            .get(index as usize)
            .unwrap_or(default_bitangent);
        let vertex_bones: [i32; 4] = [INVALID_BONE, INVALID_BONE, INVALID_BONE, INVALID_BONE];
        let weights: [f32; 4] = [0.0, 0.0, 0.0, 0.0];

        let vertex = SkinMeshVertex {
            vertex_color: vertex_color.clone(),
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

    pub fn load(&mut self, file_path: &Path) -> anyhow::Result<()> {
        if !self.scene_cache.contains_key(file_path) {
            let mut props = rs_assimp::property_store::PropertyStore::new();
            props.set_property_integer(
                &rs_assimp::config::AI_CONFIG_FBX_USE_SKELETON_BONE_CONTAINER,
                1,
            );
            self.scene_cache.insert(
                file_path.to_path_buf(),
                Rc::new(rs_assimp::scene::Scene::from_file_with_properties(
                    file_path,
                    rs_assimp::post_process_steps::PostProcessSteps::Triangulate
                        | rs_assimp::post_process_steps::PostProcessSteps::PopulateArmatureData,
                    props,
                )?),
            );
        }
        Ok(())
    }

    pub fn get(&self, file_path: &Path) -> anyhow::Result<Rc<rs_assimp::scene::Scene<'static>>> {
        let cache_scene = self.scene_cache.get(file_path);
        match cache_scene {
            Some(cache_scene) => Ok(cache_scene.clone()),
            None => Err(anyhow!("")),
        }
    }

    pub fn to_runtime_static_mesh(
        &self,
        static_mesh: &rs_engine::content::static_mesh::StaticMesh,
        asset_folder: &Path,
        resource_manager: ResourceManager,
    ) -> anyhow::Result<Arc<rs_artifact::static_mesh::StaticMesh>> {
        let url = static_mesh.url.clone();

        match resource_manager.get_static_mesh(&url) {
            Ok(loaded_mesh) => Ok(loaded_mesh),
            Err(_) => {
                let relative_path = &static_mesh.asset_info.relative_path;
                let path = asset_folder.join(relative_path);

                let scene_cache = self
                    .scene_cache
                    .get(&path)
                    .expect(&format!("{:?} Scene has been loaded.", path));
                let imported_mesh = scene_cache
                    .meshes
                    .iter()
                    .find(|x| x.borrow().name == static_mesh.asset_info.path)
                    .expect("Find matching mesh.");
                let mut triangle_count: usize = 0;
                for face in &imported_mesh.borrow().faces {
                    debug_assert_eq!(face.indices.len(), 3);
                    triangle_count += 1;
                }

                let mut vertex_buffer: Vec<MeshVertex> = vec![];
                let mut index_buffer: Vec<u32> = Vec::with_capacity(triangle_count * 3);
                let mut uv_map: Option<Vec<glam::Vec3>> = None;
                if let Some(map) = imported_mesh.borrow().texture_coords.get(0) {
                    uv_map = Some(map.to_vec());
                }

                let default_normal = glam::Vec3 {
                    x: 0.5,
                    y: 0.5,
                    z: 1.0,
                };
                let default_tangent = glam::Vec3::X;
                let default_bitangent = glam::Vec3::Y;
                let default_vertex_color = glam::Vec4::ZERO;
                let mut vertex_helper: HashMap<usize, bool> = HashMap::new();
                for face in &imported_mesh.borrow().faces {
                    for index in &face.indices {
                        vertex_helper.insert(*index as usize, false);
                    }
                }
                vertex_buffer.resize(vertex_helper.len(), Default::default());

                for face in &imported_mesh.borrow().faces {
                    let indices = &face.indices;
                    for index in indices {
                        index_buffer.push(*index);
                        let is_create = vertex_helper.get_mut(&((*index) as usize)).unwrap();
                        if *is_create {
                            continue;
                        }
                        *is_create = true;
                        let vertex = Self::make_vertex2(
                            *index,
                            &imported_mesh.borrow(),
                            &uv_map,
                            &default_normal,
                            &default_tangent,
                            &default_bitangent,
                            &default_vertex_color,
                        );
                        vertex_buffer[(*index) as usize] = vertex;
                    }
                }

                let static_mesh = rs_artifact::static_mesh::StaticMesh {
                    vertexes: vertex_buffer,
                    indexes: index_buffer,
                    name: static_mesh.asset_info.path.clone(),
                    url: static_mesh.asset_info.get_url(),
                };
                let static_mesh = Arc::new(static_mesh);
                resource_manager.add_static_mesh(static_mesh.url.clone(), static_mesh.clone());
                log::trace!(
                    r#"Load static mesh "{}" from scene {:?}."#,
                    static_mesh.clone().name,
                    path
                );
                Ok(static_mesh)
            }
        }
    }

    pub fn to_runtime_skin_mesh(
        &self,
        skeleton_mesh: &rs_engine::content::skeleton_mesh::SkeletonMesh,
        asset_folder: &Path,
        resource_manager: ResourceManager,
    ) -> Arc<SkinMesh> {
        let url = skeleton_mesh.asset_url.clone();
        match resource_manager.get_skin_mesh(&url) {
            Some(loaded_mesh) => loaded_mesh.clone(),
            None => {
                let path = asset_folder.join(skeleton_mesh.get_relative_path());
                let scene_cache = self
                    .scene_cache
                    .get(&path)
                    .expect(&format!("{:?} Scene has been loaded.", path));
                let imported_mesh = scene_cache
                    .meshes
                    .iter()
                    .find(|x| x.borrow().name == skeleton_mesh.get_skeleton_mesh_name())
                    .expect("Find matching mesh.");
                let mut triangle_count: usize = 0;
                for face in &imported_mesh.borrow().faces {
                    debug_assert_eq!(face.indices.len(), 3);
                    triangle_count += 1;
                }

                let mut vertex_buffer: Vec<SkinMeshVertex> = vec![];
                let mut index_buffer: Vec<u32> = Vec::with_capacity(triangle_count * 3);
                let mut uv_map: Option<Vec<glam::Vec3>> = None;
                if let Some(map) = imported_mesh.borrow().texture_coords.get(0) {
                    uv_map = Some(map.to_vec());
                }

                let default_normal = glam::Vec3 {
                    x: 0.5,
                    y: 0.5,
                    z: 1.0,
                };
                let default_tangent = glam::Vec3::X;
                let default_bitangent = glam::Vec3::Y;
                let default_vertex_color = glam::Vec4::ZERO;
                let mut vertex_helper: HashMap<usize, bool> = HashMap::new();
                for face in &imported_mesh.borrow().faces {
                    for index in &face.indices {
                        vertex_helper.insert(*index as usize, false);
                    }
                }
                vertex_buffer.resize(vertex_helper.len(), Default::default());

                for face in &imported_mesh.borrow().faces {
                    let indices = &face.indices;
                    for index in indices {
                        index_buffer.push(*index);
                        let is_create = vertex_helper.get_mut(&((*index) as usize)).unwrap();
                        if *is_create {
                            continue;
                        }
                        *is_create = true;
                        let vertex = Self::make_skin_vertex(
                            *index,
                            &imported_mesh.borrow(),
                            &uv_map,
                            &default_normal,
                            &default_tangent,
                            &default_bitangent,
                            &default_vertex_color,
                        );
                        vertex_buffer[(*index) as usize] = vertex;
                    }
                }

                for (bone_index, bone) in imported_mesh.borrow().bones.iter().enumerate() {
                    let bone = bone.borrow();
                    for weight in &bone.weights {
                        let vertex = vertex_buffer.get_mut(weight.vertex_id as usize).unwrap();
                        for (vertex_bone, vertex_weight) in
                            zip(vertex.bones.iter_mut(), vertex.weights.iter_mut())
                        {
                            if *vertex_bone == INVALID_BONE {
                                *vertex_bone = bone_index as _;
                                *vertex_weight = weight.weight;
                                break;
                            }
                        }
                    }
                }

                let bone_paths = imported_mesh
                    .borrow()
                    .bones
                    .iter()
                    .map(|x| x.borrow().node.clone().unwrap().borrow().path.clone())
                    .collect();
                let skin_mesh = SkinMesh {
                    name: skeleton_mesh.get_skeleton_mesh_name().clone(),
                    url: skeleton_mesh.asset_url.clone(),
                    vertexes: vertex_buffer,
                    indexes: index_buffer,
                    bone_paths,
                };
                let skin_mesh = Arc::new(skin_mesh);
                resource_manager.add_skin_mesh(skeleton_mesh.asset_url.clone(), skin_mesh.clone());
                log::trace!(
                    r#"Load skin mesh "{}" from scene {:?}."#,
                    skin_mesh.clone().name,
                    path
                );
                skin_mesh
            }
        }
    }

    pub fn to_runtime_skeleton_animation<'a>(
        &self,
        skeleton_animation: Rc<RefCell<rs_engine::content::skeleton_animation::SkeletonAnimation>>,
        asset_folder: &Path,
        resource_manager: ResourceManager,
    ) -> Arc<rs_artifact::skeleton_animation::SkeletonAnimation> {
        let url = skeleton_animation.borrow().asset_url.clone();
        match resource_manager.get_skeleton_animation(&url) {
            Some(loaded_animation) => loaded_animation.clone(),
            None => {
                let path = asset_folder.join(skeleton_animation.borrow().get_relative_path());
                let scene_cache = self
                    .scene_cache
                    .get(&path)
                    .expect(&format!("{:?} Scene has been loaded.", path));
                let animation = scene_cache
                    .animations
                    .iter()
                    .find(|x| x.name == skeleton_animation.borrow().get_animation_name())
                    .expect("Find matching animation.");
                let mut channels: Vec<rs_artifact::node_anim::NodeAnim> = vec![];
                for channel in &animation.channels {
                    let node_anim = rs_artifact::node_anim::NodeAnim {
                        node: channel.node.as_ref().unwrap().borrow().path.clone(),
                        position_keys: channel
                            .position_keys
                            .iter()
                            .map(|x| rs_artifact::node_anim::VectorKey {
                                time: x.time,
                                value: x.value,
                            })
                            .collect(),
                        scaling_keys: channel
                            .scaling_keys
                            .iter()
                            .map(|x| rs_artifact::node_anim::VectorKey {
                                time: x.time,
                                value: x.value,
                            })
                            .collect(),
                        rotation_keys: channel
                            .rotation_keys
                            .iter()
                            .map(|x| rs_artifact::node_anim::QuatKey {
                                time: x.time,
                                value: x.value,
                            })
                            .collect(),
                    };
                    channels.push(node_anim);
                }
                let skeleton_animation = rs_artifact::skeleton_animation::SkeletonAnimation {
                    name: skeleton_animation.borrow().get_animation_name().clone(),
                    url: skeleton_animation.borrow().asset_url.clone(),
                    duration: animation.duration,
                    ticks_per_second: animation.ticks_per_second,
                    channels,
                };
                let skeleton_animation = Arc::new(skeleton_animation);
                resource_manager.add_skeleton_animation(
                    skeleton_animation.url.clone(),
                    skeleton_animation.clone(),
                );
                log::trace!(
                    r#"Load skeleton animation "{}" from scene {:?}."#,
                    skeleton_animation.clone().name,
                    path
                );
                skeleton_animation
            }
        }
    }

    fn make_bones<'a>(
        node: Rc<RefCell<rs_assimp::node::Node<'a>>>,
        parent: Option<String>,
        bones: &mut HashMap<String, rs_artifact::skeleton::SkeletonBone>,
    ) {
        let node = node.borrow();
        let offset_matrix = node.bone_offset_matrix.unwrap_or(glam::Mat4::IDENTITY);
        let bone = rs_artifact::skeleton::SkeletonBone {
            path: node.path.clone(),
            parent,
            childs: node
                .children
                .iter()
                .map(|x| x.borrow().path.clone())
                .collect(),
            offset_matrix,
        };

        for child_node in node.children.iter() {
            Self::make_bones(child_node.clone(), Some(bone.path.clone()), bones);
        }

        bones.insert(node.path.clone(), bone);
    }

    fn make_skeleton_mesh_hierarchy<'a>(
        node: Rc<RefCell<rs_assimp::node::Node<'a>>>,
        parent: Option<String>,
        skeleton_mesh_hierarchy: &mut HashMap<
            String,
            rs_artifact::skeleton::SkeletonMeshHierarchyNode,
        >,
    ) {
        let node = node.borrow();
        let skeleton_mesh_hierarchy_node = rs_artifact::skeleton::SkeletonMeshHierarchyNode {
            path: node.path.clone(),
            transformation: node.transformation.clone(),
            parent,
            childs: node
                .children
                .iter()
                .map(|x| x.borrow().path.clone())
                .collect(),
        };
        for child_node in node.children.iter() {
            Self::make_skeleton_mesh_hierarchy(
                child_node.clone(),
                Some(skeleton_mesh_hierarchy_node.path.clone()),
                skeleton_mesh_hierarchy,
            );
        }

        skeleton_mesh_hierarchy.insert(node.path.clone(), skeleton_mesh_hierarchy_node);
    }

    pub fn to_runtime_skeleton<'a>(
        &self,
        skeleton: Rc<RefCell<rs_engine::content::skeleton::Skeleton>>,
        asset_folder: &Path,
        resource_manager: ResourceManager,
    ) -> Arc<rs_artifact::skeleton::Skeleton> {
        let url = skeleton.borrow().asset_url.clone();
        match resource_manager.get_skeleton(&url) {
            Some(loaded_skeleton) => loaded_skeleton.clone(),
            None => {
                let path = asset_folder.join(skeleton.borrow().get_relative_path());
                let scene = self
                    .scene_cache
                    .get(&path)
                    .expect(&format!("{:?} Scene has been loaded.", path));
                let armature = scene.armatures.values().next().unwrap();
                let root_node = scene.root_node.clone().unwrap();
                let mut bones: HashMap<String, rs_artifact::skeleton::SkeletonBone> =
                    Default::default();
                let mut skeleton_mesh_hierarchy: HashMap<
                    String,
                    rs_artifact::skeleton::SkeletonMeshHierarchyNode,
                > = Default::default();
                Self::make_skeleton_mesh_hierarchy(
                    root_node.clone(),
                    None,
                    &mut skeleton_mesh_hierarchy,
                );
                Self::make_bones(armature.clone(), None, &mut bones);
                let skeleton = rs_artifact::skeleton::Skeleton {
                    name: armature.borrow().name.clone(),
                    url: skeleton.borrow().asset_url.clone(),
                    root_bone: armature.borrow().path.clone(),
                    root_node: root_node.borrow().path.clone(),
                    bones,
                    skeleton_mesh_hierarchy,
                };
                let skeleton = Arc::new(skeleton);
                resource_manager.add_skeleton(skeleton.url.clone(), skeleton.clone());
                log::trace!(
                    r#"Load skeleton "{}" from scene {:?}."#,
                    skeleton.clone().name,
                    path
                );
                skeleton
            }
        }
    }

    fn node_to_component_type(
        node: SingleThreadMutType<rs_assimp::node::Node>,
        static_meshes: &[SingleThreadMutType<rs_engine::content::static_mesh::StaticMesh>],
    ) -> EComponentType {
        let node = node.borrow_mut();
        let name = node.name.clone();
        let transformation = node.transformation.clone();
        match node.get_node_type() {
            rs_assimp::node::ENodeType::Axis => {
                let scene_component =
                    SingleThreadMut::new(SceneComponent::new(name, transformation));
                return EComponentType::SceneComponent(scene_component);
            }
            rs_assimp::node::ENodeType::Bone => unimplemented!(),
            rs_assimp::node::ENodeType::Mesh => {
                let Some(mesh) = node.meshes.first().cloned() else {
                    unimplemented!();
                };
                let mesh_name = {
                    let mesh = mesh.borrow();
                    let mesh_name = mesh.name.clone();
                    mesh_name
                };
                let static_mesh_url = static_meshes
                    .iter()
                    .find(|x| {
                        let x = x.borrow();
                        x.asset_info.path == mesh_name
                    })
                    .map(|x| x.borrow().url.clone());
                let static_mesh_component =
                    StaticMeshComponent::new(name, static_mesh_url, None, transformation);
                let static_mesh_component = SingleThreadMut::new(static_mesh_component);
                return EComponentType::StaticMeshComponent(static_mesh_component);
            }
            rs_assimp::node::ENodeType::Armature => unimplemented!(),
        }
    }

    fn node_to_scene_node_recursion(
        node: SingleThreadMutType<rs_assimp::node::Node>,
        static_meshes: &[SingleThreadMutType<rs_engine::content::static_mesh::StaticMesh>],
    ) -> SingleThreadMutType<rs_engine::scene_node::SceneNode> {
        let component_type = Self::node_to_component_type(node.clone(), static_meshes);
        let scene_node = SingleThreadMut::new(rs_engine::scene_node::SceneNode {
            component: component_type,
            childs: vec![],
        });
        let node = node.borrow();
        for child in node.children.clone() {
            let child_scene_node = Self::node_to_scene_node_recursion(child, static_meshes);
            scene_node.borrow_mut().childs.push(child_scene_node);
        }
        scene_node
    }

    pub fn load_from_file_as_actor(
        &mut self,
        file_path: &Path,
        asset_reference: String,
        exist_content_names: Vec<String>,
        exist_actors_names: Vec<String>,
    ) -> anyhow::Result<LoadResult> {
        let mut name_generator = NameGenerator::new(exist_content_names);
        let mut actor_name_generator = NameGenerator::new(exist_actors_names);

        let mut props = rs_assimp::property_store::PropertyStore::new();
        props.set_property_bool(
            &rs_assimp::config::AI_CONFIG_FBX_USE_SKELETON_BONE_CONTAINER,
            true,
        );
        if !self.scene_cache.contains_key(file_path) {
            self.scene_cache.insert(
                file_path.to_path_buf(),
                Rc::new(rs_assimp::scene::Scene::from_file_with_properties(
                    file_path,
                    rs_assimp::post_process_steps::PostProcessSteps::Triangulate
                        | rs_assimp::post_process_steps::PostProcessSteps::PopulateArmatureData,
                    props,
                )?),
            );
        }
        let scene = self
            .scene_cache
            .get(file_path)
            .ok_or(anyhow::anyhow!("Failed to load file: {:?}", file_path))?;

        if scene.armatures.len() > 1 {
            log::warn!("Too many armatures");
        }
        let Some(scene_root_node) = scene.root_node.clone() else {
            return Err(anyhow::anyhow!("No root node"));
        };
        let _ = scene_root_node;

        let mut static_meshes: Vec<Rc<RefCell<rs_engine::content::static_mesh::StaticMesh>>> =
            vec![];
        let mut skeleton_meshes: Vec<Rc<RefCell<rs_engine::content::skeleton_mesh::SkeletonMesh>>> =
            vec![];
        let mut skeleton: Option<Rc<RefCell<rs_engine::content::skeleton::Skeleton>>> = None;
        let mut node_animations: Vec<
            Rc<RefCell<rs_engine::content::skeleton_animation::SkeletonAnimation>>,
        > = vec![];
        let actor: rs_engine::actor::Actor;

        if let Some(armature) = scene.armatures.values().next() {
            let armature = armature.borrow();
            let name = armature.name.clone().replace("|", "_");
            let name = name_generator.next(&name);
            let url: url::Url = build_content_file_url(name).context(armature.name.clone())?;

            let asset_url = rs_engine::content::skeleton::Skeleton::make_asset_url(
                &asset_reference,
                &armature.path,
            );
            skeleton = Some(Rc::new(RefCell::new(
                rs_engine::content::skeleton::Skeleton { url, asset_url },
            )));
        }

        for animation in &scene.animations {
            let animation_name = animation.name.clone();
            let name = animation_name.clone().replace("|", "_");
            let name = name_generator.next(&name);
            let url = build_content_file_url(&name).context(animation_name.clone())?;
            let asset_url =
                rs_engine::content::skeleton_animation::SkeletonAnimation::make_asset_url(
                    &asset_reference,
                    &animation_name,
                );
            let node_animation =
                rs_engine::content::skeleton_animation::SkeletonAnimation { url, asset_url };
            node_animations.push(SingleThreadMut::new(node_animation));
        }

        for imported_mesh in &scene.meshes {
            let imported_mesh = imported_mesh.clone();
            let imported_mesh = imported_mesh.borrow();
            let name = imported_mesh.name.clone().replace("|", "_");
            let name = name_generator.next(&name);
            let url = build_content_file_url(&name).context(imported_mesh.name.clone())?;
            if imported_mesh.bones.is_empty() {
                let static_mesh = rs_engine::content::static_mesh::StaticMesh {
                    // asset_reference_name: imported_mesh.name.clone(),
                    url,
                    // asset_reference_relative_path: asset_reference.clone(),
                    asset_info: rs_engine::content::static_mesh::AssetInfo {
                        relative_path: Path::new(&asset_reference).to_path_buf(),
                        path: imported_mesh.name.clone(),
                    },
                };
                static_meshes.push(Rc::new(RefCell::new(static_mesh)));
            } else {
                let asset_url = rs_engine::content::skeleton_mesh::SkeletonMesh::make_asset_url(
                    &asset_reference,
                    &imported_mesh.name,
                );
                let skeleton_mesh = rs_engine::content::skeleton_mesh::SkeletonMesh {
                    url,
                    skeleton_url: skeleton
                        .clone()
                        .ok_or(anyhow::anyhow!("Skeleton not found"))?
                        .clone()
                        .borrow()
                        .url
                        .clone(),
                    asset_url,
                };
                skeleton_meshes.push(Rc::new(RefCell::new(skeleton_mesh)));
            }
        }

        if let Some(skeleton) = skeleton.clone() {
            let animation_url: Option<url::Url>;
            if let Some(node_animation) = node_animations.first() {
                animation_url = Some(node_animation.borrow().url.clone());
            } else {
                animation_url = None;
            }

            let skeleton_mesh_component =
                rs_engine::skeleton_mesh_component::SkeletonMeshComponent::new(
                    skeleton.borrow().get_name().clone(),
                    Some(skeleton.borrow().url.clone()),
                    skeleton_meshes
                        .iter()
                        .map(|x| x.borrow().url.clone())
                        .collect(),
                    animation_url,
                    None,
                    glam::Mat4::IDENTITY,
                );

            actor = rs_engine::actor::Actor {
                name: actor_name_generator.next(&scene.name),
                scene_node: SingleThreadMut::new(rs_engine::scene_node::SceneNode {
                    component: rs_engine::scene_node::EComponentType::SkeletonMeshComponent(
                        SingleThreadMut::new(skeleton_mesh_component),
                    ),
                    childs: vec![],
                }),
            };
        } else {
            let scene_node = Self::node_to_scene_node_recursion(scene_root_node, &static_meshes);
            let name = file_path
                .file_name()
                .map(|x| x.to_str())
                .flatten()
                .map(|x| x.to_string())
                .ok_or(anyhow::anyhow!("Incorrect file path: {:?}", file_path))?;
            actor = rs_engine::actor::Actor {
                name: actor_name_generator.next(if scene.name.is_empty() {
                    &name
                } else {
                    &scene.name
                }),
                scene_node,
            };
        }

        Ok(LoadResult {
            asset_reference: asset_reference.clone(),
            static_meshes,
            skeleton_meshes,
            skeleton,
            node_animations,
            actor: SingleThreadMut::new(actor),
        })
    }
}
