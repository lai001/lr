use super::reflection::Reflection;
use crate::command::MaterialRenderPipelineHandle;
use pollster::FutureExt;
use rs_core_minimal::thread_pool::ThreadPool;
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
        struct TaskResult {
            shader_source: wgpu::ShaderSource<'static>,
            reflection: Reflection,
        }

        let mut results: HashMap<String, crate::error::Result<()>> = HashMap::new();

        let (sender, receiver) = std::sync::mpsc::channel();

        let mut is_finish = shaders.len();
        for (name, code) in shaders {
            ThreadPool::global().spawn({
                let name = name.as_ref().to_string();
                let code = code.clone();
                let sender = sender.clone();
                move || {
                    let span = tracy_client::span!("shader_source_naga");
                    let result = Self::shader_source_naga(code);
                    span.emit_text(&format!("{}", name));

                    match result {
                        Ok(result) => {
                            let _ = sender.send((
                                name,
                                Ok(TaskResult {
                                    shader_source: result.0,
                                    reflection: result.1,
                                }),
                            ));
                        }
                        Err(err) => {
                            let _ = sender.send((name, Err(err)));
                        }
                    }
                }
            });
        }

        while let Ok(result) = receiver.recv() {
            let name = result.0;
            match result.1 {
                Ok(result) => {
                    let result = self.load_shader_from_source_reflection(
                        &name,
                        device,
                        result.shader_source,
                        result.reflection,
                    );
                    results.insert(name, result);
                }
                Err(err) => {
                    results.insert(name, Err(err));
                }
            }
            is_finish -= 1;
            if is_finish == 0 {
                break;
            }
        }

        results
    }

    pub fn load_shader_from_source_reflection<K>(
        &mut self,
        name: K,
        device: &wgpu::Device,
        shader_source: wgpu::ShaderSource<'static>,
        reflection: Reflection,
    ) -> crate::error::Result<()>
    where
        K: AsRef<str>,
    {
        let shader_module = self.create_shader_module(name.as_ref(), device, shader_source)?;
        self.shader_dic
            .insert(name.as_ref().to_string(), Arc::new(shader_module));
        self.reflection_dic
            .insert(name.as_ref().to_string(), Arc::new(reflection));
        Ok(())
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
        let (shader_source, reflection) = Self::shader_source_wgsl(code.as_ref())?;
        let shader_module = self.create_shader_module(name.as_ref(), device, shader_source)?;
        self.shader_dic
            .insert(name.as_ref().to_string(), Arc::new(shader_module));
        self.reflection_dic
            .insert(name.as_ref().to_string(), Arc::new(reflection));
        Ok(())
    }

    fn create_shader_module<K>(
        &mut self,
        name: K,
        device: &wgpu::Device,
        shader_source: wgpu::ShaderSource<'static>,
    ) -> crate::error::Result<wgpu::ShaderModule>
    where
        K: AsRef<str>,
    {
        let span = tracy_client::span!();
        span.emit_text(name.as_ref());
        let shader_module = (|| {
            device.push_error_scope(wgpu::ErrorFilter::Validation);
            let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some(&name.as_ref()),
                source: shader_source,
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
        Ok(shader_module)
    }

    fn shader_source_naga<K>(
        code: K,
    ) -> crate::error::Result<(wgpu::ShaderSource<'static>, Reflection)>
    where
        K: AsRef<str>,
    {
        let module = naga::front::wgsl::parse_str(code.as_ref())
            .map_err(|err| crate::error::Error::ShaderReflection(err, None))?;
        let shader_source = wgpu::ShaderSource::Naga(std::borrow::Cow::Owned(module.clone()));
        let reflection = Reflection::from_naga_module(module, false)?;
        Ok((shader_source, reflection))
    }

    fn shader_source_wgsl<K>(
        code: K,
    ) -> crate::error::Result<(wgpu::ShaderSource<'static>, Reflection)>
    where
        K: AsRef<str>,
    {
        let shader_source =
            wgpu::ShaderSource::Wgsl(std::borrow::Cow::Owned(code.as_ref().to_string()));
        let reflection = Reflection::new(&code.as_ref(), false)?;
        Ok((shader_source, reflection))
    }

    fn _shader_source_spir_v<K>(
        code: K,
    ) -> crate::error::Result<(wgpu::ShaderSource<'static>, Reflection)>
    where
        K: AsRef<str>,
    {
        let reflection = Reflection::new(&code.as_ref(), false)?;
        let module = reflection.get_module();
        let pipeline_options: Option<naga::back::spv::PipelineOptions> =
            match reflection.get_pipeline_type() {
                crate::reflection::EPipelineType::Render(..) => None,
                crate::reflection::EPipelineType::Compute(entry_point) => {
                    Some(naga::back::spv::PipelineOptions {
                        shader_stage: naga::ShaderStage::Compute,
                        entry_point: entry_point.name.clone(),
                    })
                }
            };
        let mut validator = naga::valid::Validator::new(
            naga::valid::ValidationFlags::all(),
            naga::valid::Capabilities::all(),
        );
        let module_info = validator
            .validate(module)
            .map_err(|err| crate::error::Error::ValidationError(err))?;
        let mut options = naga::back::spv::Options::default();
        options
            .flags
            .remove(naga::back::spv::WriterFlags::ADJUST_COORDINATE_SPACE);
        let spv =
            naga::back::spv::write_vec(module, &module_info, &options, pipeline_options.as_ref())
                .map_err(|err| crate::error::Error::NagaBackSpirVError(err))?;
        let shader_source = wgpu::ShaderSource::SpirV(spv.into());
        Ok((shader_source, reflection))
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
