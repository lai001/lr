use crate::{gizmo::FGizmo, rotator::Rotator};
use egui::{color_picker::Alpha, Context, Ui, Widget};
use egui_demo_lib::DemoWindows;
use egui_gizmo::GizmoMode;
use egui_wgpu_backend::{RenderPass, ScreenDescriptor};
use egui_winit_platform::{Platform, PlatformDescriptor};
use std::time::Instant;

pub struct EGUIContext {
    screen_descriptor: ScreenDescriptor,
    platform: Platform,
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

    pub fn handle_event(&mut self, event: &winit::event::Event<()>) {
        self.platform.handle_event(event);
    }

    pub fn get_platform_context(&mut self) -> Context {
        self.platform.context()
    }

    pub fn gizmo_settings(&mut self, gizmo: &mut FGizmo) {
        let gizmo_mode = &mut gizmo.gizmo_mode;
        let gizmo_orientation = &mut gizmo.gizmo_orientation;
        let custom_highlight_color = &mut gizmo.custom_highlight_color;

        let stroke_width = &mut gizmo.visuals.stroke_width;
        let gizmo_size = &mut gizmo.visuals.gizmo_size;
        let mut highlight_color = egui::Color32::GOLD;
        let x_color = &mut gizmo.visuals.x_color;
        let y_color = &mut gizmo.visuals.y_color;
        let z_color = &mut gizmo.visuals.z_color;
        let s_color = &mut gizmo.visuals.s_color;
        let inactive_alpha = &mut gizmo.visuals.inactive_alpha;
        let highlight_alpha = &mut gizmo.visuals.highlight_alpha;
        let egui_ctx = &self.platform.context();

        egui::Window::new("Gizmo Settings")
            .resizable(false)
            .show(egui_ctx, |ui| {
                egui::ComboBox::from_label("Mode")
                    .selected_text(format!("{gizmo_mode:?}"))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(gizmo_mode, GizmoMode::Rotate, "Rotate");
                        ui.selectable_value(gizmo_mode, GizmoMode::Translate, "Translate");
                        ui.selectable_value(gizmo_mode, GizmoMode::Scale, "Scale");
                    });
                ui.end_row();

                egui::ComboBox::from_label("Orientation")
                    .selected_text(format!("{gizmo_orientation:?}"))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            gizmo_orientation,
                            egui_gizmo::GizmoOrientation::Global,
                            "Global",
                        );
                        ui.selectable_value(
                            gizmo_orientation,
                            egui_gizmo::GizmoOrientation::Local,
                            "Local",
                        );
                    });
                ui.end_row();

                ui.separator();

                egui::Slider::new(gizmo_size, 10.0f32..=500.0)
                    .text("Gizmo size")
                    .ui(ui);
                egui::Slider::new(stroke_width, 0.1..=10.0)
                    .text("Stroke width")
                    .ui(ui);
                egui::Slider::new(inactive_alpha, 0.0..=1.0)
                    .text("Inactive alpha")
                    .ui(ui);
                egui::Slider::new(highlight_alpha, 0.0..=1.0)
                    .text("Highlighted alpha")
                    .ui(ui);

                ui.horizontal(|ui| {
                    egui::color_picker::color_edit_button_srgba(
                        ui,
                        &mut highlight_color,
                        Alpha::Opaque,
                    );
                    egui::Checkbox::new(custom_highlight_color, "Custom highlight color").ui(ui);
                });

                ui.horizontal(|ui| {
                    egui::color_picker::color_edit_button_srgba(ui, x_color, Alpha::Opaque);
                    egui::Label::new("X axis color").wrap(false).ui(ui);
                });

                ui.horizontal(|ui| {
                    egui::color_picker::color_edit_button_srgba(ui, y_color, Alpha::Opaque);
                    egui::Label::new("Y axis color").wrap(false).ui(ui);
                });
                ui.horizontal(|ui| {
                    egui::color_picker::color_edit_button_srgba(ui, z_color, Alpha::Opaque);
                    egui::Label::new("Z axis color").wrap(false).ui(ui);
                });
                ui.horizontal(|ui| {
                    egui::color_picker::color_edit_button_srgba(ui, s_color, Alpha::Opaque);
                    egui::Label::new("Screen axis color").wrap(false).ui(ui);
                });
                ui.end_row();
            });
        if *custom_highlight_color {
            gizmo.visuals.highlight_color = Some(highlight_color);
        } else {
            gizmo.visuals.highlight_color = None;
        }
    }
}
