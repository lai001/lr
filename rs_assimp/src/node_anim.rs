use crate::{
    anim_behaviour::EAnimBehaviour, convert::ConvertToString, node::Node, quat_key::QuatKey,
    vector_key::VectorKey,
};
use rs_assimp_sys::*;
use std::{cell::RefCell, collections::HashMap, marker::PhantomData, rc::Rc};
use strum_macros::EnumIter;

#[derive(Debug, EnumIter, Clone, Copy, PartialEq, Eq)]
pub enum EAnimInterpolation {
    Step = aiAnimInterpolation_aiAnimInterpolation_Step as _,
    Linear = aiAnimInterpolation_aiAnimInterpolation_Linear as _,
    SphericalLinear = aiAnimInterpolation_aiAnimInterpolation_Spherical_Linear as _,
    CubicSpline = aiAnimInterpolation_aiAnimInterpolation_Cubic_Spline as _,
    Force32Bit = aiAnimInterpolation__aiAnimInterpolation_Force32Bit as _,
}

impl TryFrom<aiAnimInterpolation> for EAnimInterpolation {
    type Error = &'static str;

    fn try_from(ai_anim_interpolation: aiAnimInterpolation) -> Result<Self, Self::Error> {
        if ai_anim_interpolation == aiAnimInterpolation_aiAnimInterpolation_Step {
            Ok(EAnimInterpolation::Step)
        } else if ai_anim_interpolation == aiAnimInterpolation_aiAnimInterpolation_Linear {
            Ok(EAnimInterpolation::Linear)
        } else if ai_anim_interpolation == aiAnimInterpolation_aiAnimInterpolation_Spherical_Linear
        {
            Ok(EAnimInterpolation::SphericalLinear)
        } else if ai_anim_interpolation == aiAnimInterpolation_aiAnimInterpolation_Cubic_Spline {
            Ok(EAnimInterpolation::CubicSpline)
        } else if ai_anim_interpolation == aiAnimInterpolation__aiAnimInterpolation_Force32Bit {
            Ok(EAnimInterpolation::Force32Bit)
        } else {
            Err("Not a valid value.")
        }
    }
}

pub struct NodeAnim<'a> {
    _ai_node_anim: &'a mut aiNodeAnim,
    pub node: Option<Rc<RefCell<Node<'a>>>>,
    pub pre_state: EAnimBehaviour,
    pub post_state: EAnimBehaviour,
    pub position_keys: Vec<VectorKey<'a>>,
    pub scaling_keys: Vec<VectorKey<'a>>,
    pub rotation_keys: Vec<QuatKey<'a>>,
    marker: PhantomData<&'a ()>,
}

impl<'a> NodeAnim<'a> {
    pub fn borrow_from(
        ai_node_anim: &'a mut aiNodeAnim,
        map: &mut HashMap<String, Rc<RefCell<Node<'a>>>>,
    ) -> NodeAnim<'a> {
        let node_name: String = ai_node_anim.mNodeName.to_string();
        let pre_state = ai_node_anim.mPreState;
        let post_state = ai_node_anim.mPostState;
        let position_keys = unsafe {
            std::slice::from_raw_parts_mut(
                ai_node_anim.mPositionKeys,
                ai_node_anim.mNumPositionKeys as _,
            )
        };
        let position_keys = position_keys
            .iter_mut()
            .map(|x| VectorKey::borrow_from(x))
            .collect();

        let scaling_keys = unsafe {
            std::slice::from_raw_parts_mut(
                ai_node_anim.mScalingKeys,
                ai_node_anim.mNumScalingKeys as _,
            )
        };
        let scaling_keys = scaling_keys
            .iter_mut()
            .map(|x| VectorKey::borrow_from(x))
            .collect();

        let rotation_keys = unsafe {
            std::slice::from_raw_parts_mut(
                ai_node_anim.mRotationKeys,
                ai_node_anim.mNumRotationKeys as _,
            )
        };
        let rotation_keys = rotation_keys
            .iter_mut()
            .map(|x| QuatKey::borrow_from(x))
            .collect();

        let node = (|| {
            for node in map.values() {
                if node.borrow().name == node_name {
                    return Some(node.clone());
                }
            }
            None
        })();
        NodeAnim {
            _ai_node_anim: ai_node_anim,
            node,
            pre_state: pre_state.try_into().unwrap(),
            post_state: post_state.try_into().unwrap(),
            position_keys,
            scaling_keys,
            rotation_keys,
            marker: PhantomData,
        }
    }
}
