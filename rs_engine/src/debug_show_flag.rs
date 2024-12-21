use bitflags::bitflags;

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct DebugShowFlag: u32 {
        const CameraFrustum = 1;
        const PointLightSphere = 1 << 1;
    }
}
