use russimp_sys::*;
use strum_macros::EnumIter;

#[derive(Debug, EnumIter, Clone, Copy)]
pub enum EPrimitiveType {
    Line = aiPrimitiveType_aiPrimitiveType_LINE as _,
    Triangle = aiPrimitiveType_aiPrimitiveType_TRIANGLE as _,
    Polygon = aiPrimitiveType_aiPrimitiveType_POLYGON as _,
    Point = aiPrimitiveType_aiPrimitiveType_POINT as _,
    NgonEncodingFlag = aiPrimitiveType_aiPrimitiveType_NGONEncodingFlag as _,
    Force32Bit = aiPrimitiveType__aiPrimitiveType_Force32Bit as _,
}

impl TryFrom<aiPrimitiveType> for EPrimitiveType {
    type Error = &'static str;

    fn try_from(ai_primitive_type: aiPrimitiveType) -> Result<Self, Self::Error> {
        if ai_primitive_type == aiPrimitiveType_aiPrimitiveType_LINE {
            Ok(EPrimitiveType::Line)
        } else if ai_primitive_type == aiPrimitiveType_aiPrimitiveType_POINT {
            Ok(EPrimitiveType::Point)
        } else if ai_primitive_type == aiPrimitiveType_aiPrimitiveType_POLYGON {
            Ok(EPrimitiveType::Polygon)
        } else if ai_primitive_type == aiPrimitiveType_aiPrimitiveType_TRIANGLE {
            Ok(EPrimitiveType::Triangle)
        } else if ai_primitive_type == aiPrimitiveType_aiPrimitiveType_NGONEncodingFlag {
            Ok(EPrimitiveType::NgonEncodingFlag)
        } else if ai_primitive_type == aiPrimitiveType__aiPrimitiveType_Force32Bit {
            Ok(EPrimitiveType::Force32Bit)
        } else {
            Err("Not a valid value.")
        }
    }
}
