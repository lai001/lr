use crate::{
    dotnet::{
        application::{RuntimeApplicationFunctions, GLOBAL_RUNTIME_APPLICATION_FUNCTIONS},
        entry_info,
    },
    ffi::{
        file_watch::FileWatch, native_command_encoder::NativeWGPUCommandEncoderFunctions,
        native_device::NativeWGPUDeviceFunctions,
        native_pipeline_layout::NativeWGPUPipelineLayoutFunctions,
        native_queue::NativeWGPUQueueFunctions, native_render_pass::NativeWGPURenderPassFunctions,
        native_render_pipeline::NativeWGPURenderPipelineFunctions,
        native_shader_module::NativeWGPUShaderModuleFunctions,
    },
    project::{ProjectDescription, ProjectDescriptionDotnetField},
};

use super::application::RuntimeApplication;

pub struct DotnetRuntime {
    pub application: RuntimeApplication,
    file_watch: FileWatch,
}

impl DotnetRuntime {
    pub fn new(device: &mut wgpu::Device) -> DotnetRuntime {
        let application: RuntimeApplication;
        let mut file_watch = FileWatch {
            file_changed_func: std::ptr::null_mut(),
        };
        let project_description = ProjectDescription::default();
        let project_description = project_description.lock().unwrap();
        let dotnet_field: &ProjectDescriptionDotnetField = project_description.get_dotnet();

        unsafe {
            type EntryPointFn = unsafe extern "C" fn(entry_info: *mut libc::c_void);
            let mut entry_info = entry_info::EntryInfo {
                runtime_application: std::ptr::null_mut(),
                runtime_application_functions: (&mut GLOBAL_RUNTIME_APPLICATION_FUNCTIONS
                    .lock()
                    .unwrap())
                    as &mut RuntimeApplicationFunctions
                    as *mut _
                    as *mut RuntimeApplicationFunctions,
                native_device_functions: NativeWGPUDeviceFunctions::new(),
                native_device: device as *mut _ as *mut wgpu::Device,
                native_command_encoder_functions: NativeWGPUCommandEncoderFunctions::new(),
                native_render_pass_functions: NativeWGPURenderPassFunctions::new(),
                native_queue_functions: NativeWGPUQueueFunctions::new(),
                runtime_file_watch: &mut file_watch as *mut _ as *mut FileWatch,
                native_shader_module_functions: NativeWGPUShaderModuleFunctions::new(),
                native_render_pipeline_functions: NativeWGPURenderPipelineFunctions::new(),
                native_pipeline_layout_functions: NativeWGPUPipelineLayoutFunctions::new(),
            };

            let entry_point_func: *mut EntryPointFn =
                rs_dotnet::dotnet::load_and_get_entry_point_func(
                    dotnet_field.config_path.to_string(),
                    dotnet_field.assembly_path.to_string(),
                    dotnet_field.type_name.to_string(),
                    dotnet_field.method_name.to_string(),
                );

            let entry_point_func: EntryPointFn = std::mem::transmute(entry_point_func);
            entry_point_func((&mut entry_info) as *mut _ as *mut libc::c_void);
            application = RuntimeApplication::new(entry_info.runtime_application);
        }
        DotnetRuntime {
            application,
            file_watch,
        }
    }

    pub fn reload_script(&mut self) {
        let file_changed_func = self.file_watch.file_changed_func;
        if file_changed_func.is_null() {
            panic!();
        }
        let file_changed_func: fn() = unsafe { std::mem::transmute(file_changed_func) };
        file_changed_func();
    }
}
