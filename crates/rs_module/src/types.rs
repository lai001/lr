use rs_foundation::dyn_cast::DynCast;
use std::borrow::Cow;

pub enum ModuleType {
    Editor,
    Standalone,
}

pub trait Module: DynCast {
    fn display_name(&self) -> Cow<'static, str>;
    fn module_type(&self) -> ModuleType;
}

#[cfg(feature = "editor")]
pub trait EditorModule: Module {}

pub struct ModuleManager {
    modules: Vec<Box<dyn Module>>,
}

impl ModuleManager {
    pub fn new() -> Self {
        let modules = vec![];
        Self { modules }
    }

    pub fn modules(&self) -> &[Box<dyn Module + 'static>] {
        &self.modules
    }

    pub fn modules_mut(&mut self) -> &mut Vec<Box<dyn Module>> {
        &mut self.modules
    }

    pub fn set_modules(&mut self, modules: Vec<Box<dyn Module>>) {
        self.modules = modules;
    }
}
