use bitflags::bitflags;
use russimp_sys::*;

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct PostProcessSteps: u32 {
        const CalcTangentSpace = aiPostProcessSteps_aiProcess_CalcTangentSpace as _;
        const JoinIdenticalVertices = aiPostProcessSteps_aiProcess_JoinIdenticalVertices as _;
        const MakeLeftHanded = aiPostProcessSteps_aiProcess_MakeLeftHanded as _;
        const Triangulate = aiPostProcessSteps_aiProcess_Triangulate as _;
        const RemoveComponent = aiPostProcessSteps_aiProcess_RemoveComponent as _;
        const GenNormals = aiPostProcessSteps_aiProcess_GenNormals as _;
        const GenSmoothNormals = aiPostProcessSteps_aiProcess_GenSmoothNormals as _;
        const SplitLargeMeshes = aiPostProcessSteps_aiProcess_SplitLargeMeshes as _;
        const PreTransformVertices = aiPostProcessSteps_aiProcess_PreTransformVertices as _;
        const LimitBoneWeights = aiPostProcessSteps_aiProcess_LimitBoneWeights as _;
        const ValidateDataStructure = aiPostProcessSteps_aiProcess_ValidateDataStructure as _;
        const ImproveCacheLocality = aiPostProcessSteps_aiProcess_ImproveCacheLocality as _;
        const RemoveRedundantMaterials = aiPostProcessSteps_aiProcess_RemoveRedundantMaterials as _;
        const FixInfacingNormals = aiPostProcessSteps_aiProcess_FixInfacingNormals as _;
        const PopulateArmatureData = aiPostProcessSteps_aiProcess_PopulateArmatureData as _;
        const SortByPType = aiPostProcessSteps_aiProcess_SortByPType as _;
        const FindDegenerates = aiPostProcessSteps_aiProcess_FindDegenerates as _;
        const FindInvalidData = aiPostProcessSteps_aiProcess_FindInvalidData as _;
        const GenUVCoords = aiPostProcessSteps_aiProcess_GenUVCoords as _;
        const TransformUVCoords = aiPostProcessSteps_aiProcess_TransformUVCoords as _;
        const FindInstances = aiPostProcessSteps_aiProcess_FindInstances as _;
        const OptimizeMeshes = aiPostProcessSteps_aiProcess_OptimizeMeshes as _;
        const OptimizeGraph = aiPostProcessSteps_aiProcess_OptimizeGraph as _;
        const FlipUVs = aiPostProcessSteps_aiProcess_FlipUVs as _;
        const FlipWindingOrder = aiPostProcessSteps_aiProcess_FlipWindingOrder as _;
        const SplitByBoneCount = aiPostProcessSteps_aiProcess_SplitByBoneCount as _;
        const Debone = aiPostProcessSteps_aiProcess_Debone as _;
        const GlobalScale = aiPostProcessSteps_aiProcess_GlobalScale as _;
        const EmbedTextures = aiPostProcessSteps_aiProcess_EmbedTextures as _;
        const ForceGenNormals = aiPostProcessSteps_aiProcess_ForceGenNormals as _;
        const DropNormals = aiPostProcessSteps_aiProcess_DropNormals as _;
        const GenBoundingBoxes = aiPostProcessSteps_aiProcess_GenBoundingBoxes as _;
    }
}
