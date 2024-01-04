use crate::enviroment::Enviroment;
use crate::motion_event;
use rs_artifact::artifact::{ArtifactFileHeader, ArtifactReader};
use rs_artifact::java_input_stream::JavaInputStream;
use rs_artifact::{
    file_header::{FileHeader, ARTIFACT_FILE_MAGIC_NUMBERS},
    EEndianType,
};

pub struct Application {
    native_window: crate::native_window::NativeWindow,
    raw_input: egui::RawInput,
    scale_factor: f32,
    enviroment: Option<Enviroment>,
    engine: rs_engine::engine::Engine,
}

impl Application {
    pub fn from_native_window(
        native_window: crate::native_window::NativeWindow,
        artifact_input_stream: JavaInputStream,
    ) -> Option<Application> {
        let scale_factor = 1.0f32;

        let raw_input = egui::RawInput {
            pixels_per_point: Some(scale_factor as f32),
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::default(),
                egui::vec2(
                    native_window.get_width() as f32,
                    native_window.get_height() as f32,
                ) / scale_factor as f32,
            )),
            ..Default::default()
        };
        let artifact_reader =
            match ArtifactReader::new(artifact_input_stream, Some(EEndianType::Little)) {
                Ok(artifact_reader) => artifact_reader,
                Err(err) => {
                    log::warn!("{err:?}");
                    return None;
                }
            };

        let width = native_window.get_width();
        let height = native_window.get_height();
        let engine = match rs_engine::engine::Engine::new(
            &native_window,
            width,
            height,
            scale_factor,
            Some(artifact_reader),
        ) {
            Ok(engine) => engine,
            Err(err) => {
                log::warn!("{err:?}");
                return None;
            }
        };
        Some(Application {
            native_window,
            raw_input,
            scale_factor,
            enviroment: None,
            engine,
        })
    }

    pub fn redraw(&mut self) {
        self.engine.redraw(&self.raw_input);
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

    pub fn set_new_window(&mut self, native_window: &crate::native_window::NativeWindow) -> bool {
        let surface_width = native_window.get_width();
        let surface_height = native_window.get_height();
        let result = self
            .engine
            .set_new_window(native_window, surface_width, surface_height);
        match result {
            Ok(_) => true,
            Err(err) => {
                log::warn!("{err:?}");
                false
            }
        }
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
    let native_window = crate::native_window::NativeWindow::new(&mut env, surface);
    if let (Some(native_window), Some(mut artifact_input_stream)) = (
        native_window,
        JavaInputStream::new(env, artifact_input_stream),
    ) {
        if let Err(err) = FileHeader::check_identification(
            &mut artifact_input_stream,
            ARTIFACT_FILE_MAGIC_NUMBERS,
        ) {
            log::warn!("{err:?}");
            return std::ptr::null_mut();
        }
        let header: ArtifactFileHeader =
            match FileHeader::get_header2(&mut artifact_input_stream, Some(EEndianType::Little)) {
                Ok(header) => header,
                Err(err) => {
                    log::warn!("{err:?}");
                    return std::ptr::null_mut();
                }
            };
        let application = Application::from_native_window(native_window, artifact_input_stream);
        if let Some(application) = application {
            return Box::into_raw(Box::new(application));
        }
    }
    return std::ptr::null_mut();
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
        application.set_new_window(&native_window);
        Box::into_raw(Box::new(application));
        jni::sys::JNI_TRUE
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
    application.redraw();
    application.raw_input.events.clear();
    Box::into_raw(Box::new(application));
}

#[no_mangle]
#[jni_fn::jni_fn("com.lai001.rs_android.Application")]
pub fn Application_onTouchEvent(
    mut env: jni::JNIEnv,
    _: jni::objects::JClass,
    application: *mut Application,
    event: jni::objects::JClass,
) -> jni::sys::jboolean {
    debug_assert_ne!(application, std::ptr::null_mut());

    let mut motion_event = motion_event::MotionEvent::new(env, event);
    let mut application: Box<Application> = unsafe { Box::from_raw(application) };
    let status_bar_height = application.get_status_bar_height();

    let raw_input = &mut application.raw_input;

    let phase: egui::TouchPhase = {
        if motion_event.get_action() == 3 {
            egui::TouchPhase::Cancel
        } else if motion_event.get_action() == 0 {
            egui::TouchPhase::Start
        } else if motion_event.get_action() == 2 {
            egui::TouchPhase::Move
        } else if motion_event.get_action() == 1 {
            egui::TouchPhase::End
        } else {
            egui::TouchPhase::End
        }
    };
    let pointer_pos = egui::pos2(
        (motion_event.get_x() as f32) / application.scale_factor,
        (motion_event.get_y() as f32 - status_bar_height as f32) / application.scale_factor,
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
    application: *mut Application,
    surface: jni::sys::jobject,
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
