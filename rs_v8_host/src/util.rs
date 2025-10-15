fn to_string(scope: &mut v8::PinScope, args: v8::FunctionCallbackArguments) -> String {
    (0..args.length())
        .map(|i| {
            let message = args
                .get(i)
                .to_string(scope)
                .map(|x| x.to_rust_string_lossy(scope));
            message
        })
        .flatten()
        .collect::<Vec<String>>()
        .join(" ")
}

pub(crate) fn println_callback(
    scope: &mut v8::PinScope,
    args: v8::FunctionCallbackArguments,
    _: v8::ReturnValue,
) {
    println!("{}", to_string(scope, args));
}

pub(crate) fn log_callback(
    scope: &mut v8::PinScope,
    args: v8::FunctionCallbackArguments,
    _: v8::ReturnValue,
) {
    log::trace!("{}", to_string(scope, args));
}

pub fn return_exception(scope: &mut v8::PinScope, ret_val: &mut v8::ReturnValue, reason: &str) {
    let exception = v8::String::new(&scope, reason);
    match exception {
        Some(exception) => {
            ret_val.set(scope.throw_exception(exception.into()));
        }
        None => match scope.terminate_execution() {
            true => {}
            false => {
                panic!("Isolate was already destroyed.")
            }
        },
    }
}
