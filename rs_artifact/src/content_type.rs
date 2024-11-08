use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Deserialize, Serialize)]
pub enum EContentType {
    StaticMesh,
    SkeletonMesh,
    SkeletonAnimation,
    Skeleton,
    Texture,
    Level,
    Material,
    IBL,
    MediaSource,
    ParticleSystem,
    Sound,
    Curve,
    BlendAnimations,
    MaterialParamentersCollection,
}
