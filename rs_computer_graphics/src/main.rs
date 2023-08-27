#[cfg(feature = "rs_dotnet")]
use rs_computer_graphics::dotnet_runtime::DotnetRuntime;
#[cfg(feature = "rs_quickjs")]
use rs_computer_graphics::quickjs::quickjs_runtime::QuickJSRuntimeContext;
use rs_computer_graphics::{
    acceleration_bake::AccelerationBaker,
    actor::Actor,
    actor_selector::ActorSelector,
    bake_info::BakeInfo,
    camera::{Camera, CameraInputEventHandle, DefaultCameraInputEventHandle},
    default_textures::DefaultTextures,
    demo::capture_screen::CaptureScreen,
    egui_context::{self, EGUIContext},
    file_manager::FileManager,
    frame_buffer::FrameBuffer,
    gizmo::FGizmo,
    material_type::EMaterialType,
    native_window::NativeWindow,
    pbr_material::PBRMaterial,
    render_pipeline::{
        attachment_pipeline::AttachmentPipeline, audio_pipeline::AudioPipeline,
        pbr_pipeline::PBRPipeline, phong_pipeline::PhongPipeline, sky_box_pipeline::SkyBoxPipeline,
        virtual_texture_mesh_pipeline::VirtualTextureMeshPipeline,
    },
    shader::shader_library::ShaderLibrary,
    static_mesh::StaticMesh,
    thread_pool::ThreadPool,
    user_script_change_monitor::UserScriptChangeMonitor,
    util::{change_working_directory, init_log},
    virtual_texture::{
        block_image::BlockImage,
        packing::{ArrayTile, Packing},
        tile_index::{TileIndex, TileOffset},
        virtual_texture_async_loader::VirtualTextureAsyncLoader,
        virtual_texture_configuration::VirtualTextureConfiguration,
        virtual_texture_system::VirtualTextureSystem,
    },
    wgpu_context::WGPUContext,
};
use rs_media::{
    audio_format::EAudioSampleType, audio_frame_extractor::AudioFrameExtractor,
    audio_player_item::AudioPlayerItem, video_frame_player::VideoFramePlayer,
};
use rustfft::{num_complex::Complex, FftPlanner};
use std::{
    borrow::Borrow,
    collections::{HashMap, HashSet},
    sync::Arc,
    time::Duration,
};
use winit::{
    dpi::PhysicalPosition,
    event::{ElementState, Event::*, VirtualKeyCode},
    event_loop::ControlFlow,
};

fn main() {
    change_working_directory();
    init_log();

    rs_media::init();
    let mut video_frame_player = VideoFramePlayer::new(
        &rs_computer_graphics::util::get_resource_path("Remote/BigBuckBunny.mp4"),
    );

    ThreadPool::audio().lock().unwrap().spawn(move || {
        let mut audio_device = rs_media::audio_device::AudioDevice::new();
        audio_device.play();
        let mut audio_player_item = AudioPlayerItem::new(
            &rs_computer_graphics::util::get_resource_path("Remote/BigBuckBunny.mp4"),
        );

        loop {
            {
                let buffer_mut = audio_device.get_buffer_mut();
                let mut buffer_mut = buffer_mut.lock().unwrap();

                while buffer_mut.len() < 1024 * 8 {
                    match audio_player_item.try_recv() {
                        Ok(frame) => {
                            let pcm_buffer = &frame.pcm_buffer;
                            let mut data: Vec<f32> = Vec::new();
                            let format = pcm_buffer.get_audio_format();
                            debug_assert_eq!(format.channels_per_frame, 2);
                            debug_assert_eq!(format.get_sample_type(), EAudioSampleType::Float32);
                            debug_assert_eq!(format.is_non_interleaved(), true);

                            let channel_data_0: &[f32] = pcm_buffer.get_channel_data_view(0);
                            let channel_data_1: &[f32] = pcm_buffer.get_channel_data_view(1);
                            for (first, second) in channel_data_0.iter().zip(channel_data_1.iter())
                            {
                                data.push(*first);
                                data.push(*second);
                            }
                            buffer_mut.append(&mut data);
                        }
                        Err(error) => match error {
                            rs_media::error::Error::EndOfFile => break,
                            rs_media::error::Error::TryAgain => {
                                std::thread::sleep(Duration::from_secs_f32(0.015));
                            }
                            rs_media::error::Error::Disconnected => break,
                        },
                    }
                }
            }
            std::thread::sleep(Duration::from_secs_f32(0.01));
        }
    });

    let (sender, receiver) = std::sync::mpsc::channel();
    ThreadPool::global().lock().unwrap().spawn(move || {
        let mut player_item = AudioFrameExtractor::new(
            &rs_computer_graphics::util::get_resource_path("Remote/sample-15s.mp3"),
        );
        // player_item.seek(5.0);
        let mut index = 0;
        let _ = std::fs::remove_dir_all("./dsp");
        let _ = std::fs::create_dir("./dsp");
        while let Some(frames) = player_item.next_frames() {
            for frame in &frames {
                let buffer: &[f32] = frame.pcm_buffer.get_channel_data_view(0);
                let mut planner = FftPlanner::<f32>::new();
                let fft = planner.plan_fft_forward(buffer.len());
                let mut signals: Vec<Complex<f32>> =
                    buffer.iter().map(|x| Complex { re: *x, im: 0.0 }).collect();
                fft.process(&mut signals);
                let result: Vec<f32> = signals.iter().map(|x| x.norm()).collect();
                let image_datas: Vec<u8> = result
                    .iter()
                    .map(|x| {
                        let x = x.clamp(0.0, 1.0);
                        (x * 255.0) as u8
                    })
                    .collect();
                let audio_image =
                    image::GrayImage::from_vec(image_datas.len() as u32, 1, image_datas).unwrap();
                std::thread::sleep(Duration::from_secs_f32(0.1));
                let _ = sender.send(audio_image);
                // save_fft_result(&format!("./dsp/fft_{}.png", index), &result);
                index += 1;
            }
        }
    });

    let native_window = NativeWindow::new();

    let mut wgpu_context = WGPUContext::new(
        &native_window.window,
        Some(wgpu::PowerPreference::HighPerformance),
        // None,
        Some(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::VULKAN,
            dx12_shader_compiler: wgpu::Dx12Compiler::Fxc,
        }),
    );

    #[cfg(feature = "rs_dotnet")]
    let mut dotnet_runtime = DotnetRuntime::new(&mut wgpu_context.device);

    #[cfg(feature = "rs_quickjs")]
    let mut js_runtime = QuickJSRuntimeContext::new();

    let mut user_script_change_monitor = UserScriptChangeMonitor::new();

    let window_size = native_window.window.inner_size();
    let swapchain_format = wgpu_context.get_current_swapchain_format();

    let mut egui_context = EGUIContext::new(
        &wgpu_context.device,
        swapchain_format,
        &native_window.window,
    );

    {
        DefaultTextures::default()
            .lock()
            .unwrap()
            .init(&wgpu_context.device, &wgpu_context.queue);
        ShaderLibrary::default().lock().unwrap().load_shader_from(
            &wgpu_context.device,
            &FileManager::default().lock().unwrap().get_shader_dir_path(),
        )
    }

    let mut actor = Actor::load_from_file(
        &wgpu_context.device,
        &wgpu_context.queue,
        &rs_computer_graphics::util::get_resource_path("Cube.dae"),
    );

    let mut audio_quad_actor = Actor::load_from_static_meshs(vec![StaticMesh::quad(
        "audio_quad",
        &wgpu_context.device,
        rs_computer_graphics::material_type::EMaterialType::Phong(
            rs_computer_graphics::material::Material::new(Arc::new(None), Arc::new(None)),
        ),
    )]);
    audio_quad_actor.set_world_location(glam::vec3(0.0, 2.0, 0.0));

    let mut video_quad_actor = Actor::load_from_static_meshs(vec![StaticMesh::quad(
        "video_quad",
        &wgpu_context.device,
        rs_computer_graphics::material_type::EMaterialType::Phong(
            rs_computer_graphics::material::Material::new(Arc::new(None), Arc::new(None)),
        ),
    )]);
    video_quad_actor.set_world_location(glam::vec3(2.0, 2.0, 0.0));

    let mut actor_pbr = Actor::load_from_file(
        &wgpu_context.device,
        &wgpu_context.queue,
        &rs_computer_graphics::util::get_resource_path("Remote/Test/untitled.fbx"),
    );

    let mut cone_actor = Actor::load_from_file(
        &wgpu_context.device,
        &wgpu_context.queue,
        &rs_computer_graphics::util::get_resource_path("Remote/Cone.fbx"),
    );

    let mut cube_virtual_texture_actor = Actor::load_from_file(
        &wgpu_context.device,
        &wgpu_context.queue,
        &rs_computer_graphics::util::get_resource_path("Remote/CubeVirtualTexture.fbx"),
    );

    let mut sphere_virtual_actor = Actor::load_from_file(
        &wgpu_context.device,
        &wgpu_context.queue,
        &rs_computer_graphics::util::get_resource_path("Remote/SphereVirtual.fbx"),
    );
    sphere_virtual_actor.set_world_location(glam::vec3(0.0, 0.0, -3.0));

    let mut camera = Camera::default(window_size.width, window_size.height);

    let mut last_mouse_position: Option<PhysicalPosition<f64>> = None;
    let mut is_cursor_visible = true;

    let mut virtual_key_code_state_map =
        std::collections::HashMap::<VirtualKeyCode, ElementState>::new();

    let phone_pipeline = PhongPipeline::new(
        &wgpu_context.device,
        Some(wgpu::DepthStencilState {
            depth_compare: wgpu::CompareFunction::Less,
            format: wgpu::TextureFormat::Depth32Float,
            depth_write_enabled: true,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        }),
        &swapchain_format,
    );

    let audio_pipeline = AudioPipeline::new(
        &wgpu_context.device,
        Some(wgpu::DepthStencilState {
            depth_compare: wgpu::CompareFunction::Less,
            format: wgpu::TextureFormat::Depth32Float,
            depth_write_enabled: true,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        }),
        &swapchain_format,
    );

    let pbr_pipeline = PBRPipeline::new(
        &wgpu_context.device,
        Some(wgpu::DepthStencilState {
            depth_compare: wgpu::CompareFunction::Less,
            format: wgpu::TextureFormat::Depth32Float,
            depth_write_enabled: true,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        }),
        &swapchain_format,
    );

    let attachment_pipeline = AttachmentPipeline::new(&wgpu_context.device, &swapchain_format);

    let sky_box_pipeline = SkyBoxPipeline::new(&wgpu_context.device, &swapchain_format);

    let mut baker = AccelerationBaker::new(
        &wgpu_context.device,
        &wgpu_context.queue,
        &rs_computer_graphics::util::get_resource_path("Remote/neon_photostudio_2k.exr"),
        BakeInfo {
            is_bake_environment: true,
            is_bake_irradiance: false,
            is_bake_brdflut: false,
            is_bake_pre_filter: false,
            environment_cube_map_length: 1024,
            irradiance_cube_map_length: 1024,
            irradiance_sample_count: 1024,
            pre_filter_cube_map_length: 1024,
            pre_filter_cube_map_max_mipmap_level: 6,
            pre_filter_sample_count: 1024,
            brdflutmap_length: 1024,
            brdf_sample_count: 1024,
            is_read_back: false,
        },
    );
    baker.bake(&wgpu_context.device, &wgpu_context.queue);

    {
        let default_textures = DefaultTextures::default();
        let default_textures = default_textures.lock().unwrap();
        for mesh in actor_pbr.get_static_meshs_mut() {
            let pbr_material = PBRMaterial::new(
                default_textures.get_black_texture(),
                default_textures.get_normal_texture(),
                default_textures.get_white_texture(),
                default_textures.get_white_texture(),
                baker.get_brdflut_texture(),
                baker.get_pre_filter_cube_map_textures(),
                baker.get_irradiance_cube_map_texture(),
            );
            mesh.set_material_type(EMaterialType::Pbr(pbr_material))
        }
    }

    let mut gizmo = FGizmo::default();

    let mut data_source = egui_context::DataSource {
        is_captrue_enable: false,
        is_save_frame_buffer: false,
        frame_buffer_color: egui::Color32::BLACK,
        target_fps: egui_context.get_fps(),
        roughness_factor: 0.0,
        metalness_factor: 1.0,
        draw_image: None,
        movement_speed: 0.01,
        motion_speed: 0.1,
        player_time: 0.0,
        seek_time: 0.0,
        is_seek: false,
        mipmap_level: 0,
    };

    let frmae_buffer = FrameBuffer::new(
        &wgpu_context.device,
        winit::dpi::PhysicalSize::<u32>::new(1024, 1024),
        swapchain_format,
    );

    let virtual_texture_configuration = VirtualTextureConfiguration {
        physical_texture_size: 4096,
        virtual_texture_size: 512 * 1000,
        tile_size: 256,
        physical_texture_array_size: 1,
    };
    let packing = Packing::new(virtual_texture_configuration);

    let div: u32 = 10;
    let mut virtual_texture_system = VirtualTextureSystem::new(
        &wgpu_context.device,
        virtual_texture_configuration,
        window_size.width / div,
        window_size.height / div,
        wgpu::TextureFormat::Rgba8Unorm,
    );

    let mut virtual_texture_cache = VirtualTextureAsyncLoader::new(virtual_texture_configuration);
    virtual_texture_cache.push(
        &rs_computer_graphics::util::get_resource_path("Remote/Untitled.png"),
        "Untitled",
    );
    virtual_texture_cache.push(
        &rs_computer_graphics::util::get_resource_path("Remote/pexels-pixmike-413195.png"),
        "pexels",
    );

    let virtual_texture_mesh_pipeline = VirtualTextureMeshPipeline::new(
        &wgpu_context.device,
        Some(wgpu::DepthStencilState {
            depth_compare: wgpu::CompareFunction::Less,
            format: wgpu::TextureFormat::Depth32Float,
            depth_write_enabled: true,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        }),
        &swapchain_format,
        virtual_texture_configuration,
    );

    let mut under_cursor_actor_index: Option<usize> = None;
    let mut selected_actor_index: Option<usize> = None;

    native_window.event_loop.run(move |event, _, control_flow| {
        egui_context.handle_event(&event);

        match event {
            RedrawRequested(..) => {
                let window_size = native_window.window.inner_size();
                camera.set_window_size(window_size.width, window_size.height);
                if user_script_change_monitor.is_changed() {
                    #[cfg(feature = "rs_dotnet")]
                    dotnet_runtime.reload_script();
                }
                egui_context.tick();

                for (virtual_key_code, element_state) in &virtual_key_code_state_map {
                    DefaultCameraInputEventHandle::keyboard_input_handle(
                        &mut camera,
                        virtual_key_code,
                        element_state,
                        is_cursor_visible,
                        data_source.movement_speed,
                    );
                }

                let swapchain_format = wgpu_context.get_current_swapchain_format();
                let surface = wgpu_context.get_current_surface_texture();
                let device = &wgpu_context.device;
                let queue = &wgpu_context.queue;
                let surface_texture = &surface.texture;
                let surface_texture_view =
                    surface_texture.create_view(&wgpu::TextureViewDescriptor::default());

                attachment_pipeline.draw(
                    device,
                    queue,
                    &surface_texture_view,
                    &wgpu_context.get_depth_texture_view(),
                    wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: true,
                    },
                    Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: true,
                    }),
                    Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(0),
                        store: true,
                    }),
                );

                {
                    virtual_texture_system.new_frame(device, queue);

                    for actor in [&cube_virtual_texture_actor, &sphere_virtual_actor] {
                        virtual_texture_system.render_actor_feed_back(
                            device,
                            queue,
                            actor,
                            window_size.width,
                            &camera,
                        );
                    }
                    let mut pages = virtual_texture_system.read(device, queue);

                    {
                        let mut extend: HashSet<TileIndex> = HashSet::new();
                        for page in pages.iter() {
                            let s = ((page.mipmap_level as i32) - 3).max(0);
                            let e = (page.mipmap_level as i32) + 3;
                            for mip_level in s..=e {
                                extend.insert(TileIndex {
                                    tile_offset: page.tile_offset,
                                    mipmap_level: mip_level as u8,
                                });
                            }
                        }
                        pages = extend.into_iter().collect();
                    }

                    let pack_result = packing.pack(&pages);

                    let mut batch_textures: Vec<Arc<wgpu::Texture>> = vec![];
                    let mut batch_array_tiles: Vec<&ArrayTile> = vec![];

                    for (page, array_tile) in pack_result.iter() {
                        if let Some(cache_texture) =
                            virtual_texture_cache.get_texture(device, queue, "Untitled", &page)
                        {
                            batch_textures.push(cache_texture);
                            batch_array_tiles.push(array_tile);
                        }
                        if let Some(cache_texture) =
                            virtual_texture_cache.get_texture(device, queue, "pexels", &page)
                        {
                            batch_textures.push(cache_texture);
                            batch_array_tiles.push(array_tile);
                        }
                    }
                    virtual_texture_system.upload_physical_page_textures(
                        device,
                        queue,
                        &batch_textures,
                        &batch_array_tiles,
                    );

                    if pack_result.is_empty() == false {
                        virtual_texture_system.update_page_table(device, queue, &pack_result);
                    }

                    for actor in [&mut cube_virtual_texture_actor, &mut sphere_virtual_actor] {
                        for mesh in actor.get_static_meshs_mut() {
                            let mut material = rs_computer_graphics::material::Material::default();
                            material.set_page_table_texture(
                                virtual_texture_system.get_page_table_texture(),
                            );
                            material.set_physical_texture(
                                virtual_texture_system.get_physical_texture(),
                            );
                            mesh.set_material_type(EMaterialType::Phong(material))
                        }
                    }

                    for actor in [&mut cube_virtual_texture_actor, &mut sphere_virtual_actor] {
                        virtual_texture_mesh_pipeline.render_actor(
                            device,
                            queue,
                            &surface_texture_view,
                            &wgpu_context.get_depth_texture_view(),
                            actor,
                            &camera,
                            Some(wgpu::Operations {
                                load: wgpu::LoadOp::Load,
                                store: true,
                            }),
                            Some(wgpu::Operations {
                                load: wgpu::LoadOp::Load,
                                store: true,
                            }),
                        );
                    }

                    if data_source.draw_image.is_none() {
                        data_source.draw_image = Some(egui_context.create_image(
                            device,
                            &virtual_texture_system.get_physical_texture_view(),
                            egui::Vec2 {
                                x: 256 as f32,
                                y: 256 as f32,
                            },
                        ));
                    }
                }

                // #[cfg(feature = "rs_dotnet")]
                // dotnet_runtime.application.redraw_requested(
                //     NativeWGPUTextureView {
                //         texture_view: (&surface_texture_view),
                //     },
                //     NativeWGPUQueue { queue },
                // );

                if let Ok(audio_image) = receiver.recv_timeout(std::time::Duration::from_millis(2))
                {
                    for mesh in audio_quad_actor.get_static_meshs_mut() {
                        let audio_texture = rs_computer_graphics::util::texture2d_from_gray_image(
                            device,
                            queue,
                            &audio_image,
                        );
                        let material = rs_computer_graphics::material::Material::new(
                            std::sync::Arc::new(Some(audio_texture)),
                            std::sync::Arc::new(None),
                        );
                        mesh.set_material_type(EMaterialType::Phong(material))
                    }
                }

                if video_frame_player.is_playing() == false {
                    video_frame_player.start();
                }

                video_frame_player.tick();

                match video_frame_player.get_current_frame() {
                    Some(video_frame) => {
                        for mesh in video_quad_actor.get_static_meshs_mut() {
                            let video_texture =
                                rs_computer_graphics::util::texture2d_from_rgba_image(
                                    device,
                                    queue,
                                    &video_frame.image,
                                );
                            let material = rs_computer_graphics::material::Material::new(
                                std::sync::Arc::new(Some(video_texture)),
                                std::sync::Arc::new(None),
                            );
                            mesh.set_material_type(EMaterialType::Phong(material))
                        }
                    }
                    None => {}
                }

                if data_source.is_seek {
                    video_frame_player.seek(data_source.seek_time);
                } else {
                    data_source.seek_time = data_source.player_time;
                }
                data_source.player_time = video_frame_player.get_current_play_time();

                phone_pipeline.render_actor(
                    device,
                    queue,
                    &surface_texture_view,
                    &wgpu_context.get_depth_texture_view(),
                    &video_quad_actor,
                    &camera,
                );

                audio_pipeline.render_actor(
                    device,
                    queue,
                    &surface_texture_view,
                    &wgpu_context.get_depth_texture_view(),
                    &audio_quad_actor,
                    &camera,
                );

                match baker.get_environment_cube_texture().borrow() {
                    Some(texture) => sky_box_pipeline.render(
                        device,
                        queue,
                        &surface_texture_view,
                        &wgpu_context.get_depth_texture_view(),
                        texture,
                        &camera,
                    ),
                    None => {}
                }

                // pbr_pipeline.render_actor(
                //     device,
                //     queue,
                //     &surface_texture_view,
                //     &wgpu_context.get_depth_texture_view(),
                //     &actor_pbr,
                //     &camera,
                //     data_source.roughness_factor,
                //     data_source.metalness_factor,
                //     DirectionalLight::default(),
                //     PointLight::default(),
                //     SpotLight::default(),
                // );

                // phone_pipeline.render_actor(
                //     device,
                //     queue,
                //     &surface_texture_view,
                //     &wgpu_context.get_depth_texture_view(),
                //     &actor,
                //     &camera,
                // );

                egui_context.draw_ui(
                    queue,
                    device,
                    &native_window.window,
                    &surface_texture_view,
                    &mut data_source,
                );

                egui_context.set_fps(data_source.target_fps);
                egui_context.sync_fps(control_flow);
                egui_context.gizmo_settings(&mut gizmo, &mut data_source);

                {
                    let mut actors = vec![
                        // &mut actor,
                        &mut audio_quad_actor,
                        &mut video_quad_actor,
                        // &mut actor_pbr,
                        // &mut cone_actor,
                        &mut cube_virtual_texture_actor,
                    ];
                    match selected_actor_index {
                        Some(index) => match actors.get_mut(index) {
                            Some(actor) => {
                                egui::Area::new("Gizmo Viewport")
                                    .fixed_pos((0.0, 0.0))
                                    .show(&egui_context.get_platform_context(), |ui| {
                                        ui.with_layer_id(egui::LayerId::background(), |ui| {
                                            if let Some(model_matrix) = gizmo.interact(
                                                &camera,
                                                ui,
                                                actor.get_model_matrix(),
                                            ) {
                                                actor.set_model_matrix(model_matrix);
                                            }
                                        });
                                    });
                            }
                            None => {}
                        },
                        None => {}
                    }
                }

                // actor.set_world_location(data_source.mesh_location);
                // actor.set_rotator(data_source.mesh_rotator);

                if data_source.is_captrue_enable {
                    CaptureScreen::capture(
                        &std::format!("./CaptureScreen_{:?}.png", egui_context.get_render_ticks()),
                        device,
                        queue,
                        surface_texture,
                        swapchain_format,
                        &window_size,
                    );
                    data_source.is_captrue_enable = false;
                }
                if data_source.is_save_frame_buffer {
                    let color = data_source.frame_buffer_color;
                    let color = wgpu::Color {
                        r: color.r() as f64 / 255.0,
                        g: color.g() as f64 / 255.0,
                        b: color.b() as f64 / 255.0,
                        a: color.a() as f64 / 255.0,
                    };
                    attachment_pipeline.draw(
                        device,
                        queue,
                        &frmae_buffer.get_color_texture_view(),
                        &frmae_buffer.get_depth_texture_view(),
                        wgpu::Operations {
                            load: wgpu::LoadOp::Clear(color),
                            store: true,
                        },
                        Some(wgpu::Operations {
                            load: wgpu::LoadOp::Clear(1.0),
                            store: true,
                        }),
                        Some(wgpu::Operations {
                            load: wgpu::LoadOp::Clear(0),
                            store: true,
                        }),
                    );
                    if let Some(frame_buffer_image) = frmae_buffer.capture(device, queue) {
                        ThreadPool::io().lock().unwrap().spawn(move || {
                            match frame_buffer_image.save(std::format!("./frame_buffer_image.png"))
                            {
                                Ok(_) => {}
                                Err(error) => panic!("{}", error),
                            }
                        });
                    }
                    data_source.is_save_frame_buffer = false;
                }

                surface.present();
            }
            MainEventsCleared => {
                native_window.window.request_redraw();
            }
            winit::event::Event::DeviceEvent { event, .. } => match event {
                winit::event::DeviceEvent::MouseMotion { delta } => {
                    DefaultCameraInputEventHandle::mouse_motion_handle(
                        &mut camera,
                        delta,
                        is_cursor_visible,
                        data_source.motion_speed,
                    );
                }
                _ => {}
            },
            WindowEvent { event, .. } => match event {
                winit::event::WindowEvent::Resized(size) => {
                    log::trace!("Window resized to {:?}", size);
                    wgpu_context.window_resized(size);
                }
                winit::event::WindowEvent::CloseRequested => {
                    *control_flow = ControlFlow::Exit;
                }
                winit::event::WindowEvent::MouseInput {
                    device_id,
                    state,
                    button,
                    modifiers,
                } => match button {
                    winit::event::MouseButton::Left => {
                        if is_cursor_visible {
                            match under_cursor_actor_index {
                                Some(index) => selected_actor_index = Some(index),
                                None => selected_actor_index = None,
                            }
                        }
                    }
                    winit::event::MouseButton::Right => {}
                    winit::event::MouseButton::Middle => {}
                    winit::event::MouseButton::Other(_) => {}
                },
                winit::event::WindowEvent::CursorMoved {
                    device_id: _,
                    position,
                    modifiers: _,
                } => {
                    if is_cursor_visible {
                        let window_size = native_window.window.inner_size();
                        match ActorSelector::select(
                            vec![
                                // &actor,
                                &audio_quad_actor,
                                &video_quad_actor,
                                // &actor_pbr,
                                // &cone_actor,
                                &cube_virtual_texture_actor,
                            ],
                            position,
                            window_size,
                            &camera,
                        ) {
                            Some((index, _)) => under_cursor_actor_index = Some(index),
                            None => under_cursor_actor_index = None,
                        }
                    }

                    last_mouse_position = Some(position);

                    #[cfg(feature = "rs_dotnet")]
                    dotnet_runtime.application.cursor_moved(position);
                }
                winit::event::WindowEvent::KeyboardInput {
                    device_id: _,
                    input,
                    is_synthetic: _,
                } => {
                    if let Some(virtual_keycode) = input.virtual_keycode {
                        #[cfg(feature = "rs_dotnet")]
                        dotnet_runtime
                            .application
                            .keyboard_input(NativeKeyboardInput {
                                scancode: input.scancode,
                                state: {
                                    if input.state == winit::event::ElementState::Pressed {
                                        0
                                    } else {
                                        1
                                    }
                                },
                                virtual_key_code: virtual_keycode as i32,
                            });
                    }

                    if let Some(keycode) = input.virtual_keycode {
                        virtual_key_code_state_map.insert(keycode, input.state);
                        if keycode == winit::event::VirtualKeyCode::Escape
                            && input.state == ElementState::Released
                        {
                            *control_flow = ControlFlow::Exit;
                        }

                        if keycode == winit::event::VirtualKeyCode::F1
                            && input.state == ElementState::Released
                        {
                            is_cursor_visible = !is_cursor_visible;
                            native_window.window.set_cursor_visible(is_cursor_visible);
                            if is_cursor_visible {
                                native_window
                                    .window
                                    .set_cursor_grab(winit::window::CursorGrabMode::None)
                                    .unwrap();
                            } else {
                                native_window
                                    .window
                                    .set_cursor_grab(winit::window::CursorGrabMode::Confined)
                                    .unwrap();
                            }
                        }
                    }
                }
                _ => {}
            },
            _ => (),
        }
    });
}
