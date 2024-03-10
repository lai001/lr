pub mod bone;
pub mod convert;
pub mod error;
pub mod face;
pub mod material;
pub mod mesh;
pub mod metadata;
pub mod node;
pub mod post_process_steps;
pub mod primitive_type;
pub mod scene;
pub mod texture_type;
pub mod vertex_weight;

fn get_assimp_error() -> crate::error::AssimpError {
    unsafe {
        let error_buf = russimp_sys::aiGetErrorString();
        let error = std::ffi::CStr::from_ptr(error_buf)
            .to_string_lossy()
            .to_string();
        crate::error::AssimpError::Import(error)
    }
}
