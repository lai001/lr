pub mod color;
pub mod error;
pub mod file_manager;
pub mod frustum;
pub mod line_3d;
pub mod misc;
pub mod name_generator;
pub mod parallel;
pub mod path_ext;
pub mod plane_3d;
pub mod primitive_data;
pub mod serde_user_data;
pub mod settings;
pub mod sphere_3d;
pub mod thread_pool;

#[macro_export(local_inner_macros)]
macro_rules! vec_ref {
    ($var_name:tt, $source_name:ident) => {
        let __l = $source_name.len();
        let mut __s = Vec::with_capacity(__l);
        for item in &$source_name {
            let reference = item.borrow();
            __s.push(reference);
        }
        let mut $var_name = Vec::with_capacity(__l);
        for item in &__s {
            $var_name.push(item.deref());
        }
    };
}
