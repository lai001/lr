use rs_artifact::material::MaterialInfo;
use rs_module::types::EditorModule;
use rs_render_types::MaterialOptions;

pub trait CreationTask: Send + Sync {
    fn run(&mut self) -> crate::error::Result<(String, MaterialInfo)>;
}

pub trait MaterialCreationProxyModule: EditorModule {
    fn create_task(
        &mut self,
        material_url: url::Url,
        options: MaterialOptions,
        material_paramenters: crate::material::Paramenters,
    ) -> Option<Box<dyn CreationTask>>;
}

rs_foundation::dyn_cast_wrapper!(MaterialCreationProxyModule);
