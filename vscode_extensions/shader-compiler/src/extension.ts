import * as vscode from 'vscode';

import {
    LanguageClient,
    LanguageClientOptions,
    ServerOptions,
    ShowMessageNotification,
    TransportKind
} from 'vscode-languageclient/node';

import { MyLogOutputChannel } from './my_log_output_channel';
import { ShaderPreviewParams, ShaderPreviewRequest } from './lsp_types_ext';

let client: LanguageClient;

const clangPathCfg = 'shaderCompiler.clangPath';
const shaderCompileCommandsCfg = 'shaderCompiler.shaderCompileCommands';
const shaderCompilerPathCfg = 'shaderCompiler.shaderCompilerPath';

class MyShaderPreviewParams implements ShaderPreviewParams {
    shaderFilePath: string = "";
    constructor(shaderFilePath: string) {
        this.shaderFilePath = shaderFilePath;
    }
}

export function activate(context: vscode.ExtensionContext) {
    let clangPath: string | undefined | null = vscode.workspace.getConfiguration().get(clangPathCfg);
    let shaderCompileCommands: string | undefined | null = vscode.workspace.getConfiguration().get(shaderCompileCommandsCfg);
    let shaderCompilerPath: string | undefined | null = vscode.workspace.getConfiguration().get(shaderCompilerPathCfg);

    vscode.workspace.onDidChangeConfiguration(e => {
        if (e.affectsConfiguration(clangPathCfg)) {
            clangPath = vscode.workspace.getConfiguration().get(clangPathCfg);
        } else if (e.affectsConfiguration(shaderCompileCommandsCfg)) {
            shaderCompileCommands = vscode.workspace.getConfiguration().get(shaderCompileCommandsCfg);
        } else if (e.affectsConfiguration(shaderCompilerPathCfg)) {
            shaderCompilerPath = vscode.workspace.getConfiguration().get(shaderCompilerPathCfg);
        }
    });

    if (shaderCompilerPath) {
        const serverModule = shaderCompilerPath;

        const serverOptions: ServerOptions = {
            run: { command: serverModule, transport: TransportKind.stdio },
            debug: {
                command: serverModule,
                transport: TransportKind.stdio,
            }
        };

        const clientOptions: LanguageClientOptions = {
            outputChannel: new MyLogOutputChannel(),
            documentSelector: [{ scheme: 'file', language: 'wgsl' }],
            synchronize: {
                fileEvents: vscode.workspace.createFileSystemWatcher('**/.clientrc')
            }
        };

        client = new LanguageClient(
            'wgslLanguageServer',
            'wgsl Language Server',
            serverOptions,
            clientOptions
        );

        client.onNotification(ShowMessageNotification.type, function (params) {
            vscode.window.showInformationMessage(params.message);
            return new vscode.Disposable(() => { });
        });

        client.start();
    } else {
        vscode.window.showErrorMessage("Can not active shaderCompiler.");
    }

    vscode.workspace.onDidOpenTextDocument(function (event) {

    });

    vscode.commands.registerCommand("extension.shader_compiler.preview", function (args) {
        let currentActiveFile = vscode.window.activeTextEditor?.document.fileName;
        let currentActiveFilePath = vscode.window.activeTextEditor?.document.uri.fsPath;
        let languageId = vscode.window.activeTextEditor?.document.languageId;

        let workspaceFolders = vscode.workspace.workspaceFolders;
        if (workspaceFolders != undefined) {

        }

        (async function () {
            if (currentActiveFilePath) {
                let shaderPreviewResult = await client.sendRequest(ShaderPreviewRequest, new MyShaderPreviewParams(currentActiveFilePath));
                if (shaderPreviewResult.code) {
                    vscode.workspace.openTextDocument({
                        content: shaderPreviewResult.code,
                        language: "wgsl"
                    }).then(newDocument => {
                        vscode.window.showTextDocument(newDocument);
                    });
                }
            }
        })();

    });
}

export function deactivate(): Thenable<void> | undefined {
    if (!client) {
        return undefined;
    }
    return client.stop();
}
