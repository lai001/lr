use super::misc::{gui_render_output, update_window_with_input_mode};
use crate::{editor_context::EWindowType, windows_manager::WindowsManager};
use anyhow::anyhow;
use egui::{load::SizedTexture, TextureId};
use egui_winit::State;
use image::GenericImage;
use rs_audio::{audio_engine::AudioEngine, audio_player_node::AudioPlayerNode};
use rs_engine::{
    build_built_in_resouce_url,
    engine::Engine,
    frame_sync::{EOptions, FrameSync},
    input_mode::EInputMode,
    resource_manager::ResourceManager,
};
use rs_foundation::new::{MultipleThreadMut, MultipleThreadMutType};
use rs_media::{
    composition::{check_composition, CompositionInfo},
    video_frame_player::VideoFramePlayer,
};
use rs_render::{
    buffer_dimensions::BufferDimensions,
    command::{
        CreateTexture, CreateUITexture, InitTextureData, RenderCommand::*,
        TextureDescriptorCreateInfo, UpdateTexture,
    },
    texture_readback::get_bytes_per_pixel,
};
use std::{collections::HashMap, iter::zip, path::Path};
use wgpu::{Extent3d, ImageDataLayout};
use winit::event::WindowEvent;

struct MediaViewDrawObject {
    texture_handle: rs_engine::handle::TextureHandle,
    gui_texture_handle: rs_engine::handle::EGUITextureHandle,
}

pub struct MediaUIWindow {
    pub egui_winit_state: State,
    draw_objects: HashMap<glam::UVec2, MediaViewDrawObject>,
    frame_sync: FrameSync,
    video_frame_player: Option<VideoFramePlayer>,
    cache_sized_texture: Option<SizedTexture>,
    composition_info: Option<CompositionInfo>,
    audio_engine: AudioEngine,
    audio_player_node: Option<MultipleThreadMutType<AudioPlayerNode>>,
}

impl MediaUIWindow {
    pub fn new(
        context: egui::Context,
        window_manager: &mut WindowsManager,
        event_loop_window_target: &winit::event_loop::ActiveEventLoop,
        engine: &mut Engine,
    ) -> anyhow::Result<MediaUIWindow> {
        let window_context =
            window_manager.spwan_new_window(EWindowType::Media, event_loop_window_target)?;
        let window = &*window_context.window.borrow();

        engine
            .set_new_window(
                window_context.get_id(),
                window,
                window_context.get_width(),
                window_context.get_height(),
                window.scale_factor() as f32,
            )
            .map_err(|err| anyhow!("{err}"))?;
        let viewport_id = egui::ViewportId::from_hash_of(window_context.get_id());

        let mut egui_winit_state = egui_winit::State::new(
            context,
            viewport_id,
            window,
            Some(window.scale_factor() as f32),
            None,
            None,
        );

        egui_winit_state.egui_input_mut().viewport_id = viewport_id;
        egui_winit_state.egui_input_mut().viewports =
            std::iter::once((viewport_id, Default::default())).collect();

        let frame_sync = FrameSync::new(EOptions::FPS(60.0));

        let input_mode = EInputMode::UI;
        update_window_with_input_mode(window, input_mode);

        let audio_engine = AudioEngine::new();
        let audio_player_node = None;

        Ok(MediaUIWindow {
            egui_winit_state,
            draw_objects: HashMap::new(),
            frame_sync,
            video_frame_player: None,
            cache_sized_texture: None,
            composition_info: None,
            audio_engine,
            audio_player_node,
        })
    }

    pub fn window_event_process(
        &mut self,
        window_id: isize,
        window: &mut winit::window::Window,
        event: &WindowEvent,
        event_loop_window_target: &winit::event_loop::ActiveEventLoop,
        engine: &mut Engine,
        window_manager: &mut WindowsManager,
    ) -> bool {
        let _ = event_loop_window_target;
        let _ = self.egui_winit_state.on_window_event(window, event);
        let mut is_close = false;
        match event {
            WindowEvent::Resized(size) => {
                engine.resize(window_id, size.width, size.height);
            }
            WindowEvent::CloseRequested => {
                window_manager.remove_window(EWindowType::Media);
                engine.remove_window(window_id);
                is_close = true;
            }
            WindowEvent::RedrawRequested => {
                self.frame_sync.sync(60.0);

                engine.window_redraw_requested_begin(window_id);

                let gui_render_output =
                    gui_render_output(window_id, window, &mut self.egui_winit_state, |state| {
                        egui::Area::new("FrameBackground".into())
                            .interactable(false)
                            .show(state.egui_ctx(), |ui| {
                                ui.with_layer_id(egui::LayerId::background(), |ui| {
                                    MediaUIWindow::player_tick(
                                        engine,
                                        self.video_frame_player.as_mut(),
                                        &mut self.draw_objects,
                                        &mut self.cache_sized_texture,
                                        self.composition_info.clone(),
                                    );
                                    match self.cache_sized_texture {
                                        Some(cache_sized_texture) => {
                                            ui.image(cache_sized_texture);
                                        }
                                        None => {}
                                    }
                                });
                            });
                        egui::Window::new("Control").default_width(1000.0).show(
                            state.egui_ctx(),
                            |ui| {
                                if let Some(player) = self.video_frame_player.as_mut() {
                                    let duration = player.get_duration();
                                    let mut play_time = player.get_current_play_time();
                                    ui.horizontal(|ui| {
                                        ui.spacing_mut().slider_width = 1000.0;
                                        ui.set_min_width(150.0);
                                        if ui
                                            .add(egui::Slider::new(&mut play_time, 0.0..=duration))
                                            .changed()
                                        {
                                            if let Some(audio_player_node) =
                                                self.audio_player_node.as_ref()
                                            {
                                                let mut audio_player_node =
                                                    audio_player_node.lock().unwrap();
                                                audio_player_node.seek(play_time);
                                            }
                                            player.seek(play_time);
                                            player.start();
                                        }
                                    });
                                }
                            },
                        );
                    });
                engine.draw_gui(gui_render_output);

                engine.window_redraw_requested_end(window_id);

                window.request_redraw();
            }
            _ => {}
        }
        is_close
    }

    pub fn update(&mut self, file_path: impl AsRef<Path>) -> anyhow::Result<()> {
        self.composition_info = check_composition(file_path.as_ref())?;
        let path = file_path.as_ref().to_str().ok_or(anyhow!("to_str"))?;
        let mut video_frame_player = VideoFramePlayer::new(path);
        video_frame_player.start();
        self.video_frame_player = Some(video_frame_player);

        let audio_player_node = MultipleThreadMut::new(AudioPlayerNode::from_path(path, false));
        self.audio_player_node = Some(audio_player_node.clone());
        audio_player_node.lock().unwrap().start();
        self.audio_engine.connect(audio_player_node);

        Ok(())
    }

    fn create_media_view_draw_object(
        width: u32,
        height: u32,
        engine: &mut Engine,
    ) -> MediaViewDrawObject {
        let rm = ResourceManager::default();
        let next_ui_texture =
            rm.next_ui_texture(build_built_in_resouce_url("MediaTexture").unwrap());
        let next_texture = rm.next_texture(build_built_in_resouce_url("MediaTexture").unwrap());

        engine.send_render_command(CreateTexture(CreateTexture {
            handle: *next_texture,
            texture_descriptor_create_info: TextureDescriptorCreateInfo::d2(
                Some(format!("MediaTexture")),
                width,
                height,
                None,
            ),
            init_data: None,
        }));

        engine.send_render_command(CreateUITexture(CreateUITexture {
            handle: *next_ui_texture,
            referencing_texture_handle: *next_texture,
        }));

        let object = MediaViewDrawObject {
            texture_handle: next_texture,
            gui_texture_handle: next_ui_texture,
        };
        object
    }

    fn player_tick(
        engine: &mut Engine,
        mut video_frame_player: Option<&mut VideoFramePlayer>,
        draw_objects: &mut HashMap<glam::UVec2, MediaViewDrawObject>,
        cache_sized_texture: &mut Option<SizedTexture>,
        composition_info: Option<CompositionInfo>,
    ) {
        let Some(player) = video_frame_player.as_mut() else {
            return;
        };
        player.tick();
        let Some(frame) = player.get_current_frame() else {
            return;
        };

        let image = if let Some(composition_info) = composition_info {
            let rgb_rect = composition_info.rgb_rect.as_uvec4();
            let alpha_rect = composition_info.alpha_rect.as_uvec4();
            let mut rgb_image = frame
                .image
                .clone()
                .sub_image(rgb_rect.x, rgb_rect.y, rgb_rect.z, rgb_rect.w)
                .to_image();
            let mut alpha_image = frame
                .image
                .clone()
                .sub_image(alpha_rect.x, alpha_rect.y, alpha_rect.z, alpha_rect.w)
                .to_image();

            for (rgb, alpha) in zip(rgb_image.pixels_mut(), alpha_image.pixels_mut()) {
                rgb.0[3] = alpha.0[0];
            }
            rgb_image
        } else {
            frame.image.clone()
        };

        let width = image.width();
        let height = image.height();
        let image_data = image.as_raw();
        let size = glam::uvec2(width, height);
        let buffer_dimensions = BufferDimensions::new(
            width as usize,
            height as usize,
            get_bytes_per_pixel(wgpu::TextureFormat::Rgba8Unorm).unwrap() as usize,
        );

        let media_obejct = draw_objects
            .entry(size)
            .or_insert_with(|| Self::create_media_view_draw_object(width, height, engine));

        engine.send_render_command(UpdateTexture(UpdateTexture {
            handle: *media_obejct.texture_handle,
            texture_data: InitTextureData {
                data: image_data.to_vec(),
                data_layout: ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(buffer_dimensions.padded_bytes_per_row as u32),
                    rows_per_image: None,
                },
            },
            size: Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        }));

        let sized_texture = SizedTexture {
            id: TextureId::User(*media_obejct.gui_texture_handle),
            size: egui::vec2(width as f32, height as f32),
        };
        *cache_sized_texture = Some(sized_texture);
    }
}
