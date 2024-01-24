#[derive(Debug)]
pub enum EBuildPlatformType {
    Windows,
}

#[derive(Debug)]
pub enum EBuildType {
    Debug,
    Release,
}

#[derive(Debug)]
pub enum EArchType {
    X64,
}

#[derive(Debug)]
pub struct BuildConfig {
    pub build_platform: EBuildPlatformType,
    pub build_type: EBuildType,
    pub arch_type: EArchType,
}

impl Default for BuildConfig {
    fn default() -> Self {
        Self {
            build_platform: EBuildPlatformType::Windows,
            build_type: EBuildType::Debug,
            arch_type: EArchType::X64,
        }
    }
}
