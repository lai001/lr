use russimp_sys::*;

pub enum EPrimitiveType {
    Line,
    Triangle,
    Polygon,
    Point,
    NgonEncodingFlag,
}

impl EPrimitiveType {
    pub fn from(ai_primitive_type: russimp_sys::aiPrimitiveType) -> Option<EPrimitiveType> {
        if ai_primitive_type == aiPrimitiveType_aiPrimitiveType_LINE {
            Some(EPrimitiveType::Line)
        } else if ai_primitive_type == aiPrimitiveType_aiPrimitiveType_POINT {
            Some(EPrimitiveType::Point)
        } else if ai_primitive_type == aiPrimitiveType_aiPrimitiveType_POLYGON {
            Some(EPrimitiveType::Polygon)
        } else if ai_primitive_type == aiPrimitiveType_aiPrimitiveType_TRIANGLE {
            Some(EPrimitiveType::Triangle)
        } else if ai_primitive_type == aiPrimitiveType_aiPrimitiveType_NGONEncodingFlag {
            Some(EPrimitiveType::NgonEncodingFlag)
        } else {
            None
        }
    }
}
