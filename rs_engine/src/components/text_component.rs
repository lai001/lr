use crate::{
    build_built_in_resouce_url,
    content::{
        content_file_type::EContentFileType, level::LevelPhysics, render_target_2d::RenderTarget2D,
    },
    drawable::{Drawable, EDrawObjectType},
    engine::Engine,
    scene_node::SceneNode,
};
use egui::ViewportId;
use rs_egui_ext::{ui_begin_windowless, ui_end_windowless};
use rs_foundation::new::{SingleThreadMut, SingleThreadMutType};
use rs_render::{
    command::{RenderUIOptions, TextureDescriptorCreateInfo},
    egui_render::UICanvasType,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use winit::dpi::PhysicalSize;

#[derive(Clone)]
pub struct TextComponentRuntime {
    pub parent_final_transformation: glam::Mat4,
    pub final_transformation: glam::Mat4,
    pub ctx: egui::Context,
    pub egui_input: egui::RawInput,
    pub framebuffer_handle: crate::handle::TextureHandle,
    pub texture_size: PhysicalSize<u32>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct TextComponent {
    pub name: String,
    pub transformation: glam::Mat4,
    pub text: String,
    pub size: glam::UVec2,
    pub text_size: f32,
    #[serde(default)]
    pub text_color: glam::U8Vec4,
    pub render_target: Option<url::Url>,
    #[serde(skip)]
    pub run_time: Option<TextComponentRuntime>,
}

impl TextComponent {
    pub fn new(name: String, transformation: glam::Mat4) -> Self {
        let color = egui::Color32::WHITE;
        Self {
            name,
            transformation,
            run_time: None,
            text: String::new(),
            size: glam::uvec2(256, 256),
            render_target: None,
            text_size: 36.0,
            text_color: glam::U8Vec4::from_slice(&color.to_array()),
        }
    }

    pub fn new_scene_node(
        name: String,
        transformation: glam::Mat4,
    ) -> SingleThreadMutType<SceneNode> {
        let component = Self::new(name, transformation);
        SingleThreadMut::new(SceneNode::from_component(component))
    }

    fn custom_render_target(
        render_target: Option<url::Url>,
        files: &HashMap<url::Url, EContentFileType>,
    ) -> Option<(PhysicalSize<u32>, crate::handle::TextureHandle)> {
        let rt = &render_target?;
        for (file_url, file) in files {
            if file_url != rt {
                continue;
            }
            let file = file.borrow();
            let Some(render_target2d) = file.downcast_ref::<RenderTarget2D>() else {
                continue;
            };
            if render_target2d.format != Self::supported_foramt() {
                continue;
            }
            let texture_handle = render_target2d.texture_handle()?;
            return Some((
                PhysicalSize::new(render_target2d.width, render_target2d.height),
                texture_handle,
            ));
        }
        return None;
    }

    pub fn set_render_target(
        &mut self,
        render_target: Option<url::Url>,
        engine: &mut crate::engine::Engine,
        files: &HashMap<url::Url, EContentFileType>,
    ) {
        self.render_target = render_target;
        let Some(runtime) = &mut self.run_time else {
            return;
        };
        let name = uuid::Uuid::new_v4().to_string();
        let info = Self::texture_descriptor_create_info(self.size);

        let (texture_size, framebuffer_handle) =
            Self::custom_render_target(self.render_target.clone(), files).unwrap_or_else(|| {
                (
                    PhysicalSize::new(self.size.x, self.size.y),
                    engine.create_texture(
                        &build_built_in_resouce_url(name).expect("Valid name"),
                        info,
                    ),
                )
            });
        runtime.texture_size = PhysicalSize::new(texture_size.width, texture_size.height);
        runtime.framebuffer_handle = framebuffer_handle;
    }

    fn texture_descriptor_create_info(size: glam::UVec2) -> TextureDescriptorCreateInfo {
        let info = TextureDescriptorCreateInfo {
            label: None,
            size: wgpu::Extent3d {
                width: size.x,
                height: size.y,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: Self::supported_foramt(),
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: None,
        };
        info
    }

    fn supported_foramt() -> wgpu::TextureFormat {
        wgpu::TextureFormat::Rgba8UnormSrgb
    }
}

#[typetag::serde]
impl super::component::Component for TextComponent {
    fn get_name(&self) -> String {
        self.name.clone()
    }

    fn set_name(&mut self, new_name: String) {
        self.name = new_name;
    }

    fn get_final_transformation(&self) -> glam::Mat4 {
        let Some(run_time) = self.run_time.as_ref() else {
            return glam::Mat4::IDENTITY;
        };
        run_time.final_transformation
    }

    fn set_transformation(&mut self, transformation: glam::Mat4) {
        self.transformation = transformation;
    }

    fn get_transformation(&self) -> glam::Mat4 {
        self.transformation
    }

    fn on_post_update_transformation(
        &mut self,
        engine: &mut Engine,
        level_physics: Option<&mut LevelPhysics>,
        files: &HashMap<url::Url, EContentFileType>,
    ) {
        let _ = files;
        let _ = engine;
        let _ = level_physics;
    }

    fn set_final_transformation(&mut self, final_transformation: glam::Mat4) {
        let Some(run_time) = self.run_time.as_mut() else {
            return;
        };
        run_time.final_transformation = final_transformation;
    }

    fn set_parent_final_transformation(&mut self, parent_final_transformation: glam::Mat4) {
        let Some(run_time) = self.run_time.as_mut() else {
            return;
        };
        run_time.parent_final_transformation = parent_final_transformation;
    }

    fn get_parent_final_transformation(&self) -> glam::Mat4 {
        let Some(run_time) = self.run_time.as_ref() else {
            return glam::Mat4::IDENTITY;
        };
        run_time.parent_final_transformation
    }

    fn initialize(
        &mut self,
        engine: &mut crate::engine::Engine,
        files: &HashMap<url::Url, EContentFileType>,
        player_viewport: &mut crate::player_viewport::PlayerViewport,
    ) {
        let _ = player_viewport;

        let (texture_size, framebuffer_handle) =
            Self::custom_render_target(self.render_target.clone(), files).unwrap_or_else(|| {
                let name = uuid::Uuid::new_v4().to_string();
                let info = Self::texture_descriptor_create_info(self.size);
                (
                    PhysicalSize::new(self.size.x, self.size.y),
                    engine.create_texture(
                        &build_built_in_resouce_url(name).expect("Valid name"),
                        info,
                    ),
                )
            });

        let mut egui_input = egui::RawInput::default();
        egui_input.viewport_id = ViewportId::from_hash_of(format!(
            "[TextComponent]{}_{}",
            self.name,
            uuid::Uuid::new_v4()
        ));
        egui_input
            .viewports
            .entry(egui_input.viewport_id)
            .or_default();
        self.run_time = Some(TextComponentRuntime {
            parent_final_transformation: glam::Mat4::IDENTITY,
            final_transformation: glam::Mat4::IDENTITY,
            ctx: engine.egui_context().clone(),
            egui_input,
            framebuffer_handle,
            texture_size,
        })
    }

    fn initialize_physics(
        &mut self,
        engine: &mut crate::engine::Engine,
        level_physics: &mut LevelPhysics,
        files: &HashMap<url::Url, EContentFileType>,
    ) {
        let _ = files;
        let _ = engine;
        let _ = level_physics;
    }

    fn tick(
        &mut self,
        time: f32,
        engine: &mut crate::engine::Engine,
        level_physics: &mut LevelPhysics,
    ) {
        let _ = level_physics;
        let _ = engine;
        let _ = time;
        let Some(run_time) = &mut self.run_time else {
            return;
        };

        ui_begin_windowless(
            &run_time.ctx,
            &mut run_time.egui_input,
            1.0,
            run_time.texture_size,
        );

        let mut root_ui = egui::Ui::new(
            run_time.ctx.clone(),
            egui::Id::new((run_time.ctx.viewport_id(), "__top_ui")),
            egui::UiBuilder::new()
                .layer_id(egui::LayerId::background())
                .max_rect(run_time.ctx.viewport_rect()),
        );
        let text_color = egui::Color32::from_rgba_premultiplied(
            self.text_color.x,
            self.text_color.y,
            self.text_color.z,
            self.text_color.w,
        );
        let text = egui::RichText::new(self.text.clone())
            .size(self.text_size)
            .color(text_color);
        root_ui.label(text);

        let output = ui_end_windowless(&run_time.ctx, &mut run_time.egui_input);
        let mut render_uioptions = RenderUIOptions::new(
            UICanvasType::FrameBuffer(*run_time.framebuffer_handle),
            output,
        );
        render_uioptions.ops = wgpu::Operations {
            load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
            store: wgpu::StoreOp::Store,
        };
        engine.draw_gui(render_uioptions);
    }

    fn as_drawable(&self) -> Option<&dyn Drawable> {
        Some(self)
    }
}

impl Drawable for TextComponent {
    fn get_draw_objects(&self) -> Vec<&crate::drawable::EDrawObjectType> {
        return vec![];
    }

    fn get_draw_objects_mut(&mut self) -> Vec<&mut EDrawObjectType> {
        return vec![];
    }
}
