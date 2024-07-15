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
