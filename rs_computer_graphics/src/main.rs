#[cfg(feature = "rs_dotnet")]
use rs_computer_graphics::dotnet_runtime::DotnetRuntime;
#[cfg(feature = "rs_quickjs")]
use rs_computer_graphics::quickjs::quickjs_runtime::QuickJSRuntimeContext;
use rs_computer_graphics::{
    acceleration_bake::AccelerationBaker,
    actor::Actor,
    bake_info::BakeInfo,
    camera::{Camera, CameraInputEventHandle, DefaultCameraInputEventHandle},
    default_textures::DefaultTextures,
    demo::{
        capture_screen::CaptureScreen, compute_demo::ComputeDemo, cube_demo::CubeDemo,
        panorama_to_cube_demo::PanoramaToCubeDemo, triangle_demo::TriangleDemo,
        yuv420p_demo::YUV420PDemo,
    },
    egui_context::EGUIContext,
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
    thread_pool,
    user_script_change_monitor::UserScriptChangeMonitor,
    util::{change_working_directory, init_log},
    virtual_texture::{block_image::BlockImage, virtual_texture_system::VirtualTextureSystem},
    wgpu_context::WGPUContext,
};
use rs_media::{audio_player_item::AudioPlayerItem, video_player_item::EVideoDecoderType};
use rustfft::{num_complex::Complex, FftPlanner};
use std::{borrow::Borrow, sync::Arc, time::Duration};
use winit::{
    dpi::PhysicalPosition,
    event::{ElementState, Event::*, VirtualKeyCode},
    event_loop::ControlFlow,
};

fn main() {
    init_log();
    change_working_directory();

    // thread_pool::ThreadPool::global().lock().unwrap().spawn(|| {
    //     rs_media::hw::hw_test(&rs_computer_graphics::util::get_resource_path(
    //         "Remote/BigBuckBunny.mp4",
    //     ));
    // });
    // thread_pool::ThreadPool::global().lock().unwrap().spawn(|| {
    //     rs_media::sw::sw_test(&rs_computer_graphics::util::get_resource_path(
    //         "Remote/BigBuckBunny.mp4",
    //     ));
    // });
    rs_media::init();
    let (video_sender, video_receiver) = std::sync::mpsc::channel();
    let video_sender_clone = video_sender.clone();
    thread_pool::ThreadPool::global()
        .lock()
        .unwrap()
        .spawn(move || {
            let filepath = rs_computer_graphics::util::get_resource_path("Remote/BigBuckBunny.mp4");
            let mut video_player_item = rs_media::video_player_item::VideoPlayerItem::new(
                &filepath,
                Some(EVideoDecoderType::Hardware),
            );
            while let Some(frames) = video_player_item.next_frames() {
                for frame in frames {
                    let _ = video_sender_clone.send(frame);
                    std::thread::sleep(Duration::from_secs_f32(1.0 / 24.0));
                }
            }
        });
    let (sender, receiver) = std::sync::mpsc::channel();
    let sender_clone = sender.clone();
    thread_pool::ThreadPool::audio()
        .lock()
        .unwrap()
        .spawn(move || {
            let mut player_item = AudioPlayerItem::new(
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
                        image::GrayImage::from_vec(image_datas.len() as u32, 1, image_datas)
                            .unwrap();
                    std::thread::sleep(Duration::from_secs_f32(0.1));
                    let _ = sender_clone.send(audio_image);
                    // save_fft_result(&format!("./dsp/fft_{}.png", index), &result);
                    index += 1;
                }
            }
            let mut audio_device = rs_media::audio_device::AudioDevice::new();
            audio_device.run();
        });

    drop(sender);
    drop(video_sender);

    let native_window = NativeWindow::new();

    let mut wgpu_context = WGPUContext::new(&native_window.window, None, None);

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
    }

    // let mut actor = Actor::load_from_file(
    //     &wgpu_context.device,
    //     &wgpu_context.queue,
    //     &rs_computer_graphics::util::get_resource_path("Axis.fbx"),
    // );
    let mut actor = Actor::load_from_file(
        &wgpu_context.device,
        &wgpu_context.queue,
        &rs_computer_graphics::util::get_resource_path("Cube.dae"),
    );

    let mut quad_actor = Actor::load_from_static_meshs(vec![StaticMesh::quad(
        "quad",
        &wgpu_context.device,
        rs_computer_graphics::material_type::EMaterialType::Phong(
            rs_computer_graphics::material::Material::new(Arc::new(None), Arc::new(None)),
        ),
    )]);

    let mut video_quad_actor = Actor::load_from_static_meshs(vec![StaticMesh::quad(
        "video_quad",
        &wgpu_context.device,
        rs_computer_graphics::material_type::EMaterialType::Phong(
            rs_computer_graphics::material::Material::new(Arc::new(None), Arc::new(None)),
        ),
    )]);

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

    let shader_lib = ShaderLibrary::default();
    {
        shader_lib.lock().unwrap().load_shader_from(
            &wgpu_context.device,
            &FileManager::default().lock().unwrap().get_shader_dir_path(),
        )
    }
    let triangle_demo = TriangleDemo::new(&wgpu_context.device, &swapchain_format);
    let mut cube_demo = CubeDemo::new(
        &wgpu_context.device,
        &swapchain_format,
        &wgpu_context.queue,
        window_size.width,
        window_size.height,
    );
    let compute_demo = ComputeDemo::new(&wgpu_context.device);
    let panorama_to_cube_demo = PanoramaToCubeDemo::new(&wgpu_context.device, &wgpu_context.queue);
    let yuvimage_demo =
        YUV420PDemo::new(&wgpu_context.device, &wgpu_context.queue, &swapchain_format);

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

    let hdr_filepath =
        rs_computer_graphics::util::get_resource_path("Remote/neon_photostudio_2k.exr");
    let mut baker = AccelerationBaker::new(
        &wgpu_context.device,
        &wgpu_context.queue,
        hdr_filepath,
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

    let mut data_source = rs_computer_graphics::egui_context::DataSource {
        is_captrue_enable: false,
        is_save: false,
        is_save_frame_buffer: false,
        frame_buffer_color: egui::Color32::BLACK,
        target_fps: egui_context.get_fps(),
        roughness_factor: 0.0,
        metalness_factor: 1.0,
        draw_image: None,
        movement_speed: 0.01,
        motion_speed: 0.1,
    };

    let frmae_buffer = FrameBuffer::new(
        &wgpu_context.device,
        winit::dpi::PhysicalSize::<u32>::new(1024, 1024),
        swapchain_format,
    );

    let mut vt_flow = VirtualTextureSystem::new(
        &wgpu_context.device,
        4096,
        512 * 1000,
        256,
        window_size.width / 8,
        window_size.height / 8,
        wgpu::TextureFormat::Rgba8Unorm,
    );

    let mut block_image = BlockImage::new(&rs_computer_graphics::util::get_resource_path(
        "Remote/Untitled_4k.png",
    ));

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
    );

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
                    vt_flow.new_frame(device, queue);

                    let mut vt_camera = camera.clone();
                    let feed_back_texture_size = vt_flow.get_feed_back_texture_size();
                    vt_camera.set_window_size(
                        feed_back_texture_size.width,
                        feed_back_texture_size.height,
                    );
                    vt_flow.render_actor(device, queue, &cube_virtual_texture_actor, &vt_camera);
                    let pages = vt_flow.read(device, queue);

                    for page in &pages {
                        if let Some(cache_image) =
                            block_image.get_image(page.0 as u32, page.1 as u32)
                        {
                            vt_flow.update_page_table(
                                page.0 as u32,
                                page.1 as u32,
                                page.0,
                                page.1,
                                0,
                                0,
                            );
                            vt_flow.upload_page_image(device, queue, *page, cache_image);
                        };
                    }

                    vt_flow.upload_page_table(queue);

                    for mesh in cube_virtual_texture_actor.get_static_meshs_mut() {
                        let mut material = rs_computer_graphics::material::Material::default();
                        material.set_page_table_texture(vt_flow.get_page_table_texture());
                        material.set_physical_texture(vt_flow.get_physical_texture());
                        mesh.set_material_type(EMaterialType::Phong(material))
                    }

                    virtual_texture_mesh_pipeline.render_actor(
                        device,
                        queue,
                        &surface_texture_view,
                        &wgpu_context.get_depth_texture_view(),
                        &cube_virtual_texture_actor,
                        &camera,
                        Some(wgpu::Operations {
                            load: wgpu::LoadOp::Clear(1.0),
                            store: true,
                        }),
                        Some(wgpu::Operations {
                            load: wgpu::LoadOp::Clear(0),
                            store: true,
                        }),
                    );

                    data_source.draw_image = Some(egui_context.create_image(
                        device,
                        &vt_flow.get_physical_texture_view(),
                        egui::Vec2 {
                            x: 256 as f32,
                            y: 256 as f32,
                        },
                    ));
                }

                // triangle_demo.draw(device, &surface_texture_view, queue);
                // cube_demo.draw(device, &surface_texture_view, queue, &camera);
                // let compute_result = compute_demo.execute(&(0..16).collect(), device, queue);
                // log::debug!("{:?}", compute_result);

                // #[cfg(feature = "rs_dotnet")]
                // dotnet_runtime.application.redraw_requested(
                //     NativeWGPUTextureView {
                //         texture_view: (&surface_texture_view),
                //     },
                //     NativeWGPUQueue { queue },
                // );

                // yuvimage_demo.render(
                //     vec![
                //         Image2DVertex {
                //             pos: glam::vec2(-1.0, 1.0),
                //             uv: glam::vec2(0.0, 0.0),
                //         },
                //         Image2DVertex {
                //             pos: glam::vec2(0.0, 1.0),
                //             uv: glam::vec2(1.0, 0.0),
                //         },
                //         Image2DVertex {
                //             pos: glam::vec2(0.0, 0.0),
                //             uv: glam::vec2(1.0, 1.0),
                //         },
                //         Image2DVertex {
                //             pos: glam::vec2(-1.0, 0.0),
                //             uv: glam::vec2(0.0, 1.0),
                //         },
                //     ],
                //     device,
                //     &output_view,
                //     queue,
                // );

                if let Ok(audio_image) = receiver.recv_timeout(std::time::Duration::from_millis(2))
                {
                    for mesh in quad_actor.get_static_meshs_mut() {
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

                if let Ok(video_image) =
                    video_receiver.recv_timeout(std::time::Duration::from_millis(2))
                {
                    for mesh in video_quad_actor.get_static_meshs_mut() {
                        let video_texture = rs_computer_graphics::util::texture2d_from_rgba_image(
                            device,
                            queue,
                            &video_image.image,
                        );
                        let material = rs_computer_graphics::material::Material::new(
                            std::sync::Arc::new(Some(video_texture)),
                            std::sync::Arc::new(None),
                        );
                        mesh.set_material_type(EMaterialType::Phong(material))
                    }
                }

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
                    &quad_actor,
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
                egui_context.gizmo_settings(&mut gizmo);

                egui::Area::new("Gizmo Viewport")
                    .fixed_pos((0.0, 0.0))
                    .show(&egui_context.get_platform_context(), |ui| {
                        ui.with_layer_id(egui::LayerId::background(), |ui| {
                            let actor = &mut cone_actor;
                            if let Some(model_matrix) =
                                gizmo.interact(&camera, ui, actor.get_model_matrix())
                            {
                                actor.set_model_matrix(model_matrix);
                            }
                        });
                    });

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
                        thread_pool::ThreadPool::io()
                            .lock()
                            .unwrap()
                            .spawn(move || {
                                match frame_buffer_image
                                    .save(std::format!("./frame_buffer_image.png"))
                                {
                                    Ok(_) => {}
                                    Err(error) => panic!("{}", error),
                                }
                            });
                    }
                    data_source.is_save_frame_buffer = false;
                }
                if data_source.is_save {
                    panorama_to_cube_demo.execute(device, queue);
                    data_source.is_save = false;
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
                } => {
                    // match button {
                    //     winit::event::MouseButton::Left => todo!(),
                    //     winit::event::MouseButton::Right => todo!(),
                    //     winit::event::MouseButton::Middle => todo!(),
                    //     winit::event::MouseButton::Other(_) => todo!(),
                    // }
                }
                winit::event::WindowEvent::CursorMoved {
                    device_id: _,
                    position,
                    modifiers: _,
                } => {
                    if is_cursor_visible {
                        let hit_test_results =
                            rs_computer_graphics::util::ray_intersection_hit_test(
                                &actor,
                                position,
                                window_size,
                                *actor.get_model_matrix(),
                                &camera,
                            );

                        for result in hit_test_results {
                            log::trace!("{:?}", result);
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
