pub(crate) fn println_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    _: v8::ReturnValue,
) {
    let message = (0..args.length())
        .map(|i| {
            let message = args
                .get(i)
                .to_string(scope)
                .map(|x| x.to_rust_string_lossy(scope));
            message
        })
        .flatten()
        .collect::<Vec<String>>()
        .join(" ");
    println!("{}", message);
}

pub fn return_exception(scope: &mut v8::HandleScope, ret_val: &mut v8::ReturnValue, reason: &str) {
    let exception = v8::String::new(scope, reason);
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
