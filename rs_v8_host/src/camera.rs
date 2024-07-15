#[repr(align(2))]
#[derive(Clone, Copy)]
pub(crate) struct NativeCamera {
    pub(crate) borrow_mut: *mut rs_engine::camera::Camera,
}

impl NativeCamera {
    pub(crate) unsafe fn new(engine: &mut rs_engine::camera::Camera) -> NativeCamera {
        NativeCamera { borrow_mut: engine }
    }
}

impl v8::cppgc::GarbageCollected for NativeCamera {
    fn trace(&self, _visitor: &v8::cppgc::Visitor) {}
}

pub(crate) fn camera_set_window_size(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut _retval: v8::ReturnValue,
) {
    if args.length() < 2 {
        return;
    }
    let arg_0 = args.get(0);
    let arg_1 = args.get(1);
    if !arg_0.is_number() || !arg_1.is_number() {
        return;
    }
    let Some(width) = arg_0.to_uint32(scope).map(|x| x.value()) else {
        return;
    };
    let Some(height) = arg_1.to_uint32(scope).map(|x| x.value()) else {
        return;
    };

    let pointer = unsafe { args.this().get_aligned_pointer_from_internal_field(0) };
    let camera: &mut NativeCamera = unsafe { std::mem::transmute(pointer) };
    let camera: &mut rs_engine::camera::Camera = unsafe { std::mem::transmute(camera.borrow_mut) };

    camera.set_window_size(width, height);
}
