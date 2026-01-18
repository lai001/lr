use crate::{
    content::{content_file_type::EContentFileType, level::LevelPhysics},
    drawable::EDrawObjectType,
    engine::Engine,
    player_viewport::PlayerViewport,
    resource_manager::ResourceManager,
    skeleton_animation_provider::{
        BlendSkeletonAnimationsProvider, SingleSkeletonAnimationProvider, SkeletonAnimationProvider,
    },
    static_mesh_component::Physics,
};
use rapier3d::prelude::*;
use rs_artifact::{skeleton::Skeleton, skin_mesh::SkinMesh};
use rs_render::global_shaders::skeleton_shading::NUM_MAX_BONE;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, iter::zip, sync::Arc};

#[derive(Clone)]
struct SkeletonMeshComponentRuntime {
    draw_objects: HashMap<String, EDrawObjectType>,
    skeleton: Option<Arc<Skeleton>>,
    skin_meshes: Vec<Arc<SkinMesh>>,
    pub physics: Option<Physics>,
    pub parent_final_transformation: glam::Mat4,
    pub final_transformation: glam::Mat4,
    skeleton_animation_provider: Option<Box<dyn SkeletonAnimationProvider>>,
    // material: Option<SingleThreadMutType<crate::content::material::Material>>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SkeletonMeshComponent {
    pub name: String,
    pub skeleton_url: Option<url::Url>,
    pub skeleton_mesh_urls: Vec<url::Url>,
    pub animation_url: Option<url::Url>,
    pub material_url: Option<url::Url>,
    pub transformation: glam::Mat4,
    #[serde(skip)]
    run_time: Option<SkeletonMeshComponentRuntime>,
}

impl SkeletonMeshComponent {
    pub fn get_transformation_mut(&mut self) -> &mut glam::Mat4 {
        &mut self.transformation
    }

    pub fn get_transformation(&self) -> &glam::Mat4 {
        &self.transformation
    }

    pub fn set_parent_final_transformation(&mut self, parent_final_transformation: glam::Mat4) {
        let Some(run_time) = self.run_time.as_mut() else {
            return;
        };
        run_time.parent_final_transformation = parent_final_transformation;
    }

    pub fn get_parent_final_transformation(&self) -> glam::Mat4 {
        let Some(run_time) = self.run_time.as_ref() else {
            return glam::Mat4::IDENTITY;
        };
        run_time.parent_final_transformation
    }

    pub fn set_final_transformation(&mut self, final_transformation: glam::Mat4) {
        let Some(run_time) = self.run_time.as_mut() else {
            return;
        };
        run_time.final_transformation = final_transformation;
    }

    pub fn get_final_transformation(&self) -> glam::Mat4 {
        self.run_time
            .as_ref()
            .map(|x| x.final_transformation)
            .unwrap_or_default()
    }

    pub fn new(
        name: String,
        skeleton_url: Option<url::Url>,
        skeleton_mesh_urls: Vec<url::Url>,
        animation_url: Option<url::Url>,
        material_url: Option<url::Url>,
        transformation: glam::Mat4,
    ) -> SkeletonMeshComponent {
        SkeletonMeshComponent {
            name,
            skeleton_url,
            skeleton_mesh_urls,
            animation_url,
            transformation,
            material_url,
            run_time: None,
        }
    }

    fn find_animation_provider(
        &self,
        files: &[EContentFileType],
    ) -> Option<Box<dyn SkeletonAnimationProvider>> {
        let Some(skeleton_url) = &self.skeleton_url else {
            return None;
        };
        let Some(animation_url) = &self.animation_url else {
            return None;
        };
        let blend_skeleton_animation_provider =
            BlendSkeletonAnimationsProvider::from(skeleton_url, animation_url, files);
        if let Some(blend_skeleton_animation_provider) = blend_skeleton_animation_provider {
            return Some(Box::new(blend_skeleton_animation_provider));
        }
        let single_skeleton_animation_provider =
            SingleSkeletonAnimationProvider::from(skeleton_url, animation_url, files);
        if let Some(single_skeleton_animation_provider) = single_skeleton_animation_provider {
            return Some(Box::new(single_skeleton_animation_provider));
        }
        None
    }

    pub fn initialize(
        &mut self,
        engine: &mut Engine,
        files: &[EContentFileType],
        player_viewport: &mut PlayerViewport,
    ) {
        let resource_manager = engine.get_resource_manager().clone();
        let mut skeleton: Option<Arc<Skeleton>> = None;

        if let Some(skeleton_url) = &self.skeleton_url {
            for file in files.iter() {
                if let EContentFileType::Skeleton(content_skeleton) = file {
                    if &content_skeleton.borrow().url == skeleton_url {
                        skeleton =
                            resource_manager.get_skeleton(&content_skeleton.borrow().asset_url);
                        break;
                    }
                }
            }
        }

        let skeleton_animation_provider = self.find_animation_provider(files);

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

        self.run_time = Some(SkeletonMeshComponentRuntime {
            draw_objects: HashMap::new(),
            skeleton: skeleton.clone(),
            skin_meshes: vec![],
            physics: None,
            final_transformation: glam::Mat4::IDENTITY,
            parent_final_transformation: glam::Mat4::IDENTITY,
            skeleton_animation_provider,
            // material: material.clone(),
        });

        for skeleton_mesh in &self.skeleton_mesh_urls {
            let mut skin_mesh: Option<Arc<SkinMesh>> = None;
            for file in files.iter() {
                if let EContentFileType::SkeletonMesh(content_skin_mesh) = file {
                    if &content_skin_mesh.borrow().url == skeleton_mesh {
                        skin_mesh =
                            resource_manager.get_skin_mesh(&content_skin_mesh.borrow().asset_url);
                        break;
                    }
                }
            }
            let Some(skin_mesh) = skin_mesh else {
                continue;
            };
            let Some(skeleton) = skeleton.clone() else {
                continue;
            };
            let Some(run_time) = self.run_time.as_mut() else {
                continue;
            };
            let mut model = self.transformation;
            if let Some((_, skeleton_mesh_hierarchy_node)) = skeleton
                .skeleton_mesh_hierarchy
                .iter()
                .find(|x| x.0.ends_with(&skin_mesh.name))
            {
                model = self.transformation * skeleton_mesh_hierarchy_node.transformation;
            }

            let mut draw_object;
            if let Some(material) = material.clone() {
                draw_object = engine.create_material_draw_object_from_skin_mesh(
                    &skin_mesh.vertexes,
                    &skin_mesh.indexes,
                    Some(format!("{} - {}", &self.name, &skin_mesh.name)),
                    material,
                    player_viewport.global_constants_handle.clone(),
                    player_viewport.point_lights_constants_handle.clone(),
                    player_viewport.spot_lights_constants_handle.clone(),
                );
            } else {
                draw_object = engine.create_draw_object_from_skin_mesh(
                    &skin_mesh.vertexes,
                    &skin_mesh.indexes,
                    Some(skin_mesh.name.clone()),
                    player_viewport.global_constants_handle.clone(),
                );
            }

            match &mut draw_object {
                EDrawObjectType::Skin(draw_object) => {
                    draw_object.constants.model = model;
                }
                EDrawObjectType::SkinMaterial(draw_object) => {
                    draw_object.constants.model = model;
                }
                _ => unimplemented!(),
            }
            run_time
                .draw_objects
                .insert(skin_mesh.name.clone(), draw_object);

            run_time.skin_meshes.push(skin_mesh);
        }

        let physics = Self::build_physics(
            self.run_time.as_mut().unwrap().skin_meshes.clone(),
            false,
            self.transformation,
        )
        .ok();
        self.run_time.as_mut().unwrap().physics = physics;
    }

    pub fn tick(&mut self, time: f32, engine: &mut Engine, level_physics: &mut LevelPhysics) {
        let _ = level_physics;
        let _ = engine;
        let Some(run_time) = self.run_time.as_mut() else {
            return;
        };
        let Some(skeleton) = run_time.skeleton.as_ref() else {
            return;
        };
        let mut node_anim_transforms: HashMap<String, glam::Mat4> = HashMap::new();

        if let Some(skeleton_animation_provider) = run_time.skeleton_animation_provider.as_mut() {
            skeleton_animation_provider.seek(time);
            node_anim_transforms = skeleton_animation_provider.transforms().clone();
        }

        let mut bones: [glam::Mat4; NUM_MAX_BONE] = [glam::Mat4::IDENTITY; NUM_MAX_BONE];
        let is_animated = run_time.skeleton_animation_provider.is_some();
        for skin_mesh in run_time.skin_meshes.clone() {
            if is_animated {
                bones.fill(glam::Mat4::IDENTITY);
                for (index, bone_path) in skin_mesh.bone_paths.iter().enumerate() {
                    if let Some(node_anim_transform) = node_anim_transforms.get(bone_path) {
                        bones[index] = *node_anim_transform;
                    }
                }
            }
            let draw_object = run_time.draw_objects.get_mut(&skin_mesh.name).unwrap();
            match draw_object {
                EDrawObjectType::Skin(draw_object) => {
                    draw_object.constants.bones.copy_from_slice(&bones);
                    let mut model = self.transformation;
                    if let Some((_, skeleton_mesh_hierarchy_node)) = skeleton
                        .skeleton_mesh_hierarchy
                        .iter()
                        .find(|x| x.0.ends_with(&skin_mesh.name))
                    {
                        model = self.transformation * skeleton_mesh_hierarchy_node.transformation;
                    }
                    draw_object.constants.model = model;
                }
                EDrawObjectType::SkinMaterial(draw_object) => {
                    draw_object.skin_constants.bones.copy_from_slice(&bones);
                    let mut model = self.transformation;
                    if let Some((_, skeleton_mesh_hierarchy_node)) = skeleton
                        .skeleton_mesh_hierarchy
                        .iter()
                        .find(|x| x.0.ends_with(&skin_mesh.name))
                    {
                        model = self.transformation * skeleton_mesh_hierarchy_node.transformation;
                    }
                    draw_object.constants.model = model;
                }
                _ => unimplemented!(),
            }
        }
    }

    pub fn get_draw_objects(&self) -> Vec<&EDrawObjectType> {
        match &self.run_time {
            Some(x) => x.draw_objects.values().map(|x| x).collect(),
            None => vec![],
        }
    }

    pub fn get_draw_objects_mut(&mut self) -> Vec<&mut EDrawObjectType> {
        match &mut self.run_time {
            Some(x) => x.draw_objects.values_mut().map(|x| x).collect(),
            None => vec![],
        }
    }

    pub fn set_material(
        &mut self,
        engine: &mut Engine,
        material_url: url::Url,
        files: &[EContentFileType],
        player_viewport: &mut PlayerViewport,
    ) {
        self.material_url = Some(material_url);
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

        let Some(run_time) = self.run_time.as_mut() else {
            return;
        };

        let Some(material) = material else {
            return;
        };
        for (name, draw_object) in &mut run_time.draw_objects {
            match draw_object {
                EDrawObjectType::SkinMaterial(material_draw_object) => {
                    material_draw_object.material = material.clone();
                }
                EDrawObjectType::Skin(_) => {
                    if let Some(skin_mesh) = run_time.skin_meshes.iter().find(|x| &x.name == name) {
                        *draw_object = engine.create_material_draw_object_from_skin_mesh(
                            &skin_mesh.vertexes,
                            &skin_mesh.indexes,
                            Some(format!("{} - {}", &self.name, &skin_mesh.name)),
                            material.clone(),
                            player_viewport.global_constants_handle.clone(),
                            player_viewport.point_lights_constants_handle.clone(),
                            player_viewport.spot_lights_constants_handle.clone(),
                        );
                    }
                }
                _ => unimplemented!(),
            }
        }
    }

    fn build_physics(
        meshes: Vec<Arc<SkinMesh>>,
        is_use_convex_decomposition: bool,
        transformation: glam::Mat4,
    ) -> crate::error::Result<Physics> {
        let (_, rotation, translation) = transformation.to_scale_rotation_translation();
        let (axis, angle) = rotation.to_axis_angle();
        let mut builder = RigidBodyBuilder::dynamic();
        builder = builder.translation(translation);
        builder.position.rotation = Rotation::from_axis_angle(axis.normalize(), angle);
        // builder = builder.enabled_rotations(false, false, false);
        let rigid_body = builder.build();
        let mut colliders = Vec::with_capacity(meshes.len());

        for mesh in meshes {
            let vertices: Vec<_> = mesh.vertexes.iter().map(|x| x.position).collect();
            let mut indices: Vec<_> = vec![];
            for index in mesh.indexes.chunks(3) {
                indices.push(
                    <[u32; 3]>::try_from(index).map_err(crate::error::Error::TryFromSliceError)?,
                );
            }
            let decomposed_shape = if is_use_convex_decomposition {
                SharedShape::convex_decomposition(&vertices, &indices)
            } else {
                SharedShape::trimesh_with_flags(vertices, indices, TriMeshFlags::FIX_INTERNAL_EDGES)
                    .map_err(|err| {
                        crate::error::Error::Other(Some(format!("Fail to build mesh, {}", err)))
                    })?
            };

            let collider = ColliderBuilder::new(decomposed_shape)
                .contact_skin(0.1)
                .active_events(ActiveEvents::COLLISION_EVENTS)
                .build();
            colliders.push(collider);
        }

        Ok(Physics {
            colliders,
            rigid_body,
            rigid_body_handle: RigidBodyHandle::invalid(),
            is_apply_simulate: true,
            collider_handles: vec![],
        })
    }

    pub fn initialize_physics(&mut self, level_physics: &mut LevelPhysics) {
        let Some(physics) = self.run_time.as_mut().map(|x| x.physics.as_mut()).flatten() else {
            return;
        };
        let handle = level_physics
            .rigid_body_set
            .insert(physics.rigid_body.clone());
        for collider in physics.colliders.clone() {
            let collider_handle = level_physics.collider_set.insert_with_parent(
                collider,
                handle,
                &mut level_physics.rigid_body_set,
            );
            physics.collider_handles.push(collider_handle);
        }
        physics.rigid_body_handle = handle;
    }

    pub fn update_physics(
        &mut self,
        rigid_body_set: &mut RigidBodySet,
        collider_set: &mut ColliderSet,
    ) {
        let Some(physics) = self.run_time.as_mut().map(|x| x.physics.as_mut()).flatten() else {
            return;
        };
        let Some(rigid_body) = rigid_body_set.get_mut(physics.rigid_body_handle) else {
            return;
        };
        for (handle, collider) in zip(physics.collider_handles.clone(), physics.colliders.clone()) {
            collider_set
                .get_mut(handle)
                .expect("Should not be null")
                .copy_from(&collider);
        }
        rigid_body.copy_from(&physics.rigid_body);
    }

    pub fn get_physics_mut(&mut self) -> Option<&mut Physics> {
        self.run_time.as_mut().map(|x| x.physics.as_mut()).flatten()
    }

    pub fn get_physics(&self) -> Option<&Physics> {
        self.run_time.as_ref().map(|x| x.physics.as_ref()).flatten()
    }

    pub fn set_animation(
        &mut self,
        animation: Option<url::Url>,
        resource_manager: ResourceManager,
        files: &[EContentFileType],
    ) {
        let _ = resource_manager;
        self.animation_url = animation;
        let animation_provider = self.find_animation_provider(files);
        let Some(run_time) = self.run_time.as_mut() else {
            return;
        };
        run_time.skeleton_animation_provider = animation_provider;
    }

    pub fn on_post_update_transformation(
        &mut self,
        level_physics: Option<&mut crate::content::level::LevelPhysics>,
    ) {
        let _ = level_physics;
    }

    pub fn on_post_update_animation(&mut self, files: &[EContentFileType]) {
        let skeleton_animation_provider = self.find_animation_provider(files);
        let Some(run_time) = self.run_time.as_mut() else {
            return;
        };
        run_time.skeleton_animation_provider = skeleton_animation_provider;
    }
}
