use super::types::{PostLoading, PostLoadingContext, PreLoadingContext};
use crate::editor_context::EditorContext;
use crate::impl_default_load_future;
use crate::impl_default_load_future_body;
use crate::ui::material_view::MaterialNode;
use rs_artifact::bincode_legacy;
use rs_engine::thread_pool::ThreadPool;
use rs_foundation::new::SingleThreadMutType;
use rs_render_types::MaterialOptions;
use std::collections::HashMap;
use std::hash::Hash;

struct SnarlWrapper {
    url: url::Url,
    value: Vec<u8>,
}

impl PartialEq for SnarlWrapper {
    fn eq(&self, other: &Self) -> bool {
        self.url == other.url
    }
}
impl Eq for SnarlWrapper {}

impl Hash for SnarlWrapper {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.url.hash(state);
    }
}

impl SnarlWrapper {
    fn copy_snarl_from(url: url::Url, snarl: &egui_snarl::Snarl<MaterialNode>) -> SnarlWrapper {
        let value = bincode_legacy::serialize(snarl, None).unwrap();
        SnarlWrapper { url, value }
    }

    fn get(&self) -> egui_snarl::Snarl<MaterialNode> {
        let value = bincode_legacy::deserialize(&self.value, None).unwrap();
        value
    }
}

unsafe impl Send for SnarlWrapper {}

type ResultType =
    Result<HashMap<MaterialOptions, crate::material_resolve::ResolveResult>, anyhow::Error>;

pub struct LoadMaterial<'a> {
    _loading_context: PreLoadingContext<'a>,
    material_content: SingleThreadMutType<rs_engine::content::material::Material>,
    material_editor: SingleThreadMutType<crate::material::Material>,
    receiver: std::sync::mpsc::Receiver<ResultType>,
    resource: Option<ResultType>,
}

impl_default_load_future!(LoadMaterial<'a>);

impl<'a> PostLoading for LoadMaterial<'a> {
    fn on_loading_finished(&mut self, context: PostLoadingContext) {
        let resolve_result = match self.resource.take().expect("Not null") {
            Ok(resolve_result) => resolve_result,
            Err(err) => {
                log::warn!("{}", err);
                return;
            }
        };
        let PostLoadingContext { engine, .. } = context;
        let mut shader_code: HashMap<MaterialOptions, String> = HashMap::new();
        let mut material_info: HashMap<MaterialOptions, rs_artifact::material::MaterialInfo> =
            HashMap::new();
        if engine
            .get_settings()
            .render_setting
            .is_enable_dump_material_shader_code
        {
            if let Err(err) = EditorContext::write_debug_shader(
                &self.material_editor.clone().borrow(),
                &resolve_result,
            ) {
                log::warn!("{}", err);
            }
        }
        for (option, result) in resolve_result {
            shader_code.insert(option.clone(), result.shader_code);
            material_info.insert(option, result.material_info);
        }
        {
            let pipeline_handle = engine.create_material(shader_code);
            let mut material_content = self.material_content.borrow_mut();
            material_content.set_pipeline_handle(pipeline_handle);
            material_content.set_material_info(material_info);
        }
        self.material_editor
            .borrow_mut()
            .set_associated_material(self.material_content.clone());
    }
}

impl<'a> LoadMaterial<'a> {
    pub fn new(
        loading_context: PreLoadingContext<'a>,
        material_content: SingleThreadMutType<rs_engine::content::material::Material>,
    ) -> Option<LoadMaterial<'a>> {
        let url = material_content.borrow().asset_url.clone();
        let Some(material_editor) = loading_context
            .project_context
            .project
            .materials
            .iter()
            .find(|x| x.borrow().url == url)
            .cloned()
        else {
            return None;
        };
        let snarl_wrapper = SnarlWrapper::copy_snarl_from(url, &material_editor.borrow().snarl);
        let (sender, receiver) = std::sync::mpsc::channel();
        ThreadPool::global().spawn({
            move || {
                let resolve_result =
                    crate::material_resolve::resolve(&snarl_wrapper.get(), MaterialOptions::all());
                let _ = sender.send(resolve_result);
            }
        });
        Some(Self {
            material_content,
            material_editor,
            receiver,
            _loading_context: loading_context,
            resource: None,
        })
    }
}
