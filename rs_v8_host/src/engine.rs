use rs_render::view_mode::EViewModeType;

#[repr(align(2))]
#[derive(Clone, Copy)]
pub(crate) struct NativeEngine {
    pub(crate) borrow_mut: *mut rs_engine::engine::Engine,
}

impl NativeEngine {
    pub(crate) unsafe fn new(engine: &mut rs_engine::engine::Engine) -> NativeEngine {
        NativeEngine { borrow_mut: engine }
    }
}

impl v8::cppgc::GarbageCollected for NativeEngine {
    fn trace(&self, _visitor: &v8::cppgc::Visitor) {}
}

pub(crate) fn engine_set_view_mode(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut _retval: v8::ReturnValue,
) {
    if args.length() < 1 {
        return;
    }
    let arg_0 = args.get(0);
    if !arg_0.is_number() {
        return;
    }
    let Some(mode) = arg_0.to_int32(scope).map(|x| x.value()) else {
        return;
    };

    let pointer = unsafe { args.this().get_aligned_pointer_from_internal_field(0) };
    let engine: &mut NativeEngine = unsafe { std::mem::transmute(pointer) };
    let engine: &mut rs_engine::engine::Engine = unsafe { std::mem::transmute(engine.borrow_mut) };

    let mode = match mode {
        0 => EViewModeType::Wireframe,
        1 => EViewModeType::Lit,
        2 => EViewModeType::Unlit,
        _ => EViewModeType::Lit,
    };
    engine.set_view_mode(mode);
}

// pub(crate) fn engine_get_camera_mut(
//     scope: &mut v8::HandleScope,
//     args: v8::FunctionCallbackArguments,
//     mut retval: v8::ReturnValue,
// ) {
//     let pointer = unsafe { args.this().get_aligned_pointer_from_internal_field(0) };
//     let engine: &mut NativeEngine = unsafe { std::mem::transmute(pointer) };
//     let engine: &mut rs_engine::engine::Engine = unsafe { std::mem::transmute(engine.borrow_mut) };
//     let native_camera = unsafe { NativeCamera::new(engine.get_camera_mut()) };
//     let native_camera = Box::new(native_camera);
//     let native_camera = Box::into_raw(native_camera);

//     let camera_object: crate::error::Result<Local<Object>> = (|| {
//         let camera_object_template = v8::ObjectTemplate::new(scope);
//         camera_object_template.set_internal_field_count(1);

//         let name = v8::String::new(scope, "setWindowSize").ok_or(crate::error::Error::Null(
//             format!("Failed to create string"),
//         ))?;
//         let function = v8::FunctionTemplate::new(scope, camera_set_window_size);
//         camera_object_template.set(name.into(), function.into());

//         let camera_object =
//             camera_object_template
//                 .new_instance(scope)
//                 .ok_or(crate::error::Error::Other(format!(
//                     "Failed to create object"
//                 )))?;

//         camera_object.set_aligned_pointer_in_internal_field(0, native_camera as _);
//         Ok(camera_object)
//     })();

//     match camera_object {
//         Ok(camera_object) => {
//             retval.set(camera_object.into());
//         }
//         Err(_) => retval.set_null(),
//     }
// }
