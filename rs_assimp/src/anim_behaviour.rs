use russimp_sys::*;
use strum_macros::EnumIter;

#[derive(Debug, EnumIter, Clone, Copy)]
pub enum EAnimBehaviour {
    Default = aiAnimBehaviour_aiAnimBehaviour_DEFAULT as _,
    Constant = aiAnimBehaviour_aiAnimBehaviour_CONSTANT as _,
    Linear = aiAnimBehaviour_aiAnimBehaviour_LINEAR as _,
    Repeat = aiAnimBehaviour_aiAnimBehaviour_REPEAT as _,
    Force32Bit = aiAnimBehaviour__aiAnimBehaviour_Force32Bit as _,
}

impl TryFrom<aiAnimBehaviour> for EAnimBehaviour {
    type Error = &'static str;

    fn try_from(ai_anim_behaviour: aiAnimBehaviour) -> Result<Self, Self::Error> {
        if ai_anim_behaviour == aiAnimBehaviour_aiAnimBehaviour_DEFAULT {
            Ok(EAnimBehaviour::Default)
        } else if ai_anim_behaviour == aiAnimBehaviour_aiAnimBehaviour_CONSTANT {
            Ok(EAnimBehaviour::Constant)
        } else if ai_anim_behaviour == aiAnimBehaviour_aiAnimBehaviour_LINEAR {
            Ok(EAnimBehaviour::Linear)
        } else if ai_anim_behaviour == aiAnimBehaviour_aiAnimBehaviour_REPEAT {
            Ok(EAnimBehaviour::Repeat)
        } else if ai_anim_behaviour == aiPrimitiveType__aiPrimitiveType_Force32Bit {
            Ok(EAnimBehaviour::Force32Bit)
        } else {
            Err("Not a valid value.")
        }
    }
}
