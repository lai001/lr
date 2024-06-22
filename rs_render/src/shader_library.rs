use super::reflection::Reflection;
use crate::command::MaterialRenderPipelineHandle;
use pollster::FutureExt;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

pub struct ShaderLibrary {
    shader_dic: HashMap<String, Arc<wgpu::ShaderModule>>,
    reflection_dic: HashMap<String, Arc<Reflection>>,
}

impl ShaderLibrary {
    pub fn new() -> ShaderLibrary {
        ShaderLibrary {
            shader_dic: HashMap::new(),
            reflection_dic: HashMap::new(),
        }
    }

    pub fn load_shaders_from<K>(
        &mut self,
        shaders: HashMap<K, String>,
        device: &wgpu::Device,
    ) -> HashMap<String, crate::error::Result<()>>
    where
        K: AsRef<str>,
    {
        let mut results: HashMap<String, crate::error::Result<()>> = HashMap::new();
        for (name, code) in shaders {
            let result = self.load_shader_from(name.as_ref(), code.as_str(), device);
            results.insert(name.as_ref().to_string(), result);
        }
        results
    }

    pub fn load_shader_from<K>(
        &mut self,
        name: K,
        code: K,
        device: &wgpu::Device,
    ) -> crate::error::Result<()>
    where
        K: AsRef<str>,
    {
        let (shader_module, reflection) =
            self.load_shader_from_internal(name.as_ref(), code.as_ref(), device)?;
        self.shader_dic
            .insert(name.as_ref().to_string(), Arc::new(shader_module));
        self.reflection_dic
            .insert(name.as_ref().to_string(), Arc::new(reflection));
        Ok(())
    }

    fn load_shader_from_internal<K>(
        &mut self,
        name: K,
        code: K,
        device: &wgpu::Device,
    ) -> crate::error::Result<(wgpu::ShaderModule, Reflection)>
    where
        K: AsRef<str>,
    {
        let span = tracy_client::span!();
        span.emit_text(&format!("Load shader: {}", name.as_ref()));
        let shader_module = (|| {
            device.push_error_scope(wgpu::ErrorFilter::Validation);
            let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some(&name.as_ref()),
                source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(&code.as_ref())),
            });
            if let Some(err) = device
                .pop_error_scope()
                .block_on()
                .map(|x| crate::error::Error::Wgpu(Mutex::new(x)))
            {
                Err(err)
            } else {
                Ok(shader_module)
            }
        })()?;
        let reflection = Reflection::new(&code.as_ref(), false)?;
        span.emit_text("done");
        Ok((shader_module, reflection))
    }

    pub fn get_shader(&self, name: &str) -> Arc<wgpu::ShaderModule> {
        Arc::clone(
            self.shader_dic
                .get(name)
                .expect(&format!("{} shader is loaded.", name)),
        )
    }

    pub fn get_shader_reflection(&self, name: &str) -> Arc<Reflection> {
        Arc::clone(
            self.reflection_dic
                .get(name)
                .expect(&format!("{} shader reflection is loaded.", name)),
        )
    }

    pub fn get_material_shader_name(handle: MaterialRenderPipelineHandle) -> String {
        format!("material_{}", handle)
    }

    pub fn get_material_shader(
        &self,
        handle: MaterialRenderPipelineHandle,
    ) -> Arc<wgpu::ShaderModule> {
        let name = Self::get_material_shader_name(handle);
        self.shader_dic
            .get(&name)
            .expect(&format!("{} shader is loaded.", name))
            .clone()
    }

    pub fn get_material_shader_reflection(
        &self,
        handle: MaterialRenderPipelineHandle,
    ) -> Arc<Reflection> {
        let name = Self::get_material_shader_name(handle);
        self.reflection_dic
            .get(&name)
            .expect(&format!("{} shader reflection is loaded.", name))
            .clone()
    }

    pub fn validate_shader_module(module: &naga::Module) -> crate::error::Result<()> {
        let mut validator = naga::valid::Validator::new(
            naga::valid::ValidationFlags::all(),
            naga::valid::Capabilities::all(),
        );
        validator
            .validate(&module)
            .map_err(|err| crate::error::Error::ValidationError(err))?;
        Ok(())
    }

    pub fn validate_shader_code(shader_code: impl AsRef<str>) -> crate::error::Result<()> {
        let module = naga::front::wgsl::parse_str(&shader_code.as_ref())
            .map_err(|err| crate::error::Error::ShaderReflection(err, None))?;
        Self::validate_shader_module(&module)
    }
}
