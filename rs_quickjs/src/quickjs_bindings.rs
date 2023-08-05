#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use bitflags::bitflags;
use std::ffi::{CStr, CString};

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

#[repr(i32)]
#[derive(Debug, Copy, Clone)]
pub enum EJSCFunctionType {
    Generic = JSCFunctionEnum_JS_CFUNC_generic,
    GenericMagic = JSCFunctionEnum_JS_CFUNC_generic_magic,
    Constructor = JSCFunctionEnum_JS_CFUNC_constructor,
    ConstructorMagic = JSCFunctionEnum_JS_CFUNC_constructor_magic,
    ConstructorOrFunc = JSCFunctionEnum_JS_CFUNC_constructor_or_func,
    ConstructorOrFuncMagic = JSCFunctionEnum_JS_CFUNC_constructor_or_func_magic,
    FF = JSCFunctionEnum_JS_CFUNC_f_f,
    FFF = JSCFunctionEnum_JS_CFUNC_f_f_f,
    Getter = JSCFunctionEnum_JS_CFUNC_getter,
    Setter = JSCFunctionEnum_JS_CFUNC_setter,
    GetterMagic = JSCFunctionEnum_JS_CFUNC_getter_magic,
    SetterMagic = JSCFunctionEnum_JS_CFUNC_setter_magic,
    IteratorNext = JSCFunctionEnum_JS_CFUNC_iterator_next,
}

bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    struct JsPropflags: u32 {
        const Configurable = JS_PROP_CONFIGURABLE;
        const Writable = JS_PROP_WRITABLE;
        const Enumerable = JS_PROP_ENUMERABLE;
        const CWE = JS_PROP_C_W_E;
        const Length = JS_PROP_LENGTH;
        const Tmask = JS_PROP_TMASK;
        const Normal = JS_PROP_NORMAL;
        const Getset = JS_PROP_GETSET;
        const Varref = JS_PROP_VARREF;
        const Autoinit = JS_PROP_AUTOINIT;
        const HasShift = JS_PROP_HAS_SHIFT;
        const HasConfigurable = JS_PROP_HAS_CONFIGURABLE;
        const HasWritable = JS_PROP_HAS_WRITABLE;
        const HasEnumerable = JS_PROP_HAS_ENUMERABLE;
        const HasGet = JS_PROP_HAS_GET;
        const HasSet = JS_PROP_HAS_SET;
        const HasValue = JS_PROP_HAS_VALUE;
        const Throw = JS_PROP_THROW;
        const ThrowStrict = JS_PROP_THROW_STRICT;
        const NoAdd = JS_PROP_NO_ADD;
        const NoExotic = JS_PROP_NO_EXOTIC;
    }
}

pub struct QuickJS {}

impl QuickJS {
    pub fn new_classid() -> JSClassID {
        let mut id: JSClassID = 0;
        unsafe { JS_NewClassID(&mut id) };
        id
    }

    pub fn null() -> JSValue {
        unsafe { QuickJS_NULL() }
    }

    pub fn undefined() -> JSValue {
        unsafe { QuickJS_UNDEFINED() }
    }

    pub fn r#false() -> JSValue {
        unsafe { QuickJS_FALSE() }
    }

    pub fn r#true() -> JSValue {
        unsafe { QuickJS_TRUE() }
    }

    pub fn exception() -> JSValue {
        unsafe { QuickJS_EXCEPTION() }
    }

    pub fn uninitialized() -> JSValue {
        unsafe { QuickJS_UNINITIALIZED() }
    }

    pub fn is_object(value: JSValue) -> bool {
        unsafe { QuickJS_IsObject(value) != 0 }
    }

    pub fn is_bool(value: JSValue) -> bool {
        unsafe { QuickJS_IsBool(value) != 0 }
    }

    pub fn is_big_float(value: JSValue) -> bool {
        unsafe { QuickJS_IsBigFloat(value) != 0 }
    }

    pub fn is_number(value: JSValue) -> bool {
        unsafe { QuickJS_IsNumber(value) != 0 }
    }

    pub fn to_bool(ctx: *mut JSContext, value: JSValue) -> bool {
        unsafe { JS_ToBool(ctx, value) != 0 }
    }

    pub fn to_uint32(ctx: *mut JSContext, value: JSValue) -> u32 {
        let mut number = 0_u32;
        unsafe {
            QuickJS_ToUint32(ctx, &mut number, value);
        }
        number
    }

    pub fn is_null(val: JSValue) -> bool {
        unsafe { QuickJS_IsNull(val) != 0 }
    }

    pub fn is_undefined(val: JSValue) -> bool {
        unsafe { QuickJS_IsUndefined(val) != 0 }
    }

    pub fn is_uninitialized(val: JSValue) -> bool {
        unsafe { QuickJS_IsUninitialized(val) != 0 }
    }

    pub fn is_exception(val: JSValue) -> bool {
        unsafe { QuickJS_IsException(val) != 0 }
    }

    pub fn new_string(ctx: *mut JSContext, string: &str) -> JSValue {
        let c_str = CString::new(string).unwrap();
        let value = unsafe { JS_NewString(ctx, c_str.as_ptr()) };
        value
    }

    pub fn new_float64(ctx: *mut JSContext, val: f64) -> JSValue {
        let value = unsafe { QuickJS_NewFloat64(ctx, val) };
        value
    }

    pub fn to_int64(ctx: *mut JSContext, val: JSValue) -> i64 {
        unsafe {
            let mut out_value: i64 = 0;
            let state = JS_ToInt64(ctx, &mut out_value, val);
            assert_eq!(state, 0);
            out_value
        }
    }

    pub fn to_c_string_len2(
        ctx: *mut JSContext,
        val1: JSValue,
        cesu8: ::std::os::raw::c_int,
    ) -> String {
        unsafe {
            let mut plen: usize = 0;
            let str = JS_ToCStringLen2(ctx, &mut plen, val1, cesu8);
            if str == std::ptr::null() {
                panic!()
            }
            let buffer = std::slice::from_raw_parts(str as *const u8, plen);
            let string = String::from_utf8_lossy(buffer).to_string();
            JS_FreeCString(ctx, str);
            string
        }
    }

    pub fn new_function_list_entry(name: &CStr, func: JSCFunction) -> JSCFunctionListEntry {
        JSCFunctionListEntry {
            name: name.as_ptr(),
            prop_flags: (JS_PROP_CONFIGURABLE | JS_PROP_WRITABLE) as u8,
            def_type: JS_DEF_CFUNC as u8,
            magic: 0,
            u: JSCFunctionListEntry__bindgen_ty_1 {
                func: JSCFunctionListEntry__bindgen_ty_1__bindgen_ty_1 {
                    length: 0,
                    cproto: JSCFunctionEnum_JS_CFUNC_generic as u8,
                    cfunc: JSCFunctionType { generic: func },
                },
            },
        }
    }

    pub fn get_property_str(ctx: *mut JSContext, this_obj: JSValue, name: &str) -> JSValue {
        let c_str = CString::new(name).unwrap();
        unsafe {
            let object = JS_GetPropertyStr(ctx, this_obj, c_str.as_ptr());
            object
        }
    }

    pub fn free_value(ctx: *mut JSContext, value: JSValue) {
        unsafe {
            QuickJS_FreeValue(ctx, value);
        }
    }

    pub fn new_object_proto_class(
        ctx: *mut JSContext,
        proto: JSValue,
        class_id: JSClassID,
    ) -> JSValue {
        unsafe { JS_NewObjectProtoClass(ctx, proto, class_id) }
    }

    pub fn set_opaque<T>(obj: JSValue, opaque: *mut T) {
        unsafe {
            let opaque_pointer: *mut ::std::os::raw::c_void = std::mem::transmute(opaque);
            JS_SetOpaque(obj, opaque_pointer);
        }
    }

    pub fn get_opaque<T>(obj: JSValue, class_id: JSClassID) -> *mut T {
        unsafe {
            let opaque_pointer = JS_GetOpaque(obj, class_id);
            std::mem::transmute(opaque_pointer)
        }
    }

    pub fn get_class_proto(ctx: *mut JSContext, class_id: JSClassID) -> JSValue {
        unsafe { JS_GetClassProto(ctx, class_id) }
    }

    pub fn std_dump_error(ctx: *mut JSContext) {
        unsafe {
            js_std_dump_error(ctx);
        }
    }

    pub fn has_property(ctx: *mut JSContext, this_obj: JSValue, prop: JSAtom) -> bool {
        unsafe {
            let state = JS_HasProperty(ctx, this_obj, prop);
            state != 0
        }
    }

    pub fn has_property_str(ctx: *mut JSContext, this_obj: JSValue, prop: &str) -> bool {
        unsafe {
            let c_str = CString::new(prop).unwrap();
            let atom = JS_NewAtom(ctx, c_str.as_ptr());
            let state = JS_HasProperty(ctx, this_obj, atom);
            JS_FreeAtom(ctx, atom);
            state != 0
        }
    }
}
