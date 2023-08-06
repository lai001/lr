use crate::bake_info::BakeInfo;
use rs_quickjs::{quick_js_context::QuickJSContext, quickjs_bindings::*};
use std::{collections::HashMap, ffi::CString, sync::Mutex};

lazy_static! {
    static ref GLOBAL_BAKE_INFO_JSCLASS: Mutex<BakeInfoJSClass> =
        Mutex::new(BakeInfoJSClass::new());
}

pub struct BakeInfoJSClass {
    class_id: JSClassID,
    class_def: JSClassDef,
    class_name: String,
    def_class_name: CString,
    property_function_list: HashMap<CString, JSCFunction>,
    func_entrys: Vec<JSCFunctionListEntry>,
}
unsafe impl Send for BakeInfoJSClass {}

impl BakeInfoJSClass {
    pub fn default() -> &'static Mutex<BakeInfoJSClass> {
        &GLOBAL_BAKE_INFO_JSCLASS
    }

    fn new() -> BakeInfoJSClass {
        let class_name = "BakeInfo".to_string();
        let def_class_name = CString::new(class_name.as_str()).unwrap();

        let mut property_function_list: HashMap<CString, JSCFunction> = HashMap::new();
        property_function_list.insert(
            CString::new("toString").unwrap(),
            Some(Self::BakeInfo_toString),
        );
        let mut func_entrys: Vec<JSCFunctionListEntry> = vec![];
        for (name, func) in &property_function_list {
            let entry = QuickJS::new_function_list_entry(&name, *func);
            func_entrys.push(entry);
        }
        BakeInfoJSClass {
            class_id: QuickJS::new_classid(),
            class_def: JSClassDef {
                class_name: def_class_name.as_ptr(),
                finalizer: Some(Self::BakeInfo_finalizer),
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
        let constructor_class_func =
            ctx.new_constructor_function(Some(Self::BakeInfo_ctor), &self.class_name);
        ctx.set_constructor(constructor_class_func, proto);
        ctx.set_class_proto(self.class_id, proto);
        return constructor_class_func;
    }

    pub fn get_class_name(&self) -> &str {
        self.class_name.as_ref()
    }
}

fn create_bake_info(
    ctx: *mut JSContext,
    argc: ::std::os::raw::c_int,
    argv: *mut JSValue,
) -> Box<BakeInfo> {
    let argv = unsafe { std::slice::from_raw_parts_mut(argv, argc as usize) };
    assert_eq!(argv.len(), 12);

    assert!(QuickJS::is_bool(argv[0]));
    assert!(QuickJS::is_bool(argv[1]));
    assert!(QuickJS::is_bool(argv[2]));
    assert!(QuickJS::is_bool(argv[3]));
    assert!(QuickJS::is_number(argv[4]));
    assert!(QuickJS::is_number(argv[5]));
    assert!(QuickJS::is_number(argv[6]));
    assert!(QuickJS::is_number(argv[7]));
    assert!(QuickJS::is_number(argv[8]));
    assert!(QuickJS::is_number(argv[9]));
    assert!(QuickJS::is_number(argv[10]));
    assert!(QuickJS::is_number(argv[11]));

    let is_bake_environment: bool = QuickJS::to_bool(ctx, argv[0]);
    let is_bake_irradiance: bool = QuickJS::to_bool(ctx, argv[1]);
    let is_bake_brdflut: bool = QuickJS::to_bool(ctx, argv[2]);
    let is_bake_pre_filter: bool = QuickJS::to_bool(ctx, argv[3]);
    let environment_cube_map_length: u32 = QuickJS::to_uint32(ctx, argv[4]);
    let irradiance_cube_map_length: u32 = QuickJS::to_uint32(ctx, argv[5]);
    let irradiance_sample_count: u32 = QuickJS::to_uint32(ctx, argv[6]);
    let pre_filter_cube_map_length: u32 = QuickJS::to_uint32(ctx, argv[7]);
    let pre_filter_cube_map_max_mipmap_level: u32 = QuickJS::to_uint32(ctx, argv[8]);
    let pre_filter_sample_count: u32 = QuickJS::to_uint32(ctx, argv[9]);
    let brdflutmap_length: u32 = QuickJS::to_uint32(ctx, argv[10]);
    let brdf_sample_count: u32 = QuickJS::to_uint32(ctx, argv[11]);
    let bake_info = BakeInfo {
        is_bake_environment,
        is_bake_irradiance,
        is_bake_brdflut,
        is_bake_pre_filter,
        environment_cube_map_length,
        irradiance_cube_map_length,
        irradiance_sample_count,
        pre_filter_cube_map_length,
        pre_filter_cube_map_max_mipmap_level,
        pre_filter_sample_count,
        brdflutmap_length,
        brdf_sample_count,
    };
    let bake_info = Box::new(bake_info);
    bake_info
}

impl BakeInfoJSClass {
    extern "C" fn BakeInfo_ctor(
        ctx: *mut JSContext,
        this_val: JSValue,
        argc: ::std::os::raw::c_int,
        argv: *mut JSValue,
    ) -> JSValue {
        let jsclass = GLOBAL_BAKE_INFO_JSCLASS.lock().unwrap();
        log::trace!("{}", jsclass.class_name.clone() + " ctor");

        let mut object = QuickJS::null();
        {
            let prototype = QuickJS::get_property_str(ctx, this_val, "prototype");
            object = QuickJS::new_object_proto_class(ctx, prototype, jsclass.class_id);
            let bake_info = Box::into_raw(create_bake_info(ctx, argc, argv));
            QuickJS::set_opaque(object, bake_info);
            QuickJS::free_value(ctx, prototype);
        }
        return object;
    }

    extern "C" fn BakeInfo_toString(
        ctx: *mut JSContext,
        this_val: JSValue,
        argc: ::std::os::raw::c_int,
        argv: *mut JSValue,
    ) -> JSValue {
        let jsclass = GLOBAL_BAKE_INFO_JSCLASS.lock().unwrap();
        let mut object = QuickJS::null();
        if QuickJS::is_object(this_val) {
            let bake_info: *mut BakeInfo = QuickJS::get_opaque(this_val, jsclass.class_id);
            if bake_info != std::ptr::null_mut() {
                let bake_info = unsafe { *bake_info };
                let message = format!("{:?}", bake_info);
                object = QuickJS::new_string(ctx, &message);
            }
        }
        return object;
    }

    extern "C" fn BakeInfo_finalizer(rt: *mut JSRuntime, this_val: JSValue) {
        let jsclass = GLOBAL_BAKE_INFO_JSCLASS.lock().unwrap();
        log::trace!("{}", jsclass.class_name.clone() + " finalizer");
        if QuickJS::is_object(this_val) {
            let bake_info: *mut BakeInfo = QuickJS::get_opaque(this_val, jsclass.class_id);
            if bake_info != std::ptr::null_mut() {
                unsafe { Box::from_raw(bake_info) };
            }
        }
    }
}
