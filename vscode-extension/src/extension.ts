import * as vscode from 'vscode';
import * as cp from 'child_process';
import * as path from 'path';

export function activate(context: vscode.ExtensionContext) {
    console.log('Hielements extension is now active');

    // Register check command
    const checkCommand = vscode.commands.registerCommand('hielements.check', async () => {
        const editor = vscode.window.activeTextEditor;
        if (!editor) {
            vscode.window.showErrorMessage('No active editor');
            return;
        }

        if (editor.document.languageId !== 'hielements') {
            vscode.window.showErrorMessage('Current file is not a Hielements file');
            return;
        }

        const filePath = editor.document.uri.fsPath;
        await runHielements('check', filePath);
    });

    // Register run command
    const runCommand = vscode.commands.registerCommand('hielements.run', async () => {
        const editor = vscode.window.activeTextEditor;
        if (!editor) {
            vscode.window.showErrorMessage('No active editor');
            return;
        }

        if (editor.document.languageId !== 'hielements') {
            vscode.window.showErrorMessage('Current file is not a Hielements file');
            return;
        }

        const filePath = editor.document.uri.fsPath;
        const workspaceFolder = vscode.workspace.workspaceFolders?.[0]?.uri.fsPath;
        await runHielements('run', filePath, workspaceFolder);
    });

    context.subscriptions.push(checkCommand, runCommand);
}

async function runHielements(command: 'check' | 'run', filePath: string, workspace?: string): Promise<void> {
    const config = vscode.workspace.getConfiguration('hielements');
    const executable = config.get<string>('executable', 'hielements');

    const args = [command, filePath];
    if (command === 'run' && workspace) {
        args.push('--workspace', workspace);
    }

    const outputChannel = vscode.window.createOutputChannel('Hielements');
    outputChannel.show();
    outputChannel.appendLine(`Running: ${executable} ${args.join(' ')}`);
    outputChannel.appendLine('');

    return new Promise((resolve) => {
        const process = cp.spawn(executable, args, {
            cwd: workspace || path.dirname(filePath)
        });

        process.stdout.on('data', (data) => {
            outputChannel.append(data.toString());
        });

        process.stderr.on('data', (data) => {
            outputChannel.append(data.toString());
        });

        process.on('close', (code) => {
            outputChannel.appendLine('');
            if (code === 0) {
                outputChannel.appendLine('✓ Completed successfully');
                vscode.window.showInformationMessage('Hielements: Completed successfully');
            } else {
                outputChannel.appendLine(`✗ Exited with code ${code}`);
                vscode.window.showErrorMessage(`Hielements: Exited with code ${code}`);
            }
            resolve();
        });

        process.on('error', (error) => {
            outputChannel.appendLine(`Error: ${error.message}`);
            vscode.window.showErrorMessage(`Failed to run hielements: ${error.message}`);
            resolve();
        });
    });
}

export function deactivate() {}
