use crate::{
    content::content_file_type::EContentFileType, drawable::EDrawObjectType, engine::Engine,
    resource_manager::ResourceManager, static_mesh_component::Physics,
};
use rapier3d::{na::point, prelude::*};
use rs_artifact::{
    skeleton::{Skeleton, SkeletonBone},
    skeleton_animation::SkeletonAnimation,
    skin_mesh::SkinMesh,
};
use rs_render::global_shaders::skeleton_shading::NUM_MAX_BONE;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, iter::zip, sync::Arc};

#[derive(Clone)]
struct SkeletonMeshComponentRuntime {
    draw_objects: HashMap<String, EDrawObjectType>,
    skeleton: Option<Arc<Skeleton>>,
    skeleton_animation: Option<Arc<SkeletonAnimation>>,
    skin_meshes: Vec<Arc<SkinMesh>>,
    pub physics: Option<Physics>,
    pub final_transformation: glam::Mat4,
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

    pub fn initialize(
        &mut self,
        resource_manager: ResourceManager,
        engine: &mut Engine,
        files: &[EContentFileType],
    ) {
        let mut skeleton: Option<Arc<Skeleton>> = None;
        let mut skeleton_animation: Option<Arc<SkeletonAnimation>> = None;

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

        if let Some(animation_url) = &self.animation_url {
            for file in files.iter() {
                if let EContentFileType::SkeletonAnimation(content_skeleton_animation) = file {
                    if &content_skeleton_animation.borrow().url == animation_url {
                        skeleton_animation = resource_manager
                            .get_skeleton_animation(&content_skeleton_animation.borrow().asset_url);
                        break;
                    }
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

        self.run_time = Some(SkeletonMeshComponentRuntime {
            draw_objects: HashMap::new(),
            skeleton: skeleton.clone(),
            skeleton_animation,
            skin_meshes: vec![],
            physics: None,
            final_transformation: glam::Mat4::IDENTITY,
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
                    Some(skin_mesh.name.clone()),
                    material,
                );
            } else {
                draw_object = engine.create_draw_object_from_skin_mesh(
                    &skin_mesh.vertexes,
                    &skin_mesh.indexes,
                    Some(skin_mesh.name.clone()),
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

    fn walk_skeleton_bone(
        node_anim_transforms: &mut HashMap<String, glam::Mat4>,
        skeleton_bone: &SkeletonBone,
        bone_map: &HashMap<String, SkeletonBone>,
        skeleton_animation: Arc<SkeletonAnimation>,
        animation_time: f32,
        skeleton_mesh_hierarchy: &HashMap<String, rs_artifact::skeleton::SkeletonMeshHierarchyNode>,
        gather_parent_node_transformation: glam::Mat4,
        global_inverse_transform: glam::Mat4,
    ) {
        let mut position = glam::Vec3::ZERO;
        let mut scale = glam::Vec3::ONE;
        let mut rotation = glam::Quat::IDENTITY;
        let mut has_anim = false;
        let ticks_per_second = skeleton_animation.ticks_per_second;

        if let Some(animation) = skeleton_animation
            .channels
            .iter()
            .find(|x| x.node == skeleton_bone.path)
        {
            if animation.position_keys.len() == 1 {
                position = animation.position_keys[0].value;
            } else {
                if let Some(position_keys) =
                    animation.position_keys.windows(2).find(|position_keys| {
                        animation_time <= (position_keys[1].time / ticks_per_second) as f32
                    })
                {
                    let alpha = (animation_time
                        - (position_keys[0].time / ticks_per_second) as f32)
                        / ((position_keys[1].time / ticks_per_second)
                            - (position_keys[0].time / ticks_per_second))
                            as f32;
                    position = position_keys[0].value.lerp(position_keys[1].value, alpha);
                }
            }

            if animation.scaling_keys.len() == 1 {
                scale = animation.scaling_keys[0].value;
            } else {
                if let Some(scaling_keys) = animation.scaling_keys.windows(2).find(|scaling_keys| {
                    animation_time <= (scaling_keys[1].time / ticks_per_second) as f32
                }) {
                    let alpha = (animation_time - (scaling_keys[0].time / ticks_per_second) as f32)
                        / ((scaling_keys[1].time / ticks_per_second)
                            - (scaling_keys[0].time / ticks_per_second))
                            as f32;
                    scale = scaling_keys[0].value.lerp(scaling_keys[1].value, alpha);
                }
            }

            if animation.rotation_keys.len() == 1 {
                rotation = animation.rotation_keys[0].value;
            } else {
                if let Some(rotation_keys) =
                    animation.rotation_keys.windows(2).find(|rotation_keys| {
                        animation_time <= (rotation_keys[1].time / ticks_per_second) as f32
                    })
                {
                    let alpha = (animation_time
                        - (rotation_keys[0].time / ticks_per_second) as f32)
                        / ((rotation_keys[1].time / ticks_per_second)
                            - (rotation_keys[0].time / ticks_per_second))
                            as f32;
                    rotation = rotation_keys[0].value.slerp(rotation_keys[1].value, alpha);
                }
            }

            has_anim = true;
        }

        let mut attached_node_transformation = skeleton_mesh_hierarchy
            .get(&skeleton_bone.path)
            .unwrap()
            .transformation;

        let self_anim_transform =
            glam::Mat4::from_scale_rotation_translation(scale, rotation, position).transpose();
        if has_anim {
            attached_node_transformation = self_anim_transform;
        }

        let global_transform = attached_node_transformation * gather_parent_node_transformation;
        let bone_space_transformation =
            (skeleton_bone.offset_matrix * global_transform * global_inverse_transform).transpose();

        node_anim_transforms.insert(skeleton_bone.path.clone(), bone_space_transformation);

        for child in &skeleton_bone.childs {
            Self::walk_skeleton_bone(
                node_anim_transforms,
                bone_map.get(child).unwrap(),
                bone_map,
                skeleton_animation.clone(),
                animation_time,
                skeleton_mesh_hierarchy,
                global_transform,
                global_inverse_transform,
            );
        }
    }

    pub fn update(&mut self, time: f32, engine: &mut Engine) {
        let _ = engine;
        let Some(run_time) = self.run_time.as_mut() else {
            return;
        };
        let Some(skeleton) = run_time.skeleton.as_ref() else {
            return;
        };
        let mut node_anim_transforms: HashMap<String, glam::Mat4> = HashMap::new();

        if let Some(skeleton_animation) = run_time.skeleton_animation.as_ref() {
            let duration = skeleton_animation.duration / skeleton_animation.ticks_per_second;
            let animation_time = time % duration as f32;
            let root_bone = skeleton.bones.get(&skeleton.root_bone).unwrap();
            let global_inverse_transform = skeleton
                .skeleton_mesh_hierarchy
                .get(&skeleton.root_node)
                .unwrap()
                .transformation
                .inverse();

            Self::walk_skeleton_bone(
                &mut node_anim_transforms,
                root_bone,
                &skeleton.bones,
                skeleton_animation.clone(),
                animation_time,
                &skeleton.skeleton_mesh_hierarchy,
                skeleton
                    .skeleton_mesh_hierarchy
                    .get(&root_bone.path)
                    .unwrap()
                    .transformation,
                global_inverse_transform,
            );
        }

        for skin_mesh in run_time.skin_meshes.clone() {
            let mut bones: [glam::Mat4; NUM_MAX_BONE] = [glam::Mat4::IDENTITY; NUM_MAX_BONE];
            for (index, bone_path) in skin_mesh.bone_paths.iter().enumerate() {
                if let Some(node_anim_transform) = node_anim_transforms.get(bone_path) {
                    bones[index] = *node_anim_transform;
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

    pub fn submit_to_gpu(&mut self, engine: &mut Engine) {
        let Some(run_time) = &mut self.run_time else {
            return;
        };
        for skin_mesh in run_time.skin_meshes.clone() {
            if let Some(draw_object) = run_time.draw_objects.get_mut(&skin_mesh.name) {
                engine.update_draw_object(draw_object);
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
        for (_, draw_object) in &mut run_time.draw_objects {
            match draw_object {
                EDrawObjectType::SkinMaterial(material_draw_object) => {
                    material_draw_object.material = material.clone();
                }
                EDrawObjectType::Skin(_) => {
                    for skin_mesh in run_time.skin_meshes.clone() {
                        *draw_object = engine.create_material_draw_object_from_skin_mesh(
                            &skin_mesh.vertexes,
                            &skin_mesh.indexes,
                            Some(skin_mesh.name.clone()),
                            material.clone(),
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
        let translation = vector![translation.x, translation.y, translation.z];
        let (axis, angle) = rotation.to_axis_angle();
        let mut builder = RigidBodyBuilder::dynamic();
        builder = builder.translation(translation);
        builder.position.rotation = Rotation::from_axis_angle(
            &UnitVector::new_normalize(vector![axis.x, axis.y, axis.z]),
            angle,
        );
        // builder = builder.enabled_rotations(false, false, false);
        let rigid_body = builder.build();
        let mut colliders = Vec::with_capacity(meshes.len());

        for mesh in meshes {
            let vertices: Vec<_> = mesh
                .vertexes
                .iter()
                .map(|x| point![x.position.x, x.position.y, x.position.z])
                .collect();
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

    pub fn init_physics(
        &mut self,
        rigid_body_set: &mut RigidBodySet,
        collider_set: &mut ColliderSet,
    ) {
        let Some(physics) = self.run_time.as_mut().map(|x| x.physics.as_mut()).flatten() else {
            return;
        };
        let handle = rigid_body_set.insert(physics.rigid_body.clone());
        for collider in physics.colliders.clone() {
            let collider_handle = collider_set.insert_with_parent(collider, handle, rigid_body_set);
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
}
