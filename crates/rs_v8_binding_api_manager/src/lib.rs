pub struct BindingApiManager {
    pub engine_api: rs_v8_engine_binding_api::native_engine::EngineBindingApi,
    pub level_api: rs_v8_engine_binding_api::native_level::RcRefLevelBindingApi,
    pub player_viewport_binding_api:
        rs_v8_engine_binding_api::native_player_viewport::PlayerViewportBindingApi,
}

impl BindingApiManager {
    pub fn new(
        engine_api: rs_v8_engine_binding_api::native_engine::EngineBindingApi,
        level_api: rs_v8_engine_binding_api::native_level::RcRefLevelBindingApi,
        player_viewport_binding_api: rs_v8_engine_binding_api::native_player_viewport::PlayerViewportBindingApi,
    ) -> Self {
        Self {
            engine_api,
            level_api,
            player_viewport_binding_api,
        }
    }
}
