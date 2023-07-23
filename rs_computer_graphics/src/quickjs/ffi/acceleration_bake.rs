use rs_quickjs::{quick_js_context::QuickJSContext, quickjs_bindings::*};
use std::{collections::HashMap, ffi::CString, sync::Mutex};

lazy_static! {
    static ref GLOBAL_ACCELERATION_BAKER_JS_CLASS: Mutex<AccelerationBakerJSClass> =
        Mutex::new(AccelerationBakerJSClass::new());
}

pub struct AccelerationBakerJSClass {
    class_id: JSClassID,
    class_def: JSClassDef,
    class_name: String,
    property_function_list: HashMap<CString, JSCFunction>,
}
unsafe impl Send for AccelerationBakerJSClass {}

impl AccelerationBakerJSClass {
    pub fn default() -> &'static Mutex<AccelerationBakerJSClass> {
        &GLOBAL_ACCELERATION_BAKER_JS_CLASS
    }

    fn new() -> AccelerationBakerJSClass {
        let class_name = "AccelerationBaker".to_string();
        let c_str = CString::new(class_name.as_str()).unwrap();

        let mut property_function_list: HashMap<CString, JSCFunction> = HashMap::new();
        property_function_list.insert(
            CString::new("toString").unwrap(),
            Some(Self::AccelerationBakerJSClass_toString),
        );

        AccelerationBakerJSClass {
            class_id: QuickJS::new_classid(),
            class_def: JSClassDef {
                class_name: c_str.as_ptr(),
                finalizer: Some(Self::AccelerationBakerJSClass_finalizer),
                gc_mark: None,
                call: None,
                exotic: std::ptr::null_mut(),
            },
            class_name: class_name,
            property_function_list,
        }
    }

    pub fn import(&self, ctx: &mut QuickJSContext) -> JSValue {
        let rt = ctx.get_runtime();
        rt.borrow_mut().new_class(self.class_id, &self.class_def);
        let proto = ctx.new_object();
        let mut func_entrys: Vec<JSCFunctionListEntry> = vec![];
        for (name, func) in &self.property_function_list {
            let entry = QuickJS::new_function_list_entry(&name, *func);
            func_entrys.push(entry);
        }
        ctx.set_property_function_list(proto, &func_entrys);
        let constructor_class_func = ctx
            .new_constructor_function(Some(Self::AccelerationBakerJSClass_ctor), &self.class_name);
        ctx.set_constructor(constructor_class_func, proto);
        ctx.set_class_proto(self.class_id, proto);
        return constructor_class_func;
    }

    pub fn get_class_name(&self) -> &str {
        self.class_name.as_ref()
    }
}

impl AccelerationBakerJSClass {
    extern "C" fn AccelerationBakerJSClass_ctor(
        ctx: *mut JSContext,
        this_val: JSValue,
        argc: ::std::os::raw::c_int,
        argv: *mut JSValue,
    ) -> JSValue {
        log::trace!(
            "{}",
            GLOBAL_ACCELERATION_BAKER_JS_CLASS
                .lock()
                .unwrap()
                .class_name
                .clone()
                + " ctor"
        );
        let mut object = QuickJS::null();
        QuickJS::get_property_str(ctx, this_val, "prototype", |ctx, proto| {
            object = QuickJS::new_object_proto_class(
                ctx,
                proto,
                GLOBAL_ACCELERATION_BAKER_JS_CLASS.lock().unwrap().class_id,
            );
        });
        return object;
    }

    extern "C" fn AccelerationBakerJSClass_toString(
        ctx: *mut JSContext,
        this_val: JSValue,
        argc: ::std::os::raw::c_int,
        argv: *mut JSValue,
    ) -> JSValue {
        let mut object = QuickJS::null();
        object = QuickJS::new_string(
            ctx,
            &GLOBAL_ACCELERATION_BAKER_JS_CLASS
                .lock()
                .unwrap()
                .class_name,
        );
        return object;
    }

    extern "C" fn AccelerationBakerJSClass_finalizer(rt: *mut JSRuntime, this_val: JSValue) {
        log::trace!(
            "{}",
            GLOBAL_ACCELERATION_BAKER_JS_CLASS
                .lock()
                .unwrap()
                .class_name
                .clone()
                + " finalizer"
        );
    }
}
