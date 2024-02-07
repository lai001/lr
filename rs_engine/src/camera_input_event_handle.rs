use crate::{camera::Camera, rotator::Rotator};
use winit::event::ElementState;

pub trait CameraInputEventHandle {
    fn mouse_motion_handle(
        camera: &mut Camera,
        delta: (f64, f64),
        is_cursor_visible: bool,
        motion_speed: f32,
    );
    fn keyboard_input_handle(
        camera: &mut Camera,
        virtual_key_code: &winit::keyboard::KeyCode,
        element_state: &winit::event::ElementState,
        is_cursor_visible: bool,
        movement_speed: f32,
    );
}

pub struct DefaultCameraInputEventHandle {}

impl CameraInputEventHandle for DefaultCameraInputEventHandle {
    fn mouse_motion_handle(
        camera: &mut Camera,
        delta: (f64, f64),
        is_cursor_visible: bool,
        motion_speed: f32,
    ) {
        if is_cursor_visible == false {
            let speed_x = motion_speed as f64;
            let speed_y = motion_speed as f64;
            let yaw: f64 = (delta.0 * speed_x).to_radians();
            let pitch: f64 = (-delta.1 * speed_y).to_radians();
            camera.add_world_rotation_relative(&Rotator {
                yaw: yaw as f32,
                roll: 0.0,
                pitch: pitch as f32,
            });
        }
    }

    fn keyboard_input_handle(
        camera: &mut Camera,
        virtual_key_code: &winit::keyboard::KeyCode,
        element_state: &winit::event::ElementState,
        is_cursor_visible: bool,
        movement_speed: f32,
    ) {
        let speed = movement_speed;
        if virtual_key_code == &winit::keyboard::KeyCode::KeyW
            && element_state == &ElementState::Pressed
            && is_cursor_visible == false
        {
            camera.add_local_location(glam::Vec3 {
                x: 0.0,
                y: 0.0,
                z: 1.0 * speed,
            });
        }
        if virtual_key_code == &winit::keyboard::KeyCode::KeyA
            && element_state == &ElementState::Pressed
            && is_cursor_visible == false
        {
            camera.add_local_location(glam::Vec3 {
                x: -1.0 * speed,
                y: 0.0,
                z: 0.0,
            });
        }
        if virtual_key_code == &winit::keyboard::KeyCode::KeyS
            && element_state == &ElementState::Pressed
            && is_cursor_visible == false
        {
            camera.add_local_location(glam::Vec3 {
                x: 0.0,
                y: 0.0,
                z: -1.0 * speed,
            });
        }
        if virtual_key_code == &winit::keyboard::KeyCode::KeyD
            && element_state == &ElementState::Pressed
            && is_cursor_visible == false
        {
            camera.add_local_location(glam::Vec3 {
                x: 1.0 * speed,
                y: 0.0,
                z: 0.0,
            });
        }
        if virtual_key_code == &winit::keyboard::KeyCode::KeyE
            && element_state == &ElementState::Pressed
            && is_cursor_visible == false
        {
            camera.add_local_location(glam::Vec3 {
                x: 0.0,
                y: 1.0 * speed,
                z: 0.0,
            });
        }
        if virtual_key_code == &winit::keyboard::KeyCode::KeyQ
            && element_state == &ElementState::Pressed
            && is_cursor_visible == false
        {
            camera.add_local_location(glam::Vec3 {
                x: 0.0,
                y: -1.0 * speed,
                z: 0.0,
            });
        }
    }
}
