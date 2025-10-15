use rs_v8_host::v8_runtime::V8Runtime;

pub struct BindingApi {}

impl BindingApi {
    pub fn register(v8_runtime: &mut V8Runtime) {
        rs_engine_v8_binding_api::register(v8_runtime);
    }
}
