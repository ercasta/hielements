# Hielements VSCode Extension

This extension provides language support for Hielements, a language for describing and enforcing software structure.

## Features

- **Syntax Highlighting**: Full syntax highlighting for `.hie` files
- **Commands**: 
  - `Hielements: Check Current File` - Validate the current file
  - `Hielements: Run Checks` - Run all checks defined in the file

## Requirements

- The `hielements` CLI must be installed and available in your PATH
- Alternatively, configure the path in settings

## Extension Settings

- `hielements.executable`: Path to the hielements executable (default: `hielements`)
- `hielements.workspace`: Workspace directory for running checks

## Installation

### From Source

1. Navigate to the `vscode-extension` directory
2. Run `npm install`
3. Run `npm run compile`
4. Press F5 in VS Code to launch Extension Development Host

### Building VSIX

```bash
npm install -g @vscode/vsce
vsce package
```

Then install the generated `.vsix` file in VS Code.
