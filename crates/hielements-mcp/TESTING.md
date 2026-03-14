# MCP Server Testing Guide

This document provides quick tests to verify the MCP server is working correctly.

## Prerequisites

```bash
# Install the MCP server
cargo install --path crates/hielements-mcp

# Verify installation
hielements-mcp --version
```

## Quick Smoke Test

```bash
# Test initialize and tools listing
cat > /tmp/mcp_test.json << 'EOF'
{"jsonrpc": "2.0", "method": "initialize", "params": {"protocolVersion": "2025-11-25", "capabilities": {}, "clientInfo": {"name": "test-client", "version": "1.0"}}, "id": 1}
{"jsonrpc": "2.0", "method": "notifications/initialized", "params": {}}
{"jsonrpc": "2.0", "method": "tools/list", "params": {}, "id": 2}
EOF

cat /tmp/mcp_test.json | hielements-mcp --workspace .
```

Expected output:
- First response: `initialize` result with server info
- Second response: `tools/list` result with 8 tools

## Available Endpoints

### Tools (8 total)
1. `check_specification` - Validate specification content
2. `check_file` - Validate a .hie file in workspace
3. `run_checks` - Execute architectural checks
4. `list_patterns` - List available patterns
5. `get_pattern` - Get pattern details
6. `list_libraries` - List available libraries
7. `explain_error` - Explain error codes
8. `generate_element` - Generate element boilerplate

### Resources
- `hielements://workspace/specifications` - List all .hie files
- `hielements://patterns/catalog` - Pattern library
- `hielements://libraries/docs` - Library documentation
- `hielements://docs/language-reference` - Language reference
- `hielements://workspace/file/{filename}` - Individual files

### Prompts (5 total)
1. `architect_system` - System architecture design guide
2. `analyze_architecture` - Architecture analysis guide
3. `create_pattern` - Pattern creation guide
4. `fix_violations` - Violation fix guide
5. `implement_pattern` - Pattern implementation guide

## Testing Individual Features

### Test Tool Call

```bash
cat > /tmp/test_tool.json << 'EOF'
{"jsonrpc": "2.0", "method": "initialize", "params": {"protocolVersion": "2025-11-25", "capabilities": {}, "clientInfo": {"name": "test", "version": "1.0"}}, "id": 1}
{"jsonrpc": "2.0", "method": "notifications/initialized", "params": {}}
{"jsonrpc": "2.0", "method": "tools/call", "params": {"name": "list_libraries", "arguments": {}}, "id": 2}
EOF

cat /tmp/test_tool.json | hielements-mcp --workspace .
```

Expected: JSON response with files, python, and rust libraries

### Test Resource Read

```bash
cat > /tmp/test_resource.json << 'EOF'
{"jsonrpc": "2.0", "method": "initialize", "params": {"protocolVersion": "2025-11-25", "capabilities": {}, "clientInfo": {"name": "test", "version": "1.0"}}, "id": 1}
{"jsonrpc": "2.0", "method": "notifications/initialized", "params": {}}
{"jsonrpc": "2.0", "method": "resources/read", "params": {"uri": "hielements://workspace/specifications"}, "id": 2}
EOF

cat /tmp/test_resource.json | hielements-mcp --workspace .
```

Expected: JSON response with list of .hie files in workspace

### Test Prompt Get

```bash
cat > /tmp/test_prompt.json << 'EOF'
{"jsonrpc": "2.0", "method": "initialize", "params": {"protocolVersion": "2025-11-25", "capabilities": {}, "clientInfo": {"name": "test", "version": "1.0"}}, "id": 1}
{"jsonrpc": "2.0", "method": "notifications/initialized", "params": {}}
{"jsonrpc": "2.0", "method": "prompts/get", "params": {"name": "architect_system", "arguments": {"system_description": "A todo app"}}, "id": 2}
EOF

cat /tmp/test_prompt.json | hielements-mcp --workspace .
```

Expected: Prompt message with architecture design guidance

## Verbose Logging

For debugging, enable verbose logging:

```bash
cat /tmp/mcp_test.json | hielements-mcp --workspace . --verbose
```

This will output debug information to stderr, including:
- Raw payload logs
- Tool call logs with arguments
- Error details if any

## Common Issues

### "Command not found: hielements-mcp"
- Run `cargo install --path crates/hielements-mcp`
- Ensure `~/.cargo/bin` is in your PATH

### "Workspace directory does not exist"
- Provide a valid workspace path with `--workspace /path/to/project`
- Or run from within a directory containing .hie files

### No response from server
- Check that you're sending valid JSON-RPC 2.0 requests
- Ensure each request is on a single line
- Send `initialize` request first before other requests

## Integration with Claude Desktop

Add to `~/.config/claude-desktop/config.json` (Linux) or `~/Library/Application Support/Claude/claude_desktop_config.json` (macOS):

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

After adding, restart Claude Desktop.
