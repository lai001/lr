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
    def_class_name: CString,
    property_function_list: HashMap<CString, JSCFunction>,
    func_entrys: Vec<JSCFunctionListEntry>,
}
unsafe impl Send for AccelerationBakerJSClass {}

impl AccelerationBakerJSClass {
    pub fn default() -> &'static Mutex<AccelerationBakerJSClass> {
        &GLOBAL_ACCELERATION_BAKER_JS_CLASS
    }

    fn new() -> AccelerationBakerJSClass {
        let class_name = "AccelerationBaker".to_string();
        let def_class_name = CString::new(class_name.as_str()).unwrap();

        let mut property_function_list: HashMap<CString, JSCFunction> = HashMap::new();
        property_function_list.insert(
            CString::new("toString").unwrap(),
            Some(Self::AccelerationBakerJSClass_toString),
        );
        let mut func_entrys: Vec<JSCFunctionListEntry> = vec![];
        for (name, func) in &property_function_list {
            let entry = QuickJS::new_function_list_entry(&name, *func);
            func_entrys.push(entry);
        }
        AccelerationBakerJSClass {
            class_id: QuickJS::new_classid(),
            class_def: JSClassDef {
                class_name: def_class_name.as_ptr(),
                finalizer: Some(Self::AccelerationBakerJSClass_finalizer),
                gc_mark: None,
                call: None,
                exotic: std::ptr::null_mut(),
            },
            class_name,
            property_function_list,
            def_class_name,
            func_entrys,
        }
    }

    pub fn import(&self, ctx: &mut QuickJSContext) -> JSValue {
        let rt = ctx.get_runtime();
        rt.borrow_mut().new_class(self.class_id, &self.class_def);
        let proto = ctx.new_object();
        ctx.set_property_function_list(proto, &self.func_entrys);
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
        let jsclass = GLOBAL_ACCELERATION_BAKER_JS_CLASS.lock().unwrap();
        log::trace!("{}", jsclass.class_name.clone() + " ctor");
        let mut object = QuickJS::null();
        {
            let prototype = QuickJS::get_property_str(ctx, this_val, "prototype");
            object = QuickJS::new_object_proto_class(ctx, prototype, jsclass.class_id);
            QuickJS::free_value(ctx, prototype);
        }
        return object;
    }

    extern "C" fn AccelerationBakerJSClass_toString(
        ctx: *mut JSContext,
        this_val: JSValue,
        argc: ::std::os::raw::c_int,
        argv: *mut JSValue,
    ) -> JSValue {
        let jsclass = GLOBAL_ACCELERATION_BAKER_JS_CLASS.lock().unwrap();
        let mut object = QuickJS::null();
        object = QuickJS::new_string(ctx, &jsclass.class_name);
        return object;
    }

    extern "C" fn AccelerationBakerJSClass_finalizer(rt: *mut JSRuntime, this_val: JSValue) {
        let jsclass = GLOBAL_ACCELERATION_BAKER_JS_CLASS.lock().unwrap();
        log::trace!("{}", jsclass.class_name.clone() + " finalizer");
    }
}
