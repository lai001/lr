use glam::Vec4Swizzles;
use rs_computer_graphics::{
    camera::Camera,
    demo::{
        capture_screen::CaptureScreen, compute_demo::ComputeDemo, cube_demo::CubeDemo,
        triangle_demo::TriangleDemo,
    },
    dotnet_runtime::DotnetRuntime,
    egui_context::EGUIContext,
    ffi::{
        native_keyboard_input::NativeKeyboardInput, native_queue::NativeWGPUQueue,
        native_texture_view::NativeWGPUTextureView,
    },
    native_window::NativeWindow,
    user_script_change_monitor::UserScriptChangeMonitor,
    util::{
        change_working_directory, math_remap_value_range, screent_space_to_world_space, shape,
        triangle_plane_ray_intersection,
    },
    wgpu_context::WGPUContext,
};
use winit::{
    dpi::PhysicalPosition,
    event::{ElementState, Event::*, VirtualKeyCode},
    event_loop::ControlFlow,
};

fn main() {
    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or("rs_computer_graphics,rs_dotnet"),
    )
    .init();
    change_working_directory();
    let native_window = NativeWindow::new();

    let mut wgpu_context = WGPUContext::new(&native_window.window);

    let mut dotnet_runtime = DotnetRuntime::new(&mut wgpu_context.device);

    let mut user_script_change_monitor = UserScriptChangeMonitor::new();

    let window_size = native_window.window.inner_size();
    let swapchain_format = wgpu_context.get_surface_capabilities().formats[0];

    let mut egui_context = EGUIContext::new(
        &wgpu_context.device,
        swapchain_format,
        &native_window.window,
    );

    let triangle_demo = TriangleDemo::new(&wgpu_context.device, &swapchain_format);
    let mut cube_demo = CubeDemo::new(
        &wgpu_context.device,
        &swapchain_format,
        &wgpu_context.queue,
        window_size.width,
        window_size.height,
    );
    let compute_demo = ComputeDemo::new(&wgpu_context.device);

    let mut camera = Camera::default(window_size.width, window_size.height);

    let mut last_mouse_position: Option<PhysicalPosition<f64>> = None;
    let mut is_cursor_visible = true;

    let mut virtual_key_code_state_map =
        std::collections::HashMap::<VirtualKeyCode, ElementState>::new();

    native_window.event_loop.run(move |event, _, control_flow| {
        egui_context.platform.handle_event(&event);

        match event {
            RedrawRequested(..) => {
                if user_script_change_monitor.is_changed() {
                    dotnet_runtime.reload_script();
                }
                egui_context.tick();

                for (virtual_key_code, element_state) in &virtual_key_code_state_map {
                    let speed = 0.05_f32;
                    if virtual_key_code == &winit::event::VirtualKeyCode::W
                        && element_state == &ElementState::Pressed
                        && is_cursor_visible == false
                    {
                        camera.add_world_absolute_location(glam::Vec3 {
                            x: 0.0,
                            y: 0.0,
                            z: -1.0 * speed,
                        });
                    }
                    if virtual_key_code == &winit::event::VirtualKeyCode::A
                        && element_state == &ElementState::Pressed
                        && is_cursor_visible == false
                    {
                        camera.add_world_absolute_location(glam::Vec3 {
                            x: -1.0 * speed,
                            y: 0.0,
                            z: 0.0,
                        });
                    }
                    if virtual_key_code == &winit::event::VirtualKeyCode::S
                        && element_state == &ElementState::Pressed
                        && is_cursor_visible == false
                    {
                        camera.add_world_absolute_location(glam::Vec3 {
                            x: 0.0,
                            y: 0.0,
                            z: 1.0 * speed,
                        });
                    }
                    if virtual_key_code == &winit::event::VirtualKeyCode::D
                        && element_state == &ElementState::Pressed
                        && is_cursor_visible == false
                    {
                        camera.add_world_absolute_location(glam::Vec3 {
                            x: 1.0 * speed,
                            y: 0.0,
                            z: 0.0,
                        });
                    }
                }

                let swapchain_format = wgpu_context.get_current_swapchain_format();
                let surface = &wgpu_context.surface;
                let device = &wgpu_context.device;
                let queue = &mut wgpu_context.queue;

                let output_frame = surface.get_current_texture().unwrap();
                let mut output_view = output_frame
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());

                triangle_demo.draw(device, &output_view, queue);
                cube_demo.draw(device, &output_view, queue, &camera);
                let compute_result = compute_demo.execute(&(0..16).collect(), device, queue);
                log::debug!("{:?}", compute_result);

                dotnet_runtime.application.redraw_requested(
                    NativeWGPUTextureView {
                        texture_view: (&mut output_view),
                    },
                    NativeWGPUQueue { queue },
                );

                let data_source = egui_context.draw_ui(queue, device, &output_view);

                if data_source.is_captrue_enable {
                    CaptureScreen::capture(
                        &std::format!("./CaptureScreen_{:?}.png", egui_context.get_render_ticks()),
                        device,
                        queue,
                        &output_frame.texture,
                        swapchain_format,
                        &window_size,
                    );
                }

                output_frame.present();
            }
            MainEventsCleared => {
                native_window.window.request_redraw();
            }
            winit::event::Event::DeviceEvent { event, .. } => match event {
                winit::event::DeviceEvent::MouseMotion { delta } => {
                    if is_cursor_visible == false {
                        let speed = 0.2_f32;
                        let dx: f32;
                        if delta.0.is_sign_negative() {
                            dx = 1.0;
                        } else {
                            dx = -1.0;
                        }
                        let dy: f32;
                        if delta.1.is_sign_negative() {
                            dy = 1.0;
                        } else {
                            dy = -1.0;
                        }
                        camera.add_rotation(
                            glam::Vec3 {
                                x: 0.0,
                                y: dx,
                                z: 0.0,
                            },
                            (delta.0 as f32).abs() * speed,
                        );
                        camera.add_rotation(
                            glam::Vec3 {
                                x: dy,
                                y: 0.0,
                                z: 0.0,
                            },
                            (delta.1 as f32).abs() * speed,
                        );
                    }
                }
                _ => {}
            },
            WindowEvent { event, .. } => match event {
                winit::event::WindowEvent::Resized(size) => {
                    wgpu_context.window_resized(size);
                }
                winit::event::WindowEvent::CloseRequested => {
                    *control_flow = ControlFlow::Exit;
                }
                winit::event::WindowEvent::CursorMoved {
                    device_id: _,
                    position,
                    modifiers: _,
                } => {
                    let x = math_remap_value_range(
                        position.x as f64,
                        std::ops::Range::<f64> {
                            start: 0.0,
                            end: window_size.width as f64,
                        },
                        std::ops::Range::<f64> {
                            start: -1.0,
                            end: 1.0,
                        },
                    ) as f32;
                    let y = -math_remap_value_range(
                        position.y as f64,
                        std::ops::Range::<f64> {
                            start: 0.0,
                            end: window_size.height as f64,
                        },
                        std::ops::Range::<f64> {
                            start: -1.0,
                            end: 1.0,
                        },
                    ) as f32;

                    let near_point = glam::Vec3::new(x, y, 0.0);
                    let far_point = glam::Vec3::new(x, y, 1.0);

                    let near_point_at_world_space = screent_space_to_world_space(
                        near_point,
                        cube_demo.model_matrix,
                        camera.get_view_matrix(),
                        camera.get_projection_matrix(),
                    );

                    let far_point_at_world_space = screent_space_to_world_space(
                        far_point,
                        cube_demo.model_matrix,
                        camera.get_view_matrix(),
                        camera.get_projection_matrix(),
                    );

                    let triangles = shape(
                        cube_demo.vertex_data[0].pos.xyz(),
                        cube_demo.vertex_data[1].pos.xyz(),
                        cube_demo.vertex_data[2].pos.xyz(),
                        cube_demo.vertex_data[3].pos.xyz(),
                    );

                    for triangle in triangles {
                        let a = glam::vec4(triangle.0.x, triangle.0.y, triangle.0.z, 1.0);
                        let b = glam::vec4(triangle.1.x, triangle.1.y, triangle.1.z, 1.0);
                        let c = glam::vec4(triangle.2.x, triangle.2.y, triangle.2.z, 1.0);
                        let a_world_location = a;
                        let b_world_location = b;
                        let c_world_location = c;

                        let intersection_point = triangle_plane_ray_intersection(
                            a_world_location.xyz(),
                            b_world_location.xyz(),
                            c_world_location.xyz(),
                            near_point_at_world_space,
                            far_point_at_world_space - near_point_at_world_space,
                        );
                        log::trace!("{:?}", intersection_point);
                    }

                    // log::debug!(
                    //     "{:?}, {:?}",
                    //     near_point_at_world_space,
                    //     far_point_at_world_space
                    // );

                    last_mouse_position = Some(position);

                    dotnet_runtime.application.cursor_moved(position);
                }
                winit::event::WindowEvent::KeyboardInput {
                    device_id: _,
                    input,
                    is_synthetic: _,
                } => {
                    if let Some(virtual_keycode) = input.virtual_keycode {
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
                        }
                    }
                }
                _ => {}
            },
            _ => (),
        }
    });
}
