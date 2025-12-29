#!/usr/bin/env python3
"""
Sample external library plugin for Hielements.

This demonstrates how to create a custom library that provides selectors and checks
for use in .hie specification files.

Usage in hielements.toml:
    [libraries]
    sample = { executable = "python3", args = ["examples/plugins/sample_plugin.py"] }

Usage in .hie files:
    import sample
    
    element mycomponent:
        scope src = sample.simple_selector('src')
        check sample.file_count_check(src, 10)
"""

import json
import sys


def handle_request(request):
    """Handle a JSON-RPC request and return a response."""
    method = request.get("method", "")
    params = request.get("params", {})
    request_id = request.get("id", 1)
    
    try:
        if method == "library.metadata":
            result = {
                "name": "sample",
                "version": "1.0.0",
                "functions": ["simple_selector"],
                "checks": ["file_count_check", "always_pass", "always_fail"]
            }
        elif method == "library.call":
            result = handle_call(params)
        elif method == "library.check":
            result = handle_check(params)
        else:
            return error_response(request_id, -32601, f"Unknown method: {method}")
        
        return success_response(request_id, result)
    except Exception as e:
        return error_response(request_id, -32000, str(e))


def handle_call(params):
    """Handle a library function call (selector)."""
    function = params.get("function", "")
    args = params.get("args", [])
    workspace = params.get("workspace", ".")
    
    if function == "simple_selector":
        # Simple selector that returns a scope with the given path
        path = extract_string(args[0]) if args else ""
        import os
        
        full_path = os.path.join(workspace, path)
        if os.path.isdir(full_path):
            # Return all files in the directory
            files = []
            for root, _, filenames in os.walk(full_path):
                for filename in filenames:
                    files.append(os.path.join(root, filename))
            return {
                "Scope": {
                    "kind": {"Folder": path},
                    "paths": files,
                    "resolved": True
                }
            }
        elif os.path.isfile(full_path):
            return {
                "Scope": {
                    "kind": {"File": path},
                    "paths": [full_path],
                    "resolved": True
                }
            }
        else:
            return {
                "Scope": {
                    "kind": {"File": path},
                    "paths": [],
                    "resolved": True
                }
            }
    else:
        raise ValueError(f"Unknown function: {function}")


def handle_check(params):
    """Handle a library check function."""
    function = params.get("function", "")
    args = params.get("args", [])
    workspace = params.get("workspace", ".")
    
    if function == "always_pass":
        return {"Pass": None}
    
    elif function == "always_fail":
        message = extract_string(args[0]) if args else "Always fails"
        return {"Fail": message}
    
    elif function == "file_count_check":
        # Check that a scope has at most N files
        scope = extract_scope(args[0]) if args else None
        max_count = extract_int(args[1]) if len(args) > 1 else 100
        
        if scope is None:
            return {"Error": "First argument must be a scope"}
        
        file_count = len(scope.get("paths", []))
        if file_count <= max_count:
            return {"Pass": None}
        else:
            return {"Fail": f"Too many files: {file_count} > {max_count}"}
    
    else:
        raise ValueError(f"Unknown check: {function}")


def extract_string(value):
    """Extract a string from a Value JSON representation."""
    if isinstance(value, str):
        return value
    if isinstance(value, dict):
        if "String" in value:
            return value["String"]
    return str(value)


def extract_int(value):
    """Extract an integer from a Value JSON representation."""
    if isinstance(value, int):
        return value
    if isinstance(value, dict):
        if "Int" in value:
            return value["Int"]
    return int(value)


def extract_scope(value):
    """Extract a Scope from a Value JSON representation."""
    if isinstance(value, dict):
        if "Scope" in value:
            return value["Scope"]
    return None


def success_response(request_id, result):
    """Create a JSON-RPC success response."""
    return {
        "jsonrpc": "2.0",
        "result": result,
        "id": request_id
    }


def error_response(request_id, code, message):
    """Create a JSON-RPC error response."""
    return {
        "jsonrpc": "2.0",
        "error": {
            "code": code,
            "message": message
        },
        "id": request_id
    }


def main():
    """Main loop: read JSON-RPC requests from stdin, write responses to stdout."""
    for line in sys.stdin:
        line = line.strip()
        if not line:
            continue
        
        try:
            request = json.loads(line)
            response = handle_request(request)
            print(json.dumps(response), flush=True)
        except json.JSONDecodeError as e:
            error = error_response(0, -32700, f"Parse error: {e}")
            print(json.dumps(error), flush=True)


if __name__ == "__main__":
    main()
