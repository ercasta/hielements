# Hielements MCP Server

MCP (Model Context Protocol) server for exposing Hielements functionality to AI agents.

## Overview

This server implements the [MCP specification](https://github.com/modelcontextprotocol/specification) to enable AI agents (like Claude, GPT, etc.) to interact with Hielements through a standardized interface.

## Features

### Resources

The server exposes the following resources that agents can read:

- **`hielements://workspace/specifications`** - List of all `.hie` files in the workspace
- **`hielements://patterns/catalog`** - Available architectural patterns
- **`hielements://libraries/docs`** - Documentation for all Hielements libraries
- **`hielements://docs/language-reference`** - Language syntax and semantics reference
- **`hielements://workspace/file/{filename}`** - Individual specification files

### Tools

The server provides these tools for agents to use:

| Tool | Description |
|------|-------------|
| `check_specification` | Validate Hielements specification content for syntax and semantic errors |
| `check_file` | Validate a specification file in the workspace |
| `run_checks` | Execute checks defined in a specification |
| `list_patterns` | List available architectural patterns |
| `get_pattern` | Get details of a specific pattern |
| `list_libraries` | List available Hielements libraries and their functions |
| `explain_error` | Get detailed explanation of an error code |
| `generate_element` | Generate a basic Hielements element structure |

### Prompts

The server offers guidance prompts for common agent tasks:

| Prompt | Description |
|--------|-------------|
| `architect_system` | Guide for designing a new system architecture |
| `analyze_architecture` | Guide for analyzing an existing specification |
| `create_pattern` | Guide for creating a reusable architectural pattern |
| `fix_violations` | Guide for fixing architectural violations |
| `implement_pattern` | Guide for implementing an architectural pattern |

## Installation

```bash
# Build from source
cargo install --path crates/hielements-mcp

# Or via cargo
cargo install hielements-mcp
```

## Usage

### Starting the Server

```bash
# Start the server in the current directory
hielements-mcp

# Start with a specific workspace
hielements-mcp --workspace /path/to/project

# Enable verbose logging
hielements-mcp --verbose
```

### Configuring with AI Clients

#### Claude Desktop

Add to your Claude Desktop configuration (`~/.config/claude-desktop/config.json` on Linux or `~/Library/Application Support/Claude/claude_desktop_config.json` on macOS):

```json
{
  "mcpServers": {
    "hielements": {
      "command": "hielements-mcp",
      "args": ["--workspace", "/path/to/your/project"]
    }
  }
}
```

#### Other MCP Clients

The server uses stdio transport, which works with any MCP client that supports spawning processes. Configure the client to run `hielements-mcp` with the desired arguments.

## Example Interactions

### Validate a Specification

```json
{
  "jsonrpc": "2.0",
  "method": "tools/call",
  "params": {
    "name": "check_specification",
    "arguments": {
      "content": "import files\n\nelement myapp {\n    scope src = files.folder_selector('src/')\n}"
    }
  },
  "id": 1
}
```

### List Available Libraries

```json
{
  "jsonrpc": "2.0",
  "method": "tools/call",
  "params": {
    "name": "list_libraries"
  },
  "id": 2
}
```

### Read a Specification File

```json
{
  "jsonrpc": "2.0",
  "method": "resources/read",
  "params": {
    "uri": "hielements://workspace/file/architecture.hie"
  },
  "id": 3
}
```

## Development

### Building

```bash
cd crates/hielements-mcp
cargo build
```

### Running Tests

```bash
cargo test -p hielements-mcp
```

### Running with Debug Logging

```bash
RUST_LOG=debug cargo run -p hielements-mcp -- --workspace .
```

## License

Same as the main Hielements project.
