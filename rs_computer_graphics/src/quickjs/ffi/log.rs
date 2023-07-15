use rs_quickjs::quickjs_bindings::*;

#[no_mangle]
pub extern "C" fn rs_Log_trace(
    ctx: *mut JSContext,
    this_val: JSValue,
    argc: ::std::os::raw::c_int,
    argv: *mut JSValue,
) -> JSValue {
    let slice = unsafe { std::slice::from_raw_parts(argv, argc as usize) };
    let mut message = String::new();
    for val in slice {
        let str = QuickJS::to_c_string_len2(ctx, *val, 0);
        message = message + " " + &str;
    }
    log::trace!("{}", message);
    return QuickJS::undefined();
}
