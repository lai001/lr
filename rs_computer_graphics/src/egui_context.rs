use egui::Ui;
use egui_demo_lib::DemoWindows;
use egui_wgpu_backend::{RenderPass, ScreenDescriptor};
use egui_winit_platform::{Platform, PlatformDescriptor};
use std::time::Instant;

use crate::rotator::Rotator;

pub struct EGUIContext {
    screen_descriptor: ScreenDescriptor,
    pub platform: Platform,
    egui_rpass: RenderPass,
    demo_app: DemoWindows,
    start_time: Instant,
    render_ticks: usize,
}

pub struct DataSource {
    pub is_captrue_enable: bool,
    pub is_save: bool,

    pub mesh_location: glam::Vec3,
    pub mesh_rotator: Rotator,
}

impl EGUIContext {
    pub fn new(
        device: &wgpu::Device,
        swapchain_format: wgpu::TextureFormat,
        window: &winit::window::Window,
    ) -> EGUIContext {
        let platform_descriptor = PlatformDescriptor {
            physical_width: window.inner_size().width as u32,
            physical_height: window.inner_size().height as u32,
            scale_factor: window.scale_factor(),
            font_definitions: egui::FontDefinitions::default(),
            style: Default::default(),
        };
        let screen_descriptor = ScreenDescriptor {
            physical_width: window.inner_size().width,
            physical_height: window.inner_size().height,
            scale_factor: window.scale_factor() as f32,
        };
        let platform = Platform::new(platform_descriptor);
        let egui_rpass = egui_wgpu_backend::RenderPass::new(&device, swapchain_format, 1);
        let demo_app = egui_demo_lib::DemoWindows::default();
        EGUIContext {
            screen_descriptor,
            platform,
            egui_rpass,
            demo_app,
            start_time: Instant::now(),
            render_ticks: 0,
        }
    }

    pub fn get_render_ticks(&self) -> usize {
        self.render_ticks
    }

    pub fn tick(&mut self) {
        self.render_ticks += 1;
        self.platform
            .update_time(self.start_time.elapsed().as_secs_f64());
    }

    fn main_ui(&mut self, data_source: &mut DataSource) {
        let context = &self.platform.context();

        // self.demo_app.ui(context);

        egui::Area::new("Buttons")
            .fixed_pos(egui::pos2(32.0, 32.0))
            .show(context, |ui| {
                let response = ui.button("Capture screen");
                if response.clicked() {
                    data_source.is_captrue_enable = true;
                }
                let response = ui.button("Save");
                if response.clicked() {
                    data_source.is_save = true;
                }
            });

        egui::Window::new("Property").show(context, |ui| {
            self.draw_vec3_control(ui, &mut data_source.mesh_location, "Location ");
            self.draw_rotator_control(ui, &mut data_source.mesh_rotator, "Rotator ");
        });
        // data_source
    }

    fn draw_rotator_control(&mut self, ui: &mut Ui, value: &mut Rotator, label: &str) {
        ui.horizontal(|ui| {
            ui.label(label);
            ui.add(
                egui::DragValue::new(&mut value.pitch)
                    .speed(1.0)
                    .clamp_range(0.0..=360.0)
                    .prefix("pitch: "),
            );
            ui.add(
                egui::DragValue::new(&mut value.yaw)
                    .speed(1.0)
                    .clamp_range(0.0..=360.0)
                    .prefix("yaw: "),
            );
            ui.add(
                egui::DragValue::new(&mut value.roll)
                    .speed(1.0)
                    .clamp_range(0.0..=360.0)
                    .prefix("roll: "),
            );
        });
    }

    fn draw_vec3_control(&mut self, ui: &mut Ui, value: &mut glam::Vec3, label: &str) {
        ui.horizontal(|ui| {
            ui.label(label);
            ui.add(
                egui::DragValue::new(&mut value.x)
                    .speed(0.01)
                    .clamp_range(-100.0..=100.0)
                    .prefix("x: "),
            );
            ui.add(
                egui::DragValue::new(&mut value.y)
                    .speed(0.01)
                    .clamp_range(-100.0..=100.0)
                    .prefix("y: "),
            );
            ui.add(
                egui::DragValue::new(&mut value.z)
                    .speed(0.01)
                    .clamp_range(-100.0..=100.0)
                    .prefix("z: "),
            );
        });
    }

    pub fn draw_ui(
        &mut self,
        queue: &wgpu::Queue,
        device: &wgpu::Device,
        output_view: &wgpu::TextureView,
        data_source: &mut DataSource,
    ) /*-> DataSource*/
    {
        // let data_source: DataSource;
        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            self.platform.begin_frame();

            /*data_source = */
            self.main_ui(data_source);

            let full_output = self.platform.end_frame(None);
            let paint_jobs = self.platform.context().tessellate(full_output.shapes);

            let tdelta: egui::TexturesDelta = full_output.textures_delta;
            self.egui_rpass
                .add_textures(&device, &queue, &tdelta)
                .unwrap();
            self.egui_rpass
                .update_buffers(&device, &queue, &paint_jobs, &self.screen_descriptor);

            self.egui_rpass
                .execute(
                    &mut encoder,
                    &output_view,
                    &paint_jobs,
                    &self.screen_descriptor,
                    None,
                )
                .unwrap();
            self.egui_rpass.remove_textures(tdelta).unwrap();
        }
        queue.submit(std::iter::once(encoder.finish()));
        // data_source
    }
}
