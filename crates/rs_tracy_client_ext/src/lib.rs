#[macro_export]
macro_rules! span_alloc {
    () => {{
        struct S;
        let type_name = tracy_client::internal::type_name::<S>();
        let function_name = &type_name[..type_name.len() - 3];

        tracy_client::Client::running().unwrap().span_alloc(
            None,
            function_name,
            file!(),
            line!(),
            0,
        )
    }};
    ($label:expr) => {{
        struct S;
        let type_name = tracy_client::internal::type_name::<S>();
        let function_name = &type_name[..type_name.len() - 3];

        tracy_client::Client::running().unwrap().span_alloc(
            Some($label),
            function_name,
            file!(),
            line!(),
            0,
        )
    }};
}
