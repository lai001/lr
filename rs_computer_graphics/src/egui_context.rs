use crate::{camera::Camera, gizmo::FGizmo, rotator::Rotator};
use egui::{color_picker::Alpha, Context, TextureId, Ui, Vec2, Widget};
use egui_demo_lib::DemoWindows;
use egui_gizmo::GizmoMode;
use egui_wgpu_backend::{RenderPass, ScreenDescriptor};
use egui_winit_platform::{Platform, PlatformDescriptor};
use std::time::Instant;

pub struct EGUIContext {
    platform: Platform,
    demo_app: DemoWindows,
    start_time: Instant,
    current_frame_start_time: Instant,
    render_ticks: usize,
    fps: u64,
    current_fps: f32,
}

pub struct EGUIContextRenderer {
    context: Context,
    egui_render_pass: RenderPass,
}

impl EGUIContextRenderer {
    pub fn new(
        context: Context,
        device: &wgpu::Device,
        output_format: wgpu::TextureFormat,
        msaa_samples: u32,
    ) -> EGUIContextRenderer {
        let egui_render_pass =
            egui_wgpu_backend::RenderPass::new(&device, output_format, msaa_samples);
        EGUIContextRenderer {
            context,
            egui_render_pass,
        }
    }

    pub fn create_image(
        &mut self,
        device: &wgpu::Device,
        texture_view: &wgpu::TextureView,
        size: Vec2,
    ) -> DrawImage {
        DrawImage {
            texture_id: self.egui_render_pass.egui_texture_from_wgpu_texture(
                device,
                texture_view,
                wgpu::FilterMode::Linear,
            ),
            size,
        }
    }

    pub fn create_image2(
        &mut self,
        device: &wgpu::Device,
        texture_view: &wgpu::TextureView,
        texture_filter: Option<wgpu::FilterMode>,
    ) -> TextureId {
        self.egui_render_pass.egui_texture_from_wgpu_texture(
            device,
            texture_view,
            texture_filter.unwrap_or(wgpu::FilterMode::Linear),
        )
    }

    pub fn render(
        &mut self,
        full_output: &egui::FullOutput,
        queue: &wgpu::Queue,
        device: &wgpu::Device,
        screen_descriptor: &ScreenDescriptor,
        output_view: &wgpu::TextureView,
    ) {
        let paint_jobs = self.context.tessellate(full_output.shapes.clone());
        let textures_delta: egui::TexturesDelta = full_output.textures_delta.clone();
        if let Err(error) = self
            .egui_render_pass
            .add_textures(&device, &queue, &textures_delta)
        {
            log::warn!("{error}");
            return;
        }
        self.egui_render_pass
            .update_buffers(&device, &queue, &paint_jobs, &screen_descriptor);

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("EGUIContextRenderer.CommandEncoder"),
        });
        if let Err(error) = self.egui_render_pass.execute(
            &mut encoder,
            &output_view,
            &paint_jobs,
            &screen_descriptor,
            None,
        ) {
            log::warn!("{error}");
            return;
        }
        queue.submit(std::iter::once(encoder.finish()));
    }

    pub fn remove_texture_ids(&mut self, texture_ids: &[egui::TextureId]) {
        let textures: egui::TexturesDelta = egui::TexturesDelta {
            set: vec![],
            free: texture_ids.to_vec(),
        };
        match self.egui_render_pass.remove_textures(textures.clone()) {
            Ok(_) => {}
            Err(error) => log::warn!("{error}"),
        }
    }
}

pub struct DataSource {
    pub is_captrue_enable: bool,
    pub is_save_frame_buffer: bool,
    pub frame_buffer_color: egui::Color32,
    pub target_fps: u64,
    pub roughness_factor: f32,
    pub metalness_factor: f32,
    pub draw_image: Option<DrawImage>,
    pub video_frame: Option<DrawImage>,
    pub movement_speed: f32,
    pub motion_speed: f32,
    pub player_time: f32,
    pub seek_time: f32,
    pub is_seek: bool,
    pub mipmap_level: u32,
    pub is_show_pannel: bool,
    pub is_show_texture: bool,
    pub is_show_gizmo_settings: bool,
    pub is_show_property: bool,
    pub gizmo: FGizmo,
    pub camera: crate::camera::Camera,
    pub model_matrix: Option<glam::Mat4>,
}

impl DataSource {
    pub fn new(camera: Camera) -> DataSource {
        Self {
            is_captrue_enable: false,
            is_save_frame_buffer: false,
            frame_buffer_color: egui::Color32::BLACK,
            target_fps: 60,
            roughness_factor: 0.0,
            metalness_factor: 1.0,
            draw_image: None,
            movement_speed: 0.01,
            motion_speed: 0.1,
            player_time: 0.0,
            seek_time: 0.0,
            is_seek: false,
            mipmap_level: 0,
            is_show_pannel: false,
            is_show_texture: true,
            is_show_gizmo_settings: false,
            is_show_property: false,
            gizmo: FGizmo::default(),
            camera,
            model_matrix: None,
            video_frame: None,
        }
    }
}

pub struct DrawImage {
    pub texture_id: TextureId,
    pub size: Vec2,
}

unsafe impl Send for EGUIContext {}

impl EGUIContext {
    pub fn new(screen_descriptor: &ScreenDescriptor) -> EGUIContext {
        let platform_descriptor = PlatformDescriptor {
            physical_width: screen_descriptor.physical_width,
            physical_height: screen_descriptor.physical_height,
            scale_factor: screen_descriptor.scale_factor as f64,
            font_definitions: egui::FontDefinitions::default(),
            style: Default::default(),
        };
        let platform = Platform::new(platform_descriptor);
        let demo_app = egui_demo_lib::DemoWindows::default();
        EGUIContext {
            platform,
            demo_app,
            start_time: Instant::now(),
            render_ticks: 0,
            current_frame_start_time: Instant::now(),
            fps: 60,
            current_fps: 0.0,
        }
    }

    pub fn get_render_ticks(&self) -> usize {
        self.render_ticks
    }

    pub fn tick(&mut self) {
        let now = Instant::now();
        let duration = now - self.current_frame_start_time;
        self.current_fps = 1.0 / duration.as_secs_f32();
        self.current_frame_start_time = now;
        self.render_ticks += 1;
        self.platform
            .update_time(self.start_time.elapsed().as_secs_f64());
    }

    fn main_ui(context: &Context, data_source: &mut DataSource) {
        egui::TopBottomPanel::top("menu_bar").show(context, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("Window", |ui| {
                    ui.set_min_width(220.0);
                    ui.style_mut().wrap = Some(false);
                    if ui.add(egui::Button::new("Pannel")).clicked() {
                        data_source.is_show_pannel = !data_source.is_show_pannel;
                        ui.close_menu();
                    }
                    if ui.add(egui::Button::new("Texture")).clicked() {
                        data_source.is_show_texture = !data_source.is_show_texture;
                        ui.close_menu();
                    }
                    if ui.add(egui::Button::new("Gizmo Settings")).clicked() {
                        data_source.is_show_gizmo_settings = !data_source.is_show_gizmo_settings;
                        ui.close_menu();
                    }
                    if ui.add(egui::Button::new("Property")).clicked() {
                        data_source.is_show_property = !data_source.is_show_property;
                        ui.close_menu();
                    }
                });
            });
        });

        if data_source.is_show_pannel {
            egui::Window::new("Pannel").show(context, |ui| {
                // ui.label(format!("fps: {:.2}", data_source.target_fps));
                let response = ui.button("Capture screen");
                if response.clicked() {
                    data_source.is_captrue_enable = true;
                }
                if ui.button("Save frame buffer").clicked() {
                    data_source.is_save_frame_buffer = true;
                }
                ui.label(format!("Player time: {:.2}", data_source.player_time));
                data_source.is_seek = ui
                    .add(
                        egui::DragValue::new(&mut data_source.seek_time)
                            .speed(0.1)
                            .clamp_range(0.0..=60.0 * 5.0),
                    )
                    .changed();
                egui::color_picker::color_edit_button_srgba(
                    ui,
                    &mut data_source.frame_buffer_color,
                    Alpha::Opaque,
                );
                ui.horizontal(|ui| {
                    ui.label("fps: ");
                    ui.add(egui::DragValue::new(&mut data_source.target_fps).clamp_range(1..=300));
                });
            });
        }

        if data_source.is_show_texture {
            egui::Window::new("Physical Texture")
                // .vscroll(false)
                // .resizable(true)
                // .default_size([250.0, 150.0])
                .show(context, |ui| {
                    if let Some(draw_image) = &data_source.draw_image {
                        ui.image(draw_image.texture_id, draw_image.size);
                        // ui.allocate_space(ui.available_size());
                    }
                });

            egui::Window::new("Video").show(context, |ui| {
                if let Some(video_frame) = &data_source.video_frame {
                    ui.image(video_frame.texture_id, video_frame.size);
                }
            });
        }

        if data_source.is_show_property {
            egui::Window::new("Property").show(context, |ui| {
                ui.add(
                    egui::DragValue::new(&mut data_source.roughness_factor)
                        .speed(0.01)
                        .clamp_range(0.0..=1.0)
                        .prefix("roughness_factor: "),
                );
                ui.add(
                    egui::DragValue::new(&mut data_source.metalness_factor)
                        .speed(0.01)
                        .clamp_range(0.0..=1.0)
                        .prefix("metalness_factor: "),
                );
                ui.add(
                    egui::DragValue::new(&mut data_source.motion_speed)
                        .speed(0.01)
                        .clamp_range(0.0..=1.0)
                        .prefix("motion_speed: "),
                );
                ui.add(
                    egui::DragValue::new(&mut data_source.movement_speed)
                        .speed(0.01)
                        .clamp_range(0.0..=1.0)
                        .prefix("movement_speed: "),
                );
            });
        }

        if data_source.is_show_gizmo_settings {
            Self::gizmo_settings(context, data_source);
        }

        if let Some(model_matrix) = data_source.model_matrix {
            egui::Area::new("Gizmo Viewport")
                .fixed_pos((0.0, 0.0))
                .show(context, |ui| {
                    ui.with_layer_id(egui::LayerId::background(), |ui| {
                        if let Some(model_matrix) =
                            data_source
                                .gizmo
                                .interact(&data_source.camera, ui, &model_matrix)
                        {
                            data_source.model_matrix = Some(model_matrix);
                        }
                    });
                });
        }
    }

    fn draw_rotator_control(&mut self, ui: &mut Ui, value: &mut Rotator, label: &str) {
        ui.horizontal(|ui| {
            ui.label(label);
            ui.add(
                egui::DragValue::new(&mut value.pitch)
                    .speed(1.0)
                    .clamp_range(-180.0..=180.0)
                    .prefix("pitch: "),
            );
            ui.add(
                egui::DragValue::new(&mut value.yaw)
                    .speed(1.0)
                    .clamp_range(-180.0..=180.0)
                    .prefix("yaw: "),
            );
            ui.add(
                egui::DragValue::new(&mut value.roll)
                    .speed(1.0)
                    .clamp_range(-180.0..=180.0)
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

    pub fn layout(&mut self, data_source: &mut DataSource) -> egui::FullOutput {
        self.platform.begin_frame();
        let context = self.get_platform_context();
        // self.demo_app.ui(&context);
        Self::main_ui(&context, data_source);
        self.platform.end_frame(None)
    }

    pub fn handle_event(&mut self, event: &winit::event::Event<()>) {
        self.platform.handle_event(event);
    }

    pub fn get_platform_context(&self) -> Context {
        self.platform.context()
    }

    pub fn gizmo_settings(context: &Context, data_source: &mut DataSource) {
        let gizmo = &mut data_source.gizmo;
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
        let egui_ctx = context;

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

    pub fn get_start_time(&self) -> Instant {
        self.start_time
    }

    pub fn sync_fps(&mut self, control_flow: &mut winit::event_loop::ControlFlow) {
        let start_time = self.current_frame_start_time;
        let elapsed_time = std::time::Instant::now()
            .duration_since(start_time)
            .as_millis() as u64;
        let wait_millis = match 1000 / self.fps >= elapsed_time {
            true => 1000 / self.fps - elapsed_time,
            false => 0,
        };
        let new_inst = start_time + std::time::Duration::from_millis(wait_millis);
        *control_flow = winit::event_loop::ControlFlow::WaitUntil(new_inst);
    }

    pub fn get_fps(&self) -> u64 {
        self.fps
    }

    pub fn set_fps(&mut self, fps: u64) {
        self.fps = fps;
    }
}
