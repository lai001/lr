use crate::{node::Node, skeleton_bone::SkeletonBone};
use std::{cell::RefCell, collections::HashMap, marker::PhantomData, rc::Rc};

pub struct Skeleton<'a> {
    _ai_skeleton: &'a mut russimp_sys::aiSkeleton,
    pub name: String,
    pub bones: Vec<SkeletonBone<'a>>,
    marker: PhantomData<&'a ()>,
}

impl<'a> Skeleton<'a> {
    pub fn borrow_from(
        ai_skeleton: &'a mut russimp_sys::aiSkeleton,
        map: &HashMap<String, Rc<RefCell<Node<'a>>>>,
    ) -> Skeleton<'a> {
        let name = ai_skeleton.mName.into();
        let mut bones = vec![];
        if ai_skeleton.mBones.is_null() == false {
            let ai_bones = unsafe {
                std::slice::from_raw_parts(ai_skeleton.mBones, ai_skeleton.mNumBones as _)
            };
            for ai_bone in ai_bones {
                let bone = SkeletonBone::borrow_from(unsafe { ai_bone.as_mut().unwrap() }, map);
                bones.push(bone);
            }
        }
        Skeleton {
            _ai_skeleton: ai_skeleton,
            name,
            bones,
            marker: PhantomData,
        }
    }
}
