use crate::{
    content::content_file_type::EContentFileType, drawable::EDrawObjectType, engine::Engine,
    resource_manager::ResourceManager,
};
use rs_artifact::{
    skeleton::{Skeleton, SkeletonBone},
    skeleton_animation::SkeletonAnimation,
    skin_mesh::SkinMesh,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};

const NUM_MAX_BONE: usize = rs_render::global_shaders::skeleton_shading::NUM_MAX_BONE;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SceneComponent {
    pub name: String,
    pub transformation: glam::Mat4,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StaticMeshComponent {
    pub name: String,
    pub static_mesh: Option<url::Url>,
    pub transformation: glam::Mat4,
}

#[derive(Clone, Debug)]
struct SkeletonMeshComponentRuntime {
    draw_objects: HashMap<String, EDrawObjectType>,
    skeleton: Option<Arc<Skeleton>>,
    skeleton_animation: Option<Arc<SkeletonAnimation>>,
    skin_meshes: Vec<Arc<SkinMesh>>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SkeletonMeshComponent {
    pub name: String,
    pub skeleton_url: Option<url::Url>,
    pub skeleton_mesh_urls: Vec<url::Url>,
    pub animation_url: Option<url::Url>,
    pub transformation: glam::Mat4,
    #[serde(skip)]
    run_time: Option<SkeletonMeshComponentRuntime>,
}

impl SkeletonMeshComponent {
    pub fn new(
        name: String,
        skeleton_url: Option<url::Url>,
        skeleton_mesh_urls: Vec<url::Url>,
        animation_url: Option<url::Url>,
        transformation: glam::Mat4,
    ) -> SkeletonMeshComponent {
        SkeletonMeshComponent {
            name,
            skeleton_url,
            skeleton_mesh_urls,
            animation_url,
            transformation,
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
                log::trace!("{:?}", file);
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

        self.run_time = Some(SkeletonMeshComponentRuntime {
            draw_objects: HashMap::new(),
            skeleton: skeleton.clone(),
            skeleton_animation,
            skin_meshes: vec![],
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
            let mut model = glam::Mat4::IDENTITY;
            if let Some((_, skeleton_mesh_hierarchy_node)) = skeleton
                .skeleton_mesh_hierarchy
                .iter()
                .find(|x| x.0.ends_with(&skin_mesh.name))
            {
                model = skeleton_mesh_hierarchy_node.transformation;
            }

            let mut draw_object = engine.create_draw_object_from_skin_mesh(
                &skin_mesh.vertexes,
                &skin_mesh.indexes,
                Some(skin_mesh.name.clone()),
            );
            match &mut draw_object {
                EDrawObjectType::Static(_) => panic!(),
                EDrawObjectType::Skin(draw_object) => {
                    draw_object.constants.model = model;
                }
            }
            run_time
                .draw_objects
                .insert(skin_mesh.name.clone(), draw_object);

            self.run_time.as_mut().unwrap().skin_meshes.push(skin_mesh);
        }
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
        let Some(run_time) = self.run_time.as_mut() else {
            return;
        };
        let Some(skeleton) = run_time.skeleton.as_ref() else {
            return;
        };
        let Some(skeleton_animation) = run_time.skeleton_animation.as_ref() else {
            return;
        };
        let duration = skeleton_animation.duration / skeleton_animation.ticks_per_second;
        let animation_time = time % duration as f32;
        let mut node_anim_transforms: HashMap<String, glam::Mat4> = HashMap::new();
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

        for skin_mesh in run_time.skin_meshes.clone() {
            let mut bones: [glam::Mat4; NUM_MAX_BONE] = [glam::Mat4::IDENTITY; NUM_MAX_BONE];
            for (index, bone_path) in skin_mesh.bone_paths.iter().enumerate() {
                let node_anim_transform = node_anim_transforms.get(bone_path).unwrap();
                bones[index] = *node_anim_transform;
            }
            let draw_object = run_time.draw_objects.get_mut(&skin_mesh.name).unwrap();
            match draw_object {
                EDrawObjectType::Static(_) => panic!(),
                EDrawObjectType::Skin(draw_object) => {
                    draw_object.constants.bones.copy_from_slice(&bones);
                }
            }
            engine.update_draw_object(draw_object);
        }
    }

    pub fn get_draw_objects(&self) -> Vec<&EDrawObjectType> {
        match &self.run_time {
            Some(x) => x.draw_objects.values().map(|x| x).collect(),
            None => vec![],
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum EComponentType {
    SceneComponent(SceneComponent),
    StaticMeshComponent(StaticMeshComponent),
    SkeletonMeshComponent(SkeletonMeshComponent),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SceneNode {
    pub component: EComponentType,
    pub childs: Vec<SceneNode>,
}
