use anyhow::anyhow;
use lsp_server::Connection;
use lsp_types::{notification::Notification, request::Request, MessageType};
use std::path::Path;

fn base_log<S: AsRef<str>>(
    connection: &Connection,
    message: S,
    message_type: MessageType,
    show_message: bool,
) {
    let notification = lsp_server::Notification {
        method: lsp_types::notification::LogMessage::METHOD.to_string(),
        params: serde_json::to_value(&lsp_types::LogMessageParams {
            typ: message_type,
            message: message.as_ref().to_string(),
        })
        .unwrap(),
    };
    let msg = lsp_server::Message::Notification(notification);
    let _ = connection.sender.send(msg);

    if show_message {
        let request = lsp_server::Request {
            id: lsp_types::request::ShowMessageRequest::METHOD
                .to_string()
                .into(),
            method: lsp_types::request::ShowMessageRequest::METHOD.to_string(),
            params: serde_json::to_value(&lsp_types::ShowMessageRequestParams {
                typ: message_type,
                message: message.as_ref().to_string(),
                actions: None,
            })
            .unwrap(),
        };
        let msg = lsp_server::Message::Request(request);
        let _ = connection.sender.send(msg);
    }
}

pub fn log_i<S: AsRef<str>>(connection: &Connection, message: S) {
    base_log(connection, message, MessageType::INFO, true);
}

pub fn log_e<S: AsRef<str>>(connection: &Connection, message: S) {
    base_log(connection, message, MessageType::ERROR, true);
}

pub fn log_d<S: AsRef<str>>(connection: &Connection, message: S) {
    base_log(connection, message, MessageType::INFO, false);
}

pub fn send_request(
    connection: &lsp_server::Connection,
    method: &str,
    id: Option<lsp_server::RequestId>,
    params: serde_json::Value,
) -> std::result::Result<(), crossbeam_channel::SendError<lsp_server::Message>> {
    let id = id.unwrap_or(method.to_string().into());
    let request = lsp_server::Request {
        id,
        method: method.to_string(),
        params,
    };
    let msg = lsp_server::Message::Request(request);
    connection.sender.send(msg)
}

pub fn send_notification(
    connection: &lsp_server::Connection,
    method: &str,
    params: serde_json::Value,
) -> std::result::Result<(), crossbeam_channel::SendError<lsp_server::Message>> {
    let request = lsp_server::Notification {
        method: method.to_string(),
        params,
    };
    let msg = lsp_server::Message::Notification(request);
    connection.sender.send(msg)
}

pub fn send_response(
    connection: &lsp_server::Connection,
    id: lsp_server::RequestId,
    result: Option<serde_json::Value>,
    error: Option<lsp_server::ResponseError>,
) -> std::result::Result<(), crossbeam_channel::SendError<lsp_server::Message>> {
    let request = lsp_server::Response { id, result, error };
    let msg = lsp_server::Message::Response(request);
    connection.sender.send(msg)
}

pub fn pre_process(
    clang_path: &Path,
    shader_path: &Path,
    arguments: &str,
) -> anyhow::Result<String> {
    let mut clang = std::process::Command::new(clang_path);
    clang.arg("-E");
    clang.arg("-P");
    clang.arg("-x");
    clang.arg("c");
    clang.arg(arguments);
    clang.arg(shader_path.to_str().ok_or(anyhow!("Incorrect path"))?);
    let output = clang.output()?;
    let stderr = String::from_utf8(output.stderr);
    let stdout = String::from_utf8(output.stdout);
    let stdout = stdout?;
    let stderr = stderr?;
    if output.status.success() {
        Ok(stdout)
    } else {
        Err(anyhow!("{}", stderr))
    }
}

pub fn strip_fix<S: AsRef<str>>(str: S) -> String {
    str.as_ref()
        .strip_prefix("\"")
        .unwrap()
        .strip_suffix("\"")
        .unwrap()
        .to_string()
}
