import * as vscode from 'vscode';

export class MyLogOutputChannel implements vscode.LogOutputChannel {
    logLevel: vscode.LogLevel = vscode.LogLevel.Trace;
    
    onDidChangeLogLevel: vscode.Event<vscode.LogLevel> = () => {
        return new vscode.Disposable(() => { });
    };

    name: string = "MyLogOutputChannel";

    trace(message: string, ...args: any[]): void {
        console.log(message);
    }

    debug(message: string, ...args: any[]): void {
        console.log(message);
    }

    info(message: string, ...args: any[]): void {
        console.log(message);
    }

    warn(message: string, ...args: any[]): void {
        console.log(message);
    }

    error(error: string | Error, ...args: any[]): void {
        console.log(error);
    }

    append(value: string): void {
        console.log(value);
    }

    appendLine(value: string): void {
        console.log(value);
    }

    replace(value: string): void {
        console.log(value);
    }

    clear(): void {
    }

    show(preserveFocus?: boolean | undefined): void;

    show(column?: vscode.ViewColumn | undefined, preserveFocus?: boolean | undefined): void;

    show(column?: unknown, preserveFocus?: unknown): void {
    }

    hide(): void {
    }

    dispose(): void {
    }
}
