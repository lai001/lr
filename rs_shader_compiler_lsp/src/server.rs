use crate::{lsp_types_ext, misc};
use lsp_server::{Connection, Response};
use lsp_types::{
    notification::Notification, request::Request, CompletionOptions, ConfigurationItem,
    HoverProviderCapability, InitializeParams, OneOf, Registration, RegistrationParams,
    ServerCapabilities, TextDocumentSyncCapability, TextDocumentSyncKind, WorkDoneProgressOptions,
};
use serde::{Deserialize, Serialize};
use std::{
    error::Error,
    fs::File,
    io::BufReader,
    iter::zip,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CompileCommand {
    pub arguments: Vec<String>,
    pub file: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Settings {
    pub shader_compile_commands: Option<PathBuf>,
    pub clang_path: Option<PathBuf>,
}

pub struct Server {
    settings: Settings,
}

impl Server {
    pub fn new() -> Self {
        Self {
            settings: Settings {
                clang_path: None,
                shader_compile_commands: None,
            },
        }
    }

    pub fn run(mut self) {
        let (connection, _io_threads) = Connection::stdio();
        let (initialize_id, initialize_params) = connection.initialize_start().unwrap();
        let initialize_params: InitializeParams =
            serde_json::from_value(initialize_params).unwrap();

        let initialize_result = lsp_types::InitializeResult {
            capabilities: server_capabilities(),
            server_info: Some(lsp_types::ServerInfo {
                name: String::from("shader_compiler"),
                version: Some("0.0.1".to_string()),
            }),
        };
        let initialize_result = serde_json::to_value(initialize_result).unwrap();
        connection
            .initialize_finish(initialize_id, initialize_result)
            .unwrap();

        if let Some(_) = initialize_params.initialization_options {}
        register_capability(&connection);

        let shader_compile_commands_cfg_item = ConfigurationItem {
            scope_uri: None,
            section: Some("shaderCompiler.shaderCompileCommands".to_string()),
        };
        let clang_path_cfg_item = ConfigurationItem {
            scope_uri: None,
            section: Some("shaderCompiler.clangPath".to_string()),
        };
        let cfg_items = vec![
            shader_compile_commands_cfg_item.clone(),
            clang_path_cfg_item.clone(),
        ];

        let _ = send_get_cfg_request(&connection, cfg_items.clone());

        while let Ok(event) = connection.receiver.recv() {
            match event {
                lsp_server::Message::Request(request) => {
                    let _ = connection.handle_shutdown(&request);
                    if request.method == lsp_types::request::Completion::METHOD {
                        let result = lsp_types::CompletionResponse::Array(vec![]);
                        let result = serde_json::to_value(&result).ok();
                        let response = Response {
                            id: request.id,
                            result,
                            error: None,
                        };
                        let msg = lsp_server::Message::Response(response);
                        let _ = connection.sender.send(msg);
                    } else if request.method == lsp_types_ext::request::ShaderPreview::METHOD {
                        let shader_preview_params = serde_json::from_value::<
                            lsp_types_ext::ShaderPreviewParams,
                        >(request.params);
                        let mut result: lsp_types_ext::ShaderPreviewResult =
                            lsp_types_ext::ShaderPreviewResult { code: None };
                        if let Ok(shader_preview_params) = shader_preview_params {
                            if let (Some(shader_compile_commands), Some(clang_path)) = (
                                self.settings.shader_compile_commands.clone(),
                                self.settings.clang_path.clone(),
                            ) {
                                if shader_compile_commands.exists() {
                                    let commands =
                                        from_shader_compile_commands_path(shader_compile_commands)
                                            .unwrap_or(vec![]);
                                    for command in commands {
                                        let arguments = command
                                            .arguments
                                            .iter()
                                            .fold("".to_string(), |acc, x| acc + " " + x);
                                        let code = misc::pre_process(
                                            &clang_path,
                                            Path::new(&command.file),
                                            &arguments,
                                        );

                                        if Path::new(&shader_preview_params.shader_file_path)
                                            == Path::new(&command.file)
                                        {
                                            if let Some(code) = code {
                                                result.code = Some(code);
                                                break;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        let result = Some(serde_json::to_value(&result).unwrap());
                        let _ = misc::send_response(&connection, request.id, result, None);
                    }
                }
                lsp_server::Message::Response(response) => {
                    if response.id
                        == lsp_types::request::WorkspaceConfiguration::METHOD
                            .to_string()
                            .into()
                    {
                        let Some(workspace_configuration) = response.result else {
                            return;
                        };
                        if workspace_configuration.is_array() == false {
                            return;
                        }
                        let workspace_configurations = workspace_configuration.as_array().unwrap();
                        if cfg_items.len() != workspace_configurations.len() {
                            return;
                        }
                        for (cfg_item, workspace_configuration) in
                            zip(cfg_items.clone(), workspace_configurations)
                        {
                            if cfg_item.section.clone().unwrap()
                                == shader_compile_commands_cfg_item.section.clone().unwrap()
                            {
                                if workspace_configuration.is_string() {
                                    self.settings.shader_compile_commands = Some(
                                        misc::strip_fix(workspace_configuration.to_string()).into(),
                                    );
                                }
                            }
                            if cfg_item.section.clone().unwrap()
                                == clang_path_cfg_item.section.clone().unwrap()
                            {
                                if workspace_configuration.is_string() {
                                    self.settings.clang_path = Some(
                                        misc::strip_fix(workspace_configuration.to_string()).into(),
                                    );
                                }
                            }
                        }
                    }
                }
                lsp_server::Message::Notification(notification) => {
                    if notification.method == lsp_types::notification::Exit::METHOD {
                        break;
                    } else if notification.method
                        == lsp_types::notification::DidChangeConfiguration::METHOD
                    {
                        let _ = send_get_cfg_request(&connection, cfg_items.clone());
                    }
                }
            }
        }
    }
}

pub fn from_shader_compile_commands_path<P: AsRef<Path>>(
    shader_compile_commands_path: P,
) -> Result<Vec<CompileCommand>, Box<dyn Error>> {
    let file = File::open(shader_compile_commands_path)?;
    let reader = BufReader::new(file);
    let compile_commands = serde_json::from_reader(reader);
    Ok(compile_commands?)
}

pub fn server_capabilities() -> ServerCapabilities {
    ServerCapabilities {
        text_document_sync: Some(TextDocumentSyncCapability::Kind(
            TextDocumentSyncKind::INCREMENTAL,
        )),
        definition_provider: Some(OneOf::Left(true)),
        completion_provider: Some(CompletionOptions {
            completion_item: None,
            resolve_provider: None,
            trigger_characters: Some(vec![".".to_string()]),
            all_commit_characters: None,
            work_done_progress_options: WorkDoneProgressOptions {
                work_done_progress: None,
            },
        }),
        document_formatting_provider: Some(OneOf::Left(true)),
        hover_provider: Some(HoverProviderCapability::Simple(true)),
        inlay_hint_provider: Some(OneOf::Left(true)),
        ..Default::default()
    }
}

pub fn send_get_cfg_request(
    connection: &lsp_server::Connection,
    items: Vec<ConfigurationItem>,
) -> std::result::Result<(), crossbeam_channel::SendError<lsp_server::Message>> {
    let configuration_params = lsp_types::ConfigurationParams { items };
    let request = lsp_server::Request {
        id: lsp_types::request::WorkspaceConfiguration::METHOD
            .to_string()
            .into(),
        method: lsp_types::request::WorkspaceConfiguration::METHOD.to_string(),
        params: serde_json::to_value(&configuration_params).unwrap(),
    };
    let msg = lsp_server::Message::Request(request);
    connection.sender.send(msg)
}

pub fn register_capability(connection: &lsp_server::Connection) {
    let registration_params = RegistrationParams {
        registrations: vec![Registration {
            id: lsp_types::notification::DidChangeConfiguration::METHOD.to_string(),
            method: lsp_types::notification::DidChangeConfiguration::METHOD.to_string(),
            register_options: None,
        }],
    };

    let _ = misc::send_request(
        &connection,
        lsp_types::request::RegisterCapability::METHOD,
        None,
        serde_json::to_value(&registration_params).unwrap(),
    );
}
