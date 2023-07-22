use std::{
    collections::HashMap,
    io::Read,
    path::Path,
    sync::{Arc, Mutex},
};
use walkdir::WalkDir;

pub struct ShaderLibrary {
    shader_dic: HashMap<String, Arc<wgpu::ShaderModule>>,
}

lazy_static! {
    static ref GLOBAL_SHADER_LIBRARY: Arc<Mutex<ShaderLibrary>> =
        Arc::new(Mutex::new(ShaderLibrary::new()));
}

impl ShaderLibrary {
    pub fn new() -> ShaderLibrary {
        ShaderLibrary {
            shader_dic: HashMap::new(),
        }
    }

    pub fn default() -> Arc<Mutex<ShaderLibrary>> {
        GLOBAL_SHADER_LIBRARY.clone()
    }

    pub fn load_shader_from(&mut self, device: &wgpu::Device, search_dir: &str) {
        let mut shader_dic: HashMap<String, Arc<wgpu::ShaderModule>> = HashMap::new();
        let mut paths: Vec<String> = vec![];
        for entry in WalkDir::new(search_dir) {
            if let Ok(entry) = entry {
                if let Some(extension) = entry.path().extension() {
                    if extension.to_str() == Some("wgsl") {
                        if let Some(path) = entry.path().to_str() {
                            paths.push(path.to_string());
                        }
                    }
                }
            }
        }
        for path in &paths {
            match std::fs::File::open(path) {
                Ok(mut file) => {
                    let key = Path::new(path)
                        .file_name()
                        .unwrap()
                        .to_str()
                        .unwrap()
                        .to_string();
                    let mut contents = String::new();
                    if let Ok(_) = file.read_to_string(&mut contents) {
                        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                            label: Some(&key),
                            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(&contents)),
                        });
                        shader_dic.insert(key.to_string(), Arc::new(shader));
                        log::trace!("{} shader preload.", path);
                    } else {
                        panic!()
                    }
                }
                Err(error) => {
                    log::warn!("error: {}, load shader {} failed.", error, path);
                }
            }
        }
        for (k, v) in shader_dic {
            self.shader_dic.insert(k, v);
        }
    }

    pub fn get_shader(&self, name: &str) -> Arc<wgpu::ShaderModule> {
        Arc::clone(
            self.shader_dic
                .get(name)
                .expect(&format!("{} shader is not exist.", name)),
        )
    }
}
