use crate::{content::content_file_type::EContentFileType, resource_manager::ResourceManager};
use downcast_rs::Downcast;
use dyn_clone::DynClone;
use rs_artifact::{
    asset::Asset,
    skeleton::{Skeleton, SkeletonBone},
    skeleton_animation::SkeletonAnimation,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, iter::zip, sync::Arc};

pub trait SkeletonAnimationProvider: DynClone + Downcast {
    fn transforms(&mut self) -> &HashMap<String, glam::Mat4>;
    fn seek(&mut self, time: f32);

    fn walk_skeleton_bone(
        node_anim_transforms: &mut HashMap<String, glam::Mat4>,
        skeleton_bone: &SkeletonBone,
        bone_map: &HashMap<String, SkeletonBone>,
        skeleton_animation: Arc<SkeletonAnimation>,
        animation_time: f32,
        skeleton_mesh_hierarchy: &HashMap<String, rs_artifact::skeleton::SkeletonMeshHierarchyNode>,
        parent_global_transformation: glam::Mat4,
    ) where
        Self: Sized,
    {
        let mut translation = glam::Vec3::ZERO;
        let mut scale = glam::Vec3::ONE;
        let mut rotation = glam::Quat::IDENTITY;
        let mut has_anim = false;
        let ticks_per_second = skeleton_animation.ticks_per_second;

        if let Some(animation) = skeleton_animation
            .channels
            .iter()
            .find(|x| x.node == skeleton_bone.path)
        {
            let (src_scale, src_rotation, src_translation) =
                Self::calculate_animation(animation, animation_time, ticks_per_second);
            translation = src_translation;
            scale = src_scale;
            rotation = src_rotation;
            has_anim = true;
        }

        let global_transform = Self::post_calculate_animation(
            node_anim_transforms,
            skeleton_bone,
            skeleton_mesh_hierarchy,
            parent_global_transformation,
            scale,
            rotation,
            translation,
            has_anim,
        );

        for child in &skeleton_bone.childs {
            Self::walk_skeleton_bone(
                node_anim_transforms,
                bone_map.get(child).unwrap(),
                bone_map,
                skeleton_animation.clone(),
                animation_time,
                skeleton_mesh_hierarchy,
                global_transform,
            );
        }
    }

    fn calculate_animation(
        animation: &rs_artifact::node_anim::NodeAnim,
        animation_time: f32,
        ticks_per_second: f64,
    ) -> (glam::Vec3, glam::Quat, glam::Vec3)
    where
        Self: Sized,
    {
        let mut position = glam::Vec3::ZERO;
        let mut scale = glam::Vec3::ONE;
        let mut rotation = glam::Quat::IDENTITY;
        if animation.position_keys.len() == 1 {
            position = animation.position_keys[0].value;
        } else {
            if let Some(position_keys) = animation.position_keys.windows(2).find(|position_keys| {
                animation_time <= (position_keys[1].time / ticks_per_second) as f32
            }) {
                let alpha = (animation_time - (position_keys[0].time / ticks_per_second) as f32)
                    / ((position_keys[1].time / ticks_per_second)
                        - (position_keys[0].time / ticks_per_second)) as f32;
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
                        - (scaling_keys[0].time / ticks_per_second)) as f32;
                scale = scaling_keys[0].value.lerp(scaling_keys[1].value, alpha);
            }
        }

        if animation.rotation_keys.len() == 1 {
            rotation = animation.rotation_keys[0].value;
        } else {
            if let Some(rotation_keys) = animation.rotation_keys.windows(2).find(|rotation_keys| {
                animation_time <= (rotation_keys[1].time / ticks_per_second) as f32
            }) {
                let alpha = (animation_time - (rotation_keys[0].time / ticks_per_second) as f32)
                    / ((rotation_keys[1].time / ticks_per_second)
                        - (rotation_keys[0].time / ticks_per_second)) as f32;
                rotation = rotation_keys[0].value.slerp(rotation_keys[1].value, alpha);
            }
        }

        (scale, rotation, position)
    }

    fn post_calculate_animation(
        node_anim_transforms: &mut HashMap<String, glam::Mat4>,
        skeleton_bone: &SkeletonBone,
        skeleton_mesh_hierarchy: &HashMap<String, rs_artifact::skeleton::SkeletonMeshHierarchyNode>,
        parent_global_transformation: glam::Mat4,
        scale: glam::Vec3,
        rotation: glam::Quat,
        translation: glam::Vec3,
        has_anim: bool,
    ) -> glam::Mat4
    where
        Self: Sized,
    {
        let Some(node) = skeleton_mesh_hierarchy.get(&skeleton_bone.path) else {
            return glam::Mat4::IDENTITY;
        };
        let self_transformation = node.transformation;
        let anim_transform =
            glam::Mat4::from_scale_rotation_translation(scale, rotation, translation);

        let global_transform = parent_global_transformation
            * self_transformation
            * if has_anim {
                self_transformation.inverse()
            } else {
                glam::Mat4::IDENTITY
            }
            * anim_transform;

        node_anim_transforms.insert(
            skeleton_bone.path.clone(),
            global_transform * skeleton_bone.offset_matrix,
        );
        global_transform
    }

    fn walk_skeleton_bone_blend(
        node_anim_transforms: &mut HashMap<String, glam::Mat4>,
        skeleton_bone: &SkeletonBone,
        bone_map: &HashMap<String, SkeletonBone>,
        skeleton_animation_blends: Vec<SkeletonAnimationBlend>,
        animation_time: f32,
        skeleton_mesh_hierarchy: &HashMap<String, rs_artifact::skeleton::SkeletonMeshHierarchyNode>,
        parent_global_transformation: glam::Mat4,
    ) where
        Self: Sized,
    {
        let mut dest_translation = glam::Vec3::ZERO;
        let mut dest_scale = glam::Vec3::ONE;
        let mut dest_rotation = glam::Quat::IDENTITY;
        let mut has_anim = false;

        fn to_local_time(time: f32, time_range: std::ops::RangeInclusive<f32>) -> f32 {
            (time - *time_range.start()).clamp(*time_range.start(), *time_range.end())
        }

        for skeleton_animation_blend in skeleton_animation_blends.iter() {
            let ticks_per_second = skeleton_animation_blend.skeleton_animation.ticks_per_second;

            if let Some(animation) = skeleton_animation_blend
                .skeleton_animation
                .channels
                .iter()
                .find(|x| x.node == skeleton_bone.path)
            {
                let local_time =
                    to_local_time(animation_time, skeleton_animation_blend.time_range.clone());
                let local_total_duration = skeleton_animation_blend
                    .skeleton_animation
                    .duration_as_secs_f32();
                let local_time = local_time.clamp(0.0, local_total_duration);

                let (src_scale, src_rotation, src_translation) =
                    Self::calculate_animation(animation, local_time, ticks_per_second);
                match skeleton_animation_blend.blend_type {
                    SkeletonAnimationBlendType::Combine(factor) => {
                        dest_scale = dest_scale.lerp(src_scale, factor);
                        dest_rotation = dest_rotation.slerp(src_rotation, factor);
                        dest_translation = dest_translation.lerp(src_translation, factor);
                    }
                }
                has_anim = true;
            }
        }

        let global_transform = Self::post_calculate_animation(
            node_anim_transforms,
            skeleton_bone,
            skeleton_mesh_hierarchy,
            parent_global_transformation,
            dest_scale,
            dest_rotation,
            dest_translation,
            has_anim,
        );

        for child in &skeleton_bone.childs {
            Self::walk_skeleton_bone_blend(
                node_anim_transforms,
                bone_map.get(child).unwrap(),
                bone_map,
                skeleton_animation_blends.clone(),
                animation_time,
                skeleton_mesh_hierarchy,
                global_transform,
            );
        }
    }
}

downcast_rs::impl_downcast!(SkeletonAnimationProvider);
dyn_clone::clone_trait_object!(SkeletonAnimationProvider);

#[derive(Clone)]
pub struct SingleSkeletonAnimationProvider {
    skeleton_animation: Arc<SkeletonAnimation>,
    skeleton: Arc<Skeleton>,
    animation_time: f32,
    duration: f32,
    transforms: HashMap<String, glam::Mat4>,
}

impl SkeletonAnimationProvider for SingleSkeletonAnimationProvider {
    fn transforms(&mut self) -> &HashMap<String, glam::Mat4> {
        &self.transforms
    }

    fn seek(&mut self, time: f32) {
        for transform in self.transforms.values_mut() {
            *transform = glam::Mat4::IDENTITY;
        }
        self.animation_time = time % self.duration as f32;
        let root_bone = self.skeleton.bones.get(&self.skeleton.root_bone).unwrap();
        let parent_global_transformation = glam::Mat4::IDENTITY;
        Self::walk_skeleton_bone(
            &mut self.transforms,
            root_bone,
            &self.skeleton.bones,
            self.skeleton_animation.clone(),
            self.animation_time,
            &self.skeleton.skeleton_mesh_hierarchy,
            parent_global_transformation,
        );
    }
}

impl SingleSkeletonAnimationProvider {
    pub fn new(
        skeleton_animation: Arc<SkeletonAnimation>,
        skeleton: Arc<Skeleton>,
    ) -> SingleSkeletonAnimationProvider {
        let duration = skeleton_animation.duration / skeleton_animation.ticks_per_second;
        SingleSkeletonAnimationProvider {
            skeleton_animation,
            skeleton,
            animation_time: 0.0,
            duration: duration as f32,
            transforms: HashMap::new(),
        }
    }

    pub fn from(
        skeleton_url: &url::Url,
        animation_url: &url::Url,
        files: &[EContentFileType],
    ) -> Option<SingleSkeletonAnimationProvider> {
        let Some(animation_content) = files.iter().find_map(|x| {
            if let EContentFileType::SkeletonAnimation(x) = x {
                if &x.borrow().get_url() != animation_url {
                    return None;
                }
                return Some(x);
            } else {
                return None;
            };
        }) else {
            return None;
        };
        let Some(skeleton_content) = files.iter().find_map(|x| {
            if let EContentFileType::Skeleton(x) = x {
                if &x.borrow().get_url() != skeleton_url {
                    return None;
                }
                return Some(x);
            } else {
                return None;
            };
        }) else {
            return None;
        };
        let resource_manager = ResourceManager::default();
        let Some(skeleton) = resource_manager.get_skeleton(&skeleton_content.borrow().asset_url)
        else {
            return None;
        };
        let Some(skeleton_animation) =
            resource_manager.get_skeleton_animation(&animation_content.borrow().asset_url)
        else {
            return None;
        };
        Some(Self::new(skeleton_animation, skeleton))
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub enum SkeletonAnimationBlendType {
    Combine(f32),
}

#[derive(Clone)]
pub struct SkeletonAnimationBlend {
    blend_type: SkeletonAnimationBlendType,
    skeleton_animation: Arc<SkeletonAnimation>,
    time_range: std::ops::RangeInclusive<f32>,
}

impl SkeletonAnimationBlend {
    pub fn new(
        blend_type: SkeletonAnimationBlendType,
        skeleton_animation: Arc<SkeletonAnimation>,
        time_range: std::ops::RangeInclusive<f32>,
    ) -> SkeletonAnimationBlend {
        SkeletonAnimationBlend {
            blend_type,
            skeleton_animation,
            time_range,
        }
    }
}

#[derive(Clone)]
pub struct BlendSkeletonAnimationsProvider {
    skeleton_animations: Vec<SkeletonAnimationBlend>,
    skeleton: Arc<Skeleton>,
    animation_time: f32,
    duration: f32,
    transforms: HashMap<String, glam::Mat4>,
}

impl BlendSkeletonAnimationsProvider {
    pub fn from(
        skeleton_url: &url::Url,
        blend_animation_url: &url::Url,
        files: &[EContentFileType],
    ) -> Option<BlendSkeletonAnimationsProvider> {
        let Some(blend_animation_content) = files.iter().find_map(|x| {
            if let EContentFileType::BlendAnimations(x) = x {
                if &x.borrow().get_url() != blend_animation_url {
                    return None;
                }
                return Some(x);
            } else {
                return None;
            };
        }) else {
            return None;
        };

        let blend_animation = blend_animation_content.borrow();

        let Some(skeleton_content) = files.iter().find_map(|x| {
            if let EContentFileType::Skeleton(x) = x {
                if &x.borrow().get_url() != skeleton_url {
                    return None;
                }
                return Some(x);
            } else {
                return None;
            };
        }) else {
            return None;
        };
        let resource_manager = ResourceManager::default();

        let Some(skeleton) = resource_manager.get_skeleton(&skeleton_content.borrow().asset_url)
        else {
            return None;
        };

        let mut skeleton_animation_assets = Vec::with_capacity(blend_animation.channels.len());
        let mut skeleton_animation_blends: Vec<SkeletonAnimationBlend> =
            Vec::with_capacity(blend_animation.channels.len());

        for channel in blend_animation.channels.iter() {
            let search_url = &channel.animation_url;
            let find_animation_content = files.iter().find(|x| match x {
                EContentFileType::SkeletonAnimation(rc) => &rc.borrow().url == search_url,
                _ => false,
            });
            let Some(find_animation_content) = find_animation_content else {
                return None;
            };
            let EContentFileType::SkeletonAnimation(find_animation_content) =
                find_animation_content
            else {
                return None;
            };
            let find_animation_content = find_animation_content.borrow();
            let skeleton_animation_asset =
                resource_manager.get_skeleton_animation(&find_animation_content.asset_url);
            let Some(skeleton_animation_asset) = skeleton_animation_asset else {
                return None;
            };
            skeleton_animation_assets.push(skeleton_animation_asset);
        }

        for (skeleton_animation_asset, channel) in
            zip(skeleton_animation_assets, &blend_animation.channels)
        {
            let skeleton_animation_blend = SkeletonAnimationBlend::new(
                channel.blend_type.clone(),
                skeleton_animation_asset.clone(),
                channel.time_range.clone(),
            );
            skeleton_animation_blends.push(skeleton_animation_blend);
        }

        Some(BlendSkeletonAnimationsProvider::new(
            skeleton_animation_blends,
            skeleton,
        ))
    }

    pub fn new(
        skeleton_animation_blends: Vec<SkeletonAnimationBlend>,
        skeleton: Arc<Skeleton>,
    ) -> Self {
        let mut duration: f32 = 0.0;
        for skeleton_animation in skeleton_animation_blends.iter() {
            assert!(*skeleton_animation.time_range.start() >= 0.0);
            duration = duration.max(*skeleton_animation.time_range.end());
        }
        Self {
            skeleton_animations: skeleton_animation_blends,
            skeleton,
            animation_time: 0.0,
            duration,
            transforms: HashMap::new(),
        }
    }
}

impl SkeletonAnimationProvider for BlendSkeletonAnimationsProvider {
    fn transforms(&mut self) -> &HashMap<String, glam::Mat4> {
        &self.transforms
    }

    fn seek(&mut self, time: f32) {
        for transform in self.transforms.values_mut() {
            *transform = glam::Mat4::IDENTITY;
        }
        self.animation_time = time % self.duration as f32;

        let root_bone = self.skeleton.bones.get(&self.skeleton.root_bone).unwrap();
        let parent_global_transformation = glam::Mat4::IDENTITY;

        Self::walk_skeleton_bone_blend(
            &mut self.transforms,
            root_bone,
            &self.skeleton.bones,
            self.skeleton_animations.clone(),
            self.animation_time,
            &self.skeleton.skeleton_mesh_hierarchy,
            parent_global_transformation,
        );
    }
}
