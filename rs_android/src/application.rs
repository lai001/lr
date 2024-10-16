use crate::enviroment::Enviroment;
use crate::error::Result;
use crate::motion_event::{self, EActionType, Geometry};
use rs_artifact::artifact::{ArtifactFileHeader, ArtifactReader};
use rs_artifact::java_input_stream::JavaInputStream;
use rs_artifact::{
    file_header::{FileHeader, ARTIFACT_FILE_MAGIC_NUMBERS},
    EEndianType,
};
use rs_engine::logger::Logger;

const WINDOW_ID: isize = 0;

pub struct Application {
    native_window: crate::native_window::NativeWindow,
    raw_input: egui::RawInput,
    scale_factor: f32,
    enviroment: Option<Enviroment>,
    engine: rs_engine::engine::Engine,
    gui_context: egui::Context,
    geometries: Vec<Geometry>,
    camera: rs_engine::camera::Camera,
}

impl Application {
    pub fn from_native_window(
        native_window: crate::native_window::NativeWindow,
        artifact_input_stream: JavaInputStream,
        logger: Logger,
    ) -> Result<Application> {
        let scale_factor = 1.0f32;

        let raw_input = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::default(),
                egui::vec2(
                    native_window.get_width() as f32,
                    native_window.get_height() as f32,
                ) / scale_factor as f32,
            )),
            ..Default::default()
        };
        let artifact_reader = ArtifactReader::new(artifact_input_stream, Some(EEndianType::Little))
            .map_err(|err| crate::error::Error::Artifact(err))?;

        let width = native_window.get_width();
        let height = native_window.get_height();
        let gui_context = egui::Context::default();
        let mut engine = rs_engine::engine::Engine::new(
            WINDOW_ID,
            &native_window,
            width,
            height,
            scale_factor,
            logger,
            Some(artifact_reader),
            std::collections::HashMap::new(),
        )
        .map_err(|err| crate::error::Error::Engine(err))?;
        engine.init_resources();
        let mut camera = rs_engine::camera::Camera::default(width, height);
        camera.set_world_location(glam::vec3(0.0, 10.0, 20.0));
        Ok(Application {
            native_window,
            raw_input,
            scale_factor,
            enviroment: None,
            engine,
            gui_context,
            geometries: vec![],
            camera,
        })
    }

    pub fn redraw(&mut self) {
        self.engine.window_redraw_requested_begin(WINDOW_ID);
        let context = &self.gui_context;
        context.begin_pass(self.raw_input.clone());

        egui::Window::new("Pannel")
            .default_pos((200.0, 200.0))
            .show(&context, |ui| {
                let response = ui.button("Button");
                if response.clicked() {}
                if ui.button("Button2").clicked() {}
                ui.label(format!("Time: {:.2}", 0.0f32));
            });

        let full_output = context.end_pass();
        let gui_render_output = rs_render::egui_render::EGUIRenderOutput {
            textures_delta: full_output.textures_delta,
            clipped_primitives: context
                .tessellate(full_output.shapes, full_output.pixels_per_point),
            window_id: WINDOW_ID,
        };
        self.engine.tick();
        self.engine.draw_gui(gui_render_output);
        self.engine.window_redraw_requested_end(WINDOW_ID);
    }

    pub fn get_status_bar_height(&self) -> i32 {
        let status_bar_height = {
            if let Some(ref enviroment) = self.enviroment {
                enviroment.status_bar_height
            } else {
                0
            }
        };
        status_bar_height
    }

    pub fn set_new_window(
        &mut self,
        native_window: crate::native_window::NativeWindow,
    ) -> Result<()> {
        let surface_width = native_window.get_width();
        let surface_height = native_window.get_height();
        self.engine
            .set_new_window(
                WINDOW_ID,
                &native_window,
                surface_width,
                surface_height,
                1.0,
            )
            .map_err(|err| crate::error::Error::Engine(err))?;
        self.native_window = native_window;
        Ok(())
    }
}

#[no_mangle]
#[jni_fn::jni_fn("com.lai001.rs_android.Application")]
pub fn Application_fromSurface(
    mut env: jni::JNIEnv,
    _: jni::objects::JClass,
    surface: jni::sys::jobject,
    artifact_input_stream: jni::objects::JObject,
) -> *mut Application {
    debug_assert_ne!(surface, std::ptr::null_mut());
    let logger = rs_engine::logger::Logger::new(rs_engine::logger::LoggerConfiguration {
        is_write_to_file: false,
    });
    let result: crate::error::Result<*mut Application> = (|| {
        let native_window = crate::native_window::NativeWindow::new(&mut env, surface)
            .ok_or(crate::error::Error::NativeWindowNull)?;
        let mut artifact_input_stream = JavaInputStream::new(env, artifact_input_stream)
            .map_err(|_| crate::error::Error::JavaInputStreamNull)?;
        FileHeader::check_identification(&mut artifact_input_stream, ARTIFACT_FILE_MAGIC_NUMBERS)
            .map_err(|err| crate::error::Error::CheckIdentificationFail(err))?;

        let _: ArtifactFileHeader =
            FileHeader::get_header2(&mut artifact_input_stream, Some(EEndianType::Little))
                .map_err(|err| crate::error::Error::Artifact(err))?;
        let application =
            Application::from_native_window(native_window, artifact_input_stream, logger)?;
        Ok(Box::into_raw(Box::new(application)))
    })();
    match result {
        Ok(application) => application,
        Err(err) => {
            log::warn!("{}", err);
            std::ptr::null_mut()
        }
    }
}

#[no_mangle]
#[jni_fn::jni_fn("com.lai001.rs_android.Application")]
pub fn Application_setNewSurface(
    mut env: jni::JNIEnv,
    _: jni::objects::JClass,
    application: *mut Application,
    surface: jni::sys::jobject,
) -> jni::sys::jboolean {
    debug_assert_ne!(application, std::ptr::null_mut());
    debug_assert_ne!(surface, std::ptr::null_mut());
    let native_window = crate::native_window::NativeWindow::new(&mut env, surface);
    if let Some(native_window) = native_window {
        let mut application: Box<Application> = unsafe { Box::from_raw(application) };
        let result = application.set_new_window(native_window);
        Box::into_raw(Box::new(application));
        match result {
            Ok(_) => jni::sys::JNI_TRUE,
            Err(_) => jni::sys::JNI_FALSE,
        }
    } else {
        jni::sys::JNI_FALSE
    }
}

#[no_mangle]
#[jni_fn::jni_fn("com.lai001.rs_android.Application")]
pub fn Application_drop(_: jni::JNIEnv, _: jni::objects::JClass, application: *mut Application) {
    debug_assert_ne!(application, std::ptr::null_mut());
    let _: Box<Application> = unsafe { Box::from_raw(application) };
}

#[no_mangle]
#[jni_fn::jni_fn("com.lai001.rs_android.Application")]
pub fn Application_redraw(_: jni::JNIEnv, _: jni::objects::JClass, application: *mut Application) {
    debug_assert_ne!(application, std::ptr::null_mut());
    let mut application: Box<Application> = unsafe { Box::from_raw(application) };

    for pair in application.geometries.windows(2) {
        let old_geometry = &pair[0];
        let new_geometry = &pair[1];
        let status_bar_height = application.get_status_bar_height();

        if old_geometry.action == EActionType::ActionMove
            && new_geometry.action == EActionType::ActionMove
        {
            let delta_x = new_geometry.x - old_geometry.x;
            let delta_y = new_geometry.y - old_geometry.y;

            let camera = &mut application.camera;
            if new_geometry.x <= (application.native_window.get_width() / 2) as f32 {
                let motion_speed = 0.1;
                camera.add_world_location(glam::vec3(
                    delta_x * motion_speed,
                    0.0,
                    delta_y * motion_speed,
                ));
            } else {
                let motion_speed = 0.1;
                let speed_x = motion_speed as f64;
                let speed_y = motion_speed as f64;
                let yaw: f64 = (delta_x as f64 * speed_x).to_radians();
                let pitch: f64 = (-delta_y as f64 * speed_y).to_radians();
                camera.add_world_rotation_relative(&rs_engine::rotator::Rotator {
                    yaw: yaw as f32,
                    roll: 0.0,
                    pitch: pitch as f32,
                });
            }
        }

        let raw_input = &mut application.raw_input;

        let phase: egui::TouchPhase = {
            match old_geometry.action {
                EActionType::ActionUp => egui::TouchPhase::End,
                EActionType::ActionMove => egui::TouchPhase::Move,
                EActionType::ActionDown => egui::TouchPhase::Start,
                EActionType::ActionCancel => egui::TouchPhase::Cancel,
                EActionType::ActionOutside => egui::TouchPhase::End,
            }
        };
        let pointer_pos = egui::pos2(
            (old_geometry.x) / application.scale_factor,
            (old_geometry.y - status_bar_height as f32) / application.scale_factor,
        );
        match phase {
            egui::TouchPhase::Start => {
                raw_input.events.push(egui::Event::PointerButton {
                    pos: pointer_pos,
                    button: egui::PointerButton::Primary,
                    pressed: true,
                    modifiers: Default::default(),
                });
            }
            egui::TouchPhase::Move => {
                raw_input
                    .events
                    .push(egui::Event::PointerMoved(pointer_pos));
            }
            egui::TouchPhase::End => {
                raw_input.events.push(egui::Event::PointerButton {
                    pos: pointer_pos,
                    button: egui::PointerButton::Primary,
                    pressed: false,
                    modifiers: Default::default(),
                });
                raw_input.events.push(egui::Event::PointerGone);
            }
            egui::TouchPhase::Cancel => {}
        }
    }
    if application.geometries.len() >= 2 {
        application.geometries.clear();
    }

    application.redraw();
    application.raw_input.events.clear();
    Box::into_raw(Box::new(application));
}

#[no_mangle]
#[jni_fn::jni_fn("com.lai001.rs_android.Application")]
pub fn Application_onTouchEvent(
    env: jni::JNIEnv,
    _: jni::objects::JClass,
    application: *mut Application,
    event: jni::objects::JClass,
) -> jni::sys::jboolean {
    debug_assert_ne!(application, std::ptr::null_mut());

    let mut motion_event = motion_event::MotionEvent::new(env, event);
    let mut application: Box<Application> = unsafe { Box::from_raw(application) };
    let new_geometry = motion_event.to_geometry();
    application.geometries.push(new_geometry);

    Box::into_raw(Box::new(application));
    return jni::sys::JNI_TRUE;
}

#[no_mangle]
#[jni_fn::jni_fn("com.lai001.rs_android.Application")]
pub fn Application_surfaceChanged(
    _: jni::JNIEnv,
    _: jni::objects::JClass,
    application: *mut Application,
    _: jni::sys::jint,
    w: jni::sys::jint,
    h: jni::sys::jint,
) {
    debug_assert_ne!(application, std::ptr::null_mut());

    // let format = ndk_sys::AHardwareBuffer_Format::AHARDWAREBUFFER_FORMAT_R8G8B8A8_UNORM.0;
    let mut application: Box<Application> = unsafe { Box::from_raw(application) };
    application.raw_input.screen_rect = Some(egui::Rect::from_min_size(
        egui::Pos2::default(),
        egui::vec2(w as f32, h as f32) / application.scale_factor as f32,
    ));

    application.native_window.set_buffers_geometry(
        w as u32,
        h as u32,
        application.native_window.get_format(),
    );
    Box::into_raw(Box::new(application));
}

#[no_mangle]
#[jni_fn::jni_fn("com.lai001.rs_android.Application")]
pub fn Application_surfaceDestroyed(
    _: jni::JNIEnv,
    _: jni::objects::JClass,
    _: *mut Application,
    _surface: jni::sys::jobject,
) {
}

#[no_mangle]
#[jni_fn::jni_fn("com.lai001.rs_android.Application")]
pub fn Application_setEnvironment(
    mut env: jni::JNIEnv,
    _: jni::objects::JClass,
    application: *mut Application,
    mut android_enviroment: jni::objects::JClass,
) {
    debug_assert_ne!(application, std::ptr::null_mut());

    let mut application: Box<Application> = unsafe { Box::from_raw(application) };
    application.enviroment = Some(Enviroment::new(&mut env, &mut android_enviroment));
    Box::into_raw(Box::new(application));
}
