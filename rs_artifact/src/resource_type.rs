use crate::content_type::EContentType;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Deserialize, Serialize)]
pub enum EResourceType {
    IBLBaking,
    Image,
    StaticMesh,
    SkinMesh,
    SkeletonAnimation,
    ShaderSourceCode,
    Binary,
    Skeleton,
    Material,
    Sound,
    Content(EContentType),
    DeriveData,
}
