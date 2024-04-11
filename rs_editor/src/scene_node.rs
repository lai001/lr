use rs_artifact::{
    skeleton::{Skeleton, SkeletonBone},
    skeleton_animation::SkeletonAnimation,
    skin_mesh::SkinMesh,
};
use rs_engine::{engine::Engine, resource_manager::ResourceManager};
use rs_render::command::{DrawObject, EMaterialType, PhongMaterial};
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

#[derive(Clone)]
struct SkeletonMeshComponentRuntime {
    draw_objects: HashMap<String, DrawObject>,
    skeleton: Option<Arc<Skeleton>>,
    skeleton_animation: Option<Arc<SkeletonAnimation>>,
    skin_meshes: Vec<Arc<SkinMesh>>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SkeletonMeshComponent {
    pub name: String,
    pub skeleton: Option<url::Url>,
    pub skeleton_meshes: Vec<url::Url>,
    pub animation: Option<url::Url>,
    pub transformation: glam::Mat4,
    #[serde(skip)]
    run_time: Option<SkeletonMeshComponentRuntime>,
}

impl SkeletonMeshComponent {
    pub fn new(
        name: String,
        skeleton: Option<url::Url>,
        skeleton_meshes: Vec<url::Url>,
        animation: Option<url::Url>,
        transformation: glam::Mat4,
    ) -> SkeletonMeshComponent {
        SkeletonMeshComponent {
            name,
            skeleton,
            skeleton_meshes,
            animation,
            transformation,
            run_time: None,
        }
    }

    pub fn initialize(&mut self, resource_manager: ResourceManager, engine: &mut Engine) {
        let mut skeleton: Option<Arc<Skeleton>> = None;
        let mut skeleton_animation: Option<Arc<SkeletonAnimation>> = None;

        if let Some(skeleton_url) = &self.skeleton {
            skeleton = resource_manager.get_skeleton(&skeleton_url);
        }

        if let Some(animation_url) = &self.animation {
            skeleton_animation = resource_manager.get_skeleton_animation(&animation_url);
        }

        self.run_time = Some(SkeletonMeshComponentRuntime {
            draw_objects: HashMap::new(),
            skeleton: skeleton.clone(),
            skeleton_animation,
            skin_meshes: vec![],
        });

        for skeleton_mesh in &self.skeleton_meshes {
            let Some(skin_mesh) = resource_manager.get_skin_mesh(skeleton_mesh) else {
                continue;
            };
            let Some(skeleton) = skeleton.clone() else {
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

            let material_type = EMaterialType::Phong(PhongMaterial {
                constants: rs_render::render_pipeline::phong_pipeline::Constants {
                    model,
                    view: glam::Mat4::IDENTITY,
                    projection: glam::Mat4::IDENTITY,
                },
                diffuse_texture: None,
                specular_texture: None,
            });

            let draw_object = engine.create_draw_object_from_skin_mesh(
                &skin_mesh.vertexes,
                &skin_mesh.indexes,
                material_type,
            );
            self.run_time
                .as_mut()
                .unwrap()
                .draw_objects
                .insert(skin_mesh.name.clone(), draw_object);
            self.run_time.as_mut().unwrap().skin_meshes.push(skin_mesh);
        }
    }

    pub fn camera_did_update(&mut self, view: glam::Mat4, projection: glam::Mat4) {
        let Some(run_time) = self.run_time.as_mut() else {
            return;
        };
        for (_, draw_object) in run_time.draw_objects.iter_mut() {
            match &mut draw_object.material_type {
                EMaterialType::Phong(material) => {
                    material.constants.view = view;
                    material.constants.projection = projection;
                }
                EMaterialType::PBR(_) => todo!(),
            }
        }
    }

    fn walk_skeleton_bone(
        node_anim_transforms: &mut HashMap<String, glam::Mat4>,
        skeleton_bone: &SkeletonBone,
        bone_map: &HashMap<String, SkeletonBone>,
        skeleton_animation: Arc<SkeletonAnimation>,
        animation_time: f32,
    ) {
        let mut position = glam::Vec3::ZERO;
        let mut scale = glam::Vec3::ONE;
        let mut rotation = glam::Quat::IDENTITY;

        let animation_time = animation_time * 60.0 % skeleton_animation.duration as f32;
        if let Some(animation) = skeleton_animation
            .channels
            .iter()
            .find(|x| x.node == skeleton_bone.path)
        {
            if animation.position_keys.len() == 1 {
                position = animation.position_keys[0].value;
            } else {
                if let Some(position_keys) = animation
                    .position_keys
                    .windows(2)
                    .find(|position_keys| animation_time <= position_keys[1].time as f32)
                {
                    let alpha = (animation_time - position_keys[0].time as f32)
                        / (position_keys[1].time - position_keys[0].time) as f32;
                    position = position_keys[0].value.lerp(position_keys[1].value, alpha);
                }
            }

            if animation.scaling_keys.len() == 1 {
                scale = animation.scaling_keys[0].value;
            } else {
                if let Some(scaling_keys) = animation.scaling_keys.windows(2).find(|scaling_keys| {
                    animation_time >= scaling_keys[0].time as f32
                        && animation_time <= scaling_keys[1].time as f32
                }) {
                    let alpha = (animation_time - scaling_keys[0].time as f32)
                        / (scaling_keys[1].time - scaling_keys[0].time) as f32;
                    scale = scaling_keys[0].value.lerp(scaling_keys[1].value, alpha);
                }
            }

            if animation.rotation_keys.len() == 1 {
                rotation = animation.rotation_keys[0].value;
            } else {
                if let Some(rotation_keys) =
                    animation.rotation_keys.windows(2).find(|rotation_keys| {
                        animation_time >= rotation_keys[0].time as f32
                            && animation_time <= rotation_keys[1].time as f32
                    })
                {
                    let alpha = (animation_time - rotation_keys[0].time as f32)
                        / (rotation_keys[1].time - rotation_keys[0].time) as f32;
                    rotation = rotation_keys[0].value.lerp(rotation_keys[1].value, alpha);
                }
            }
        }

        let final_transform = if let Some(parent) = &skeleton_bone.parent {
            *node_anim_transforms.get(parent).unwrap()
                * glam::Mat4::from_scale_rotation_translation(scale, rotation, position)
        } else {
            glam::Mat4::from_scale_rotation_translation(scale, rotation, position)
        };

        node_anim_transforms.insert(skeleton_bone.path.clone(), final_transform);

        for child in &skeleton_bone.childs {
            Self::walk_skeleton_bone(
                node_anim_transforms,
                bone_map.get(child).unwrap(),
                bone_map,
                skeleton_animation.clone(),
                animation_time,
            );
        }
    }

    pub fn update(&mut self, time: f32) {
        let Some(run_time) = self.run_time.as_mut() else {
            return;
        };
        let Some(skeleton) = run_time.skeleton.as_ref() else {
            return;
        };
        let Some(skeleton_animation) = run_time.skeleton_animation.as_ref() else {
            return;
        };
        let duration = skeleton_animation.duration;
        let animation_time = time % duration as f32;
        let mut node_anim_transforms: HashMap<String, glam::Mat4> = HashMap::new();
        let root_bone = skeleton.bones.get(&skeleton.root_bone).unwrap();
        Self::walk_skeleton_bone(
            &mut node_anim_transforms,
            root_bone,
            &skeleton.bones,
            skeleton_animation.clone(),
            animation_time,
        );

        for skin_mesh in run_time.skin_meshes.clone() {
            let mut bones: [glam::Mat4; NUM_MAX_BONE] = [glam::Mat4::IDENTITY; NUM_MAX_BONE];
            for (index, bone_path) in skin_mesh.bone_paths.iter().enumerate() {
                let node_anim_transform = node_anim_transforms.get(bone_path).unwrap();
                bones[index] = *node_anim_transform;
            }
            let draw_object = run_time.draw_objects.get_mut(&skin_mesh.name).unwrap();
            draw_object.bones = Some(bones.clone());
        }
    }

    pub fn get_draw_objects(&self) -> Vec<DrawObject> {
        match &self.run_time {
            Some(x) => x.draw_objects.values().map(|x| x.clone()).collect(),
            None => vec![],
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub enum EComponentType {
    SceneComponent(SceneComponent),
    StaticMeshComponent(StaticMeshComponent),
    SkeletonMeshComponent(SkeletonMeshComponent),
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SceneNode {
    pub component: EComponentType,
    pub childs: Vec<SceneNode>,
}
