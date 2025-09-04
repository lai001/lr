use crate::enviroment::Enviroment;
use crate::error::Result;
use crate::gui::GUI;
use crate::key_event::{to_element_state, to_key_code};
use crate::motion_event::{self, MotionEvent};
use rs_artifact::artifact::{ArtifactFileHeader, ArtifactReader};
use rs_artifact::java_input_stream::JavaInputStream;
use rs_artifact::{
    file_header::{FileHeader, ARTIFACT_FILE_MAGIC_NUMBERS},
    EEndianType,
};
use rs_engine::frame_sync::FrameSync;
use rs_engine::input_mode::EInputMode;
use rs_engine::keys_detector::KeysDetector;
use rs_engine::logger::{Logger, SlotFlags};
use rs_render::command::ResizeInfo;

const WINDOW_ID: isize = 0;

pub struct ApplicationContext {
    native_window: crate::native_window::NativeWindow,
    enviroment: Option<Enviroment>,
    engine: rs_engine::engine::Engine,
    gui: GUI,
    app: rs_engine::standalone::application::Application,
    is_window_available: bool,
    frame_sync: FrameSync,
    keys_detector: KeysDetector,
}

impl ApplicationContext {
    pub fn from_native_window(
        native_window: crate::native_window::NativeWindow,
        scale_factor: f32,
        artifact_input_stream: JavaInputStream,
        logger: Logger,
    ) -> Result<ApplicationContext> {
        let width = native_window.get_width();
        let height = native_window.get_height();

        let gui = GUI::new(scale_factor, width, height, 0);

        let artifact_reader = ArtifactReader::new(artifact_input_stream, Some(EEndianType::Little))
            .map_err(|err| crate::error::Error::Artifact(err))?;

        let mut engine = rs_engine::engine::Engine::new(
            WINDOW_ID,
            &native_window,
            width,
            height,
            scale_factor,
            logger,
            Some(artifact_reader),
            std::collections::HashMap::new(),
            std::collections::HashMap::new(),
        )
        .map_err(|err| crate::error::Error::Engine(err))?;
        engine.init_resources();
        let current_active_level = engine.new_main_level().unwrap();
        let contents = engine
            .content_files
            .iter()
            .map(|(_, x)| x.clone())
            .collect();
        #[cfg(feature = "plugin_shared_crate")]
        let plugins = rs_proc_macros::load_static_plugins!(rs_android);
        let app = rs_engine::standalone::application::Application::new(
            WINDOW_ID,
            width,
            height,
            &mut engine,
            &current_active_level,
            contents,
            EInputMode::Game,
            #[cfg(feature = "plugin_shared_crate")]
            plugins,
        );
        let sync = FrameSync::new(rs_engine::frame_sync::EOptions::FPS(60.0));
        let keys_detector = KeysDetector::new();
        Ok(ApplicationContext {
            native_window,
            enviroment: None,
            engine,
            app,
            gui,
            is_window_available: true,
            frame_sync: sync,
            keys_detector,
        })
    }

    pub fn redraw(&mut self) {
        if !self.is_window_available {
            return;
        }
        self.engine.window_redraw_requested_begin(WINDOW_ID);
        self.gui.begin_ui();
        egui::TopBottomPanel::top("my_top_panel")
            .exact_height(0.01)
            .show(&self.gui.egui_context(), |ui| {
                let _ = ui;
            });
        self.app
            .on_redraw_requested(&mut self.engine, self.gui.egui_context().clone());
        let gui_render_output = self.gui.end_ui(WINDOW_ID);
        self.engine.tick();
        self.engine.draw_gui(gui_render_output);
        self.engine.window_redraw_requested_end(WINDOW_ID);
        self.frame_sync.sync();
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
                self.gui.scale_factor(),
            )
            .map_err(|err| crate::error::Error::Engine(err))?;
        self.native_window = native_window;
        self.is_window_available = true;
        Ok(())
    }

    pub fn window_destroyed(&mut self) {
        self.is_window_available = false;
        self.engine.remove_window(WINDOW_ID);
    }

    pub fn window_reszied(&mut self, w: i32, h: i32) {
        if !self.is_window_available {
            return;
        }
        self.engine
            .send_render_command(rs_render::command::RenderCommand::Resize(ResizeInfo {
                window_id: WINDOW_ID,
                width: w as u32,
                height: h as u32,
            }));
        self.gui.on_size_changed(w as u32, h as u32);
        self.native_window.set_buffers_geometry(
            w as u32,
            h as u32,
            self.native_window.get_format(),
        );
    }

    pub fn on_touch(&mut self, motion_event: MotionEvent<'_>) {
        self.gui.on_touch(motion_event);
    }

    fn on_key_up(&mut self, key_code: i32, key_event: &mut crate::key_event::KeyEvent) {
        let Some(key_code) = to_key_code(key_code) else {
            return;
        };
        let Some(element_state) = to_element_state(key_event.get_action()) else {
            return;
        };
        self.keys_detector.on_key(key_code, element_state);
        self.app
            .on_window_input(rs_engine::input_type::EInputType::KeyboardInput(
                self.keys_detector.virtual_key_code_states(),
            ));
    }

    fn on_key_down(&mut self, key_code: i32, key_event: &mut crate::key_event::KeyEvent) {
        let Some(key_code) = to_key_code(key_code) else {
            return;
        };
        let Some(element_state) = to_element_state(key_event.get_action()) else {
            return;
        };
        self.keys_detector.on_key(key_code, element_state);
        self.app
            .on_window_input(rs_engine::input_type::EInputType::KeyboardInput(
                self.keys_detector.virtual_key_code_states(),
            ));
    }
}

#[jni_fn::jni_fn("com.lai001.lib.lrjni.Application")]
pub fn fromSurface(
    mut env: jni::JNIEnv,
    _: jni::objects::JClass,
    surface: jni::sys::jobject,
    scale_factor: f32,
    artifact_input_stream: jni::objects::JObject,
) -> *mut ApplicationContext {
    debug_assert_ne!(surface, std::ptr::null_mut());
    let logger = rs_engine::logger::Logger::new(rs_engine::logger::LoggerConfiguration {
        is_write_to_file: false,
        is_flush_before_drop: false,
        slot_flags: SlotFlags::empty(),
    });
    let result: crate::error::Result<*mut ApplicationContext> = (|| {
        let native_window = crate::native_window::NativeWindow::new(&mut env, surface)
            .ok_or(crate::error::Error::NativeWindowNull)?;
        let mut artifact_input_stream = JavaInputStream::new(env, artifact_input_stream)
            .map_err(|_| crate::error::Error::JavaInputStreamNull)?;
        FileHeader::check_identification(&mut artifact_input_stream, ARTIFACT_FILE_MAGIC_NUMBERS)
            .map_err(|err| crate::error::Error::CheckIdentificationFail(err))?;

        let _: ArtifactFileHeader =
            FileHeader::get_header2(&mut artifact_input_stream, Some(EEndianType::Little))
                .map_err(|err| crate::error::Error::Artifact(err))?;
        let application = ApplicationContext::from_native_window(
            native_window,
            scale_factor,
            artifact_input_stream,
            logger,
        )?;
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
#[jni_fn::jni_fn("com.lai001.lib.lrjni.Application")]
pub fn setNewSurface(
    mut env: jni::JNIEnv,
    _: jni::objects::JClass,
    application: *mut ApplicationContext,
    surface: jni::sys::jobject,
) -> jni::sys::jboolean {
    debug_assert_ne!(application, std::ptr::null_mut());
    debug_assert_ne!(surface, std::ptr::null_mut());
    let native_window = crate::native_window::NativeWindow::new(&mut env, surface);
    if let Some(native_window) = native_window {
        let application = unsafe {
            (application as *mut ApplicationContext)
                .as_mut()
                .expect("A valid pointer")
        };
        let result = application.set_new_window(native_window);
        match result {
            Ok(_) => jni::sys::JNI_TRUE,
            Err(_) => jni::sys::JNI_FALSE,
        }
    } else {
        jni::sys::JNI_FALSE
    }
}

#[no_mangle]
#[jni_fn::jni_fn("com.lai001.lib.lrjni.Application")]
pub fn drop(_: jni::JNIEnv, _: jni::objects::JClass, application: *mut ApplicationContext) {
    debug_assert_ne!(application, std::ptr::null_mut());
    let _: Box<ApplicationContext> = unsafe { Box::from_raw(application) };
}

#[no_mangle]
#[jni_fn::jni_fn("com.lai001.lib.lrjni.Application")]
pub fn redraw(_: jni::JNIEnv, _: jni::objects::JClass, application: *mut ApplicationContext) {
    let application = unsafe {
        (application as *mut ApplicationContext)
            .as_mut()
            .expect("A valid pointer")
    };
    application.redraw();
}

#[no_mangle]
#[jni_fn::jni_fn("com.lai001.lib.lrjni.Application")]
pub fn onTouchEvent(
    env: jni::JNIEnv,
    _: jni::objects::JClass,
    application: *mut ApplicationContext,
    event: jni::objects::JObject,
) -> jni::sys::jboolean {
    debug_assert_ne!(application, std::ptr::null_mut());
    let motion_event = motion_event::MotionEvent::new(env, event);
    let application = unsafe {
        (application as *mut ApplicationContext)
            .as_mut()
            .expect("A valid pointer")
    };
    application.on_touch(motion_event);
    return jni::sys::JNI_TRUE;
}

#[no_mangle]
#[jni_fn::jni_fn("com.lai001.lib.lrjni.Application")]
pub fn surfaceChanged(
    _: jni::JNIEnv,
    _: jni::objects::JClass,
    application: *mut ApplicationContext,
    _: jni::sys::jint,
    w: jni::sys::jint,
    h: jni::sys::jint,
) {
    debug_assert_ne!(application, std::ptr::null_mut());

    // let format = ndk_sys::AHardwareBuffer_Format::AHARDWAREBUFFER_FORMAT_R8G8B8A8_UNORM.0;
    let application = unsafe {
        (application as *mut ApplicationContext)
            .as_mut()
            .expect("A valid pointer")
    };
    application.window_reszied(w, h);
}

#[no_mangle]
#[jni_fn::jni_fn("com.lai001.lib.lrjni.Application")]
pub fn surfaceDestroyed(
    _: jni::JNIEnv,
    _: jni::objects::JClass,
    application: *mut ApplicationContext,
    _surface: jni::sys::jobject,
) {
    let application = unsafe {
        (application as *mut ApplicationContext)
            .as_mut()
            .expect("A valid pointer")
    };
    application.window_destroyed();
}

#[no_mangle]
#[jni_fn::jni_fn("com.lai001.lib.lrjni.Application")]
pub fn setEnvironment(
    _: jni::JNIEnv,
    _: jni::objects::JClass,
    application: jni::sys::jlong,
    enviroment: jni::sys::jlong,
) {
    let enviroment = unsafe {
        (enviroment as *mut Enviroment)
            .as_mut()
            .expect("A valid pointer")
    };
    let application = unsafe {
        (application as *mut ApplicationContext)
            .as_mut()
            .expect("A valid pointer")
    };
    application.enviroment = Some(Enviroment {
        status_bar_height: enviroment.status_bar_height,
    });
    application
        .gui
        .set_status_bar_height(enviroment.status_bar_height);
}

#[jni_fn::jni_fn("com.lai001.lib.lrjni.Application")]
pub fn onKeyDown(
    env: jni::JNIEnv,
    _: jni::objects::JClass,
    application: jni::sys::jlong,
    key_code: jni::sys::jint,
    key_event: jni::objects::JObject,
) -> jni::sys::jboolean {
    let application = unsafe {
        (application as *mut ApplicationContext)
            .as_mut()
            .expect("A valid pointer")
    };
    application.on_key_down(
        key_code,
        &mut crate::key_event::KeyEvent::new(env, key_event),
    );
    return jni::sys::JNI_TRUE;
}

#[jni_fn::jni_fn("com.lai001.lib.lrjni.Application")]
pub fn onKeyUp(
    env: jni::JNIEnv,
    _: jni::objects::JClass,
    application: jni::sys::jlong,
    key_code: jni::sys::jint,
    key_event: jni::objects::JObject,
) -> jni::sys::jboolean {
    let application = unsafe {
        (application as *mut ApplicationContext)
            .as_mut()
            .expect("A valid pointer")
    };
    application.on_key_up(
        key_code,
        &mut crate::key_event::KeyEvent::new(env, key_event),
    );
    return jni::sys::JNI_TRUE;
}
