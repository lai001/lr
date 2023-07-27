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
    gizmo::FGizmo,
    light::{DirectionalLight, PointLight, SpotLight},
    material_type::EMaterialType,
    native_window::NativeWindow,
    pbr_material::PBRMaterial,
    render_pipeline::{
        attachment_pipeline::AttachmentPipeline, pbr_pipeline::PBRPipeline,
        phong_pipeline::PhongPipeline, sky_box_pipeline::SkyBoxPipeline,
    },
    shader::shader_library::ShaderLibrary,
    user_script_change_monitor::UserScriptChangeMonitor,
    util::{change_working_directory, init_log},
    wgpu_context::WGPUContext,
};
use std::borrow::Borrow;
use winit::{
    dpi::PhysicalPosition,
    event::{ElementState, Event::*, VirtualKeyCode},
    event_loop::ControlFlow,
};

fn main() {
    init_log();
    change_working_directory();

    let native_window = NativeWindow::new();

    let mut wgpu_context =
        WGPUContext::new(&native_window.window, Some(wgpu::PowerPreference::LowPower));

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

    let mut actor_pbr = Actor::load_from_file(
        &wgpu_context.device,
        &wgpu_context.queue,
        &rs_computer_graphics::util::get_resource_path("Remote/Test/untitled.fbx"),
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
            is_bake_irradiance: true,
            is_bake_brdflut: true,
            is_bake_pre_filter: true,
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
        target_fps: egui_context.get_fps(),
    };

    native_window.event_loop.run(move |event, _, control_flow| {
        egui_context.handle_event(&event);

        match event {
            RedrawRequested(..) => {
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

                pbr_pipeline.render_actor(
                    device,
                    queue,
                    &surface_texture_view,
                    &wgpu_context.get_depth_texture_view(),
                    &actor_pbr,
                    &camera,
                    0.1,
                    0.0,
                    DirectionalLight::default(),
                    PointLight::default(),
                    SpotLight::default(),
                );

                // phone_pipeline.render_actor(
                //     device,
                //     queue,
                //     &surface_texture_view,
                //     &wgpu_context.get_depth_texture_view(),
                //     &actor,
                //     &camera,
                // );

                egui_context.draw_ui(queue, device, &surface_texture_view, &mut data_source);
                egui_context.set_fps(data_source.target_fps);
                egui_context.sync_fps(control_flow);
                egui_context.gizmo_settings(&mut gizmo);

                egui::Area::new("Gizmo Viewport")
                    .fixed_pos((0.0, 0.0))
                    .show(&egui_context.get_platform_context(), |ui| {
                        ui.with_layer_id(egui::LayerId::background(), |ui| {
                            if let Some(model_matrix) =
                                gizmo.interact(&camera, ui, actor_pbr.get_model_matrix())
                            {
                                actor_pbr.set_model_matrix(model_matrix);
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
