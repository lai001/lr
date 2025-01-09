pub mod compile_command;
pub mod error;
pub mod pre_process;

#[cfg(test)]
mod test {
    use crate::pre_process::pre_process;
    use rs_core_minimal::path_ext::CanonicalizeSlashExt;
    use wgpu::*;

    struct TestGPUContext {
        device: Device,
    }

    impl TestGPUContext {
        pub fn new() -> TestGPUContext {
            let instance = wgpu::Instance::default();

            let adapter = pollster::block_on(
                instance.request_adapter(&wgpu::RequestAdapterOptions::default()),
            )
            .unwrap();

            let (device, _) = pollster::block_on(
                adapter.request_device(&wgpu::DeviceDescriptor::default(), None),
            )
            .unwrap();
            TestGPUContext { device }
        }
    }

    #[test]
    fn test_case() {
        let render_crate_dir = rs_core_minimal::file_manager::get_engine_root_dir()
            .join("rs_render")
            .canonicalize_slash()
            .unwrap();
        let self_crate_dir = rs_core_minimal::file_manager::get_engine_root_dir()
            .join("rs_shader_compiler")
            .canonicalize_slash()
            .unwrap();
        let shader_path = render_crate_dir
            .join("shaders/pbr_shading.wgsl")
            .canonicalize_slash()
            .unwrap();
        let include_dirs = vec![render_crate_dir
            .join("shaders")
            .canonicalize_slash()
            .unwrap()];

        let mut definitions = vec![
            "VIRTUAL_TEXTURE",
            "MAX_CASCADES_PER_LIGHT=1",
            "MAX_DIRECTIONAL_LIGHTS=1u",
            "VIRTUAL_TEXTURE_CONSTANTS_BINDING=3",
            "STANDARD_MATERIAL_CLEARCOAT",
            "SKELETON_MAX_BONES",
        ];
        macro_rules! group_binding {
            ($name:literal, $g:expr, $b:expr) => {
                definitions.append(&mut vec![
                    concat!($name, "_GROUP=", $g),
                    concat!($name, "_BINDING=", $b),
                ]);
            };
        }
        group_binding!("GLOBAL_CONSTANTS", 0, 0);
        group_binding!("BASE_COLOR_SAMPLER", 0, 1);
        group_binding!("PHYSICAL_TEXTURE", 0, 2);
        group_binding!("PAGE_TABLE_TEXTURE", 0, 3);
        group_binding!("BRDFLUT_TEXTURE", 0, 4);
        group_binding!("PRE_FILTER_CUBE_MAP_TEXTURE", 0, 5);
        group_binding!("IRRADIANCE_TEXTURE", 0, 6);
        group_binding!("SHADOW_MAP", 0, 7);
        group_binding!("CONSTANTS", 0, 8);
        group_binding!("CLUSTERABLE_OBJECTS", 0, 9);
        group_binding!("LIGHTS", 0, 10);
        group_binding!("SKIN_CONSTANTS", 0, 11);
        group_binding!("VIRTUAL_TEXTURE_CONSTANTS", 0, 12);

        let shader_code = pre_process(
            &shader_path,
            include_dirs.into_iter(),
            definitions.into_iter(),
        )
        .unwrap();
        let module = naga::front::wgsl::parse_str(&shader_code).unwrap();
        validate_shader_module(&module);
        std::fs::write(self_crate_dir.join("target/pbr_shading.wgsl"), &shader_code).unwrap();
        let gpu_context = TestGPUContext::new();
        gpu_context
            .device
            .create_shader_module(ShaderModuleDescriptor {
                label: None,
                source: ShaderSource::Wgsl(shader_code.into()),
            });
    }

    pub fn validate_shader_module(module: &naga::Module) {
        let mut validator = naga::valid::Validator::new(
            naga::valid::ValidationFlags::all(),
            naga::valid::Capabilities::all(),
        );
        validator.validate(&module).unwrap();
    }
}
