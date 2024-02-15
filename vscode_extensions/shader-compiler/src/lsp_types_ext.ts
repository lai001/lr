import { ProtocolRequestType } from "vscode-languageserver";

export interface ShaderPreviewParams {
    shaderFilePath: string;
}

export interface ShaderPreviewResult {
    code: string | null;
}

export const ShaderPreviewRequest =
    new ProtocolRequestType<ShaderPreviewParams, ShaderPreviewResult, never, void, void>("shader_compiler/ShaderPreview");
