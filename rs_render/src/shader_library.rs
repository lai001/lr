use super::reflection::Reflection;
use std::{collections::HashMap, sync::Arc};

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

    pub fn load_shader_from<K>(&mut self, shaders: HashMap<K, String>, device: &wgpu::Device)
    where
        K: ToString,
    {
        let mut shader_dic: HashMap<String, Arc<wgpu::ShaderModule>> = HashMap::new();
        let mut reflection_dic: HashMap<String, Arc<Reflection>> = HashMap::new();

        for (name, code) in shaders {
            let shader = Arc::new(device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some(&name.to_string()),
                source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(&code)),
            }));
            match Reflection::new(&code) {
                Ok(reflection) => {
                    let reflection = Arc::new(reflection);
                    shader_dic.insert(name.to_string(), shader);
                    reflection_dic.insert(name.to_string(), reflection);
                }
                Err(err) => {
                    log::warn!("{err:?}");
                }
            }
        }

        for (k, v) in shader_dic {
            self.shader_dic.insert(k, v);
        }
        for (k, v) in reflection_dic {
            self.reflection_dic.insert(k, v);
        }
    }

    pub fn get_shader(&self, name: &str) -> Arc<wgpu::ShaderModule> {
        Arc::clone(
            self.shader_dic
                .get(name)
                .expect(&format!("{} shader is not exist.", name)),
        )
    }

    pub fn get_shader_reflection(&self, name: &str) -> Arc<Reflection> {
        Arc::clone(
            self.reflection_dic
                .get(name)
                .expect(&format!("{} shader reflection is not exist.", name)),
        )
    }
}
