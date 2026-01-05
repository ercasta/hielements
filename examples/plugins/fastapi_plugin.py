#!/usr/bin/env python3
"""
FastAPI Authentication Checker Plugin for Hielements.

This plugin provides deep code analysis capabilities for FastAPI applications,
specifically focusing on authentication verification for API endpoints.

Features:
- Parse FastAPI route decorators using libcst
- Detect authentication patterns (decorators, dependencies, security schemes)
- Verify that POST and GET endpoints have proper authentication
- Support multiple authentication strategies

Usage in hielements.toml:
    [libraries]
    fastapi = { executable = "python3", args = ["examples/plugins/fastapi_plugin.py"] }

Usage in .hie files:
    import fastapi
    
    element my_api:
        scope api = fastapi.app_selector('app/api')
        check fastapi.all_routes_authenticated(api)
        check fastapi.has_authentication_scheme(api)

Dependencies:
    pip install libcst

Author: Hielements Community
License: MIT
"""

import json
import sys
import os
from typing import Dict, List, Any, Optional, Set
from dataclasses import dataclass, asdict

try:
    import libcst as cst
    LIBCST_AVAILABLE = True
except ImportError:
    LIBCST_AVAILABLE = False
    print("Warning: libcst not available, using fallback AST parser", file=sys.stderr)
    import ast


@dataclass
class Route:
    """Represents a FastAPI route endpoint."""
    name: str
    path: str
    method: str  # GET, POST, PUT, DELETE, etc.
    authenticated: bool
    auth_method: Optional[str] = None
    line_number: int = 0


class FastAPIAuthVisitor(cst.CSTTransformer):
    """
    LibCST visitor to analyze FastAPI code for authentication patterns.
    
    Detects:
    1. Route decorators (@app.get, @app.post, etc.)
    2. Authentication decorators (@requires_auth, @authenticated)
    3. Dependency injection with auth (Depends(get_current_user))
    4. FastAPI Security schemes (Security(), HTTPBearer, OAuth2PasswordBearer)
    """
    
    def __init__(self):
        super().__init__()
        self.routes: List[Route] = []
        self.has_security_scheme = False
        self.security_schemes: Set[str] = set()
        
    def visit_FunctionDef(self, node: cst.FunctionDef) -> None:
        """Analyze function definitions for FastAPI routes."""
        # Check if this is a route handler
        route_info = self._extract_route_info(node)
        if route_info:
            # Check for authentication
            has_auth, auth_method = self._check_authentication(node)
            
            route = Route(
                name=node.name.value,
                path=route_info['path'],
                method=route_info['method'],
                authenticated=has_auth,
                auth_method=auth_method,
                line_number=node.leading_lines[0].line.start if node.leading_lines else 0
            )
            self.routes.append(route)
    
    def visit_Assign(self, node: cst.Assign) -> None:
        """Detect security scheme definitions."""
        # Look for OAuth2PasswordBearer, HTTPBearer, etc.
        if isinstance(node.value, cst.Call):
            if self._is_security_scheme_call(node.value):
                self.has_security_scheme = True
                # Extract scheme name if available
                for target in node.targets:
                    if isinstance(target.target, cst.Name):
                        self.security_schemes.add(target.target.value)
    
    def _extract_route_info(self, node: cst.FunctionDef) -> Optional[Dict[str, str]]:
        """Extract route path and HTTP method from decorators."""
        for decorator in node.decorators:
            if isinstance(decorator.decorator, cst.Call):
                # Check for @app.get(), @app.post(), etc.
                if isinstance(decorator.decorator.func, cst.Attribute):
                    attr = decorator.decorator.func
                    if isinstance(attr.value, cst.Name):
                        # Could be 'app', 'router', 'api', etc.
                        method = attr.attr.value.upper()
                        if method in ['GET', 'POST', 'PUT', 'DELETE', 'PATCH', 'OPTIONS', 'HEAD']:
                            # Extract path from first argument
                            path = self._extract_path_from_decorator(decorator.decorator)
                            return {'method': method, 'path': path}
        return None
    
    def _extract_path_from_decorator(self, call: cst.Call) -> str:
        """Extract the path string from route decorator."""
        if call.args:
            first_arg = call.args[0]
            if isinstance(first_arg.value, cst.SimpleString):
                return first_arg.value.value.strip('"\'')
        return "/"
    
    def _check_authentication(self, node: cst.FunctionDef) -> tuple[bool, Optional[str]]:
        """
        Check if a route has authentication.
        
        Returns: (has_auth, auth_method)
        """
        # Strategy 1: Check for authentication decorators
        for decorator in node.decorators:
            if self._is_auth_decorator(decorator):
                return (True, "decorator")
        
        # Strategy 2: Check for Depends() in function parameters
        for param in node.params.params:
            if param.default and self._is_auth_dependency(param.default):
                return (True, "dependency_injection")
        
        # Strategy 3: Check for Security() in function parameters
        for param in node.params.params:
            if param.default and self._is_security_param(param.default):
                return (True, "security_scheme")
        
        return (False, None)
    
    def _is_auth_decorator(self, decorator: cst.Decorator) -> bool:
        """Check if decorator is an authentication decorator."""
        auth_names = [
            'requires_auth', 'authenticated', 'login_required',
            'require_auth', 'auth_required', 'protected'
        ]
        
        if isinstance(decorator.decorator, cst.Name):
            return decorator.decorator.value in auth_names
        elif isinstance(decorator.decorator, cst.Call):
            if isinstance(decorator.decorator.func, cst.Name):
                return decorator.decorator.func.value in auth_names
        return False
    
    def _is_auth_dependency(self, default: cst.BaseExpression) -> bool:
        """Check if parameter default is an auth dependency like Depends(get_current_user)."""
        if isinstance(default, cst.Call):
            if isinstance(default.func, cst.Name):
                if default.func.value == 'Depends':
                    # Check if the argument suggests authentication
                    if default.args:
                        arg = default.args[0].value
                        if isinstance(arg, cst.Name):
                            arg_name = arg.value.lower()
                            return any(keyword in arg_name for keyword in 
                                     ['auth', 'user', 'token', 'login', 'verify'])
        return False
    
    def _is_security_param(self, default: cst.BaseExpression) -> bool:
        """Check if parameter uses FastAPI Security()."""
        if isinstance(default, cst.Call):
            if isinstance(default.func, cst.Name):
                return default.func.value in ['Security', 'Depends']
        return False
    
    def _is_security_scheme_call(self, call: cst.Call) -> bool:
        """Check if this is a security scheme initialization."""
        security_schemes = [
            'OAuth2PasswordBearer',
            'OAuth2AuthorizationCodeBearer',
            'HTTPBearer',
            'HTTPBasic',
            'HTTPDigest',
            'APIKeyHeader',
            'APIKeyQuery',
            'APIKeyCookie'
        ]
        
        if isinstance(call.func, cst.Name):
            return call.func.value in security_schemes
        return False


class FastAPIAuthAnalyzer:
    """Main analyzer class for FastAPI authentication checking."""
    
    def __init__(self, workspace: str):
        self.workspace = workspace
    
    def analyze_file(self, file_path: str) -> Dict[str, Any]:
        """Analyze a single Python file for FastAPI routes and authentication."""
        full_path = os.path.join(self.workspace, file_path)
        
        try:
            with open(full_path, 'r', encoding='utf-8') as f:
                source_code = f.read()
            
            if LIBCST_AVAILABLE:
                return self._analyze_with_libcst(source_code)
            else:
                return self._analyze_with_ast(source_code)
        
        except FileNotFoundError:
            return {
                'routes': [],
                'has_security_scheme': False,
                'error': f'File not found: {file_path}'
            }
        except Exception as e:
            return {
                'routes': [],
                'has_security_scheme': False,
                'error': f'Error analyzing file: {str(e)}'
            }
    
    def _analyze_with_libcst(self, source_code: str) -> Dict[str, Any]:
        """Analyze using libcst (preferred method)."""
        try:
            tree = cst.parse_module(source_code)
            visitor = FastAPIAuthVisitor()
            tree.visit(visitor)
            
            return {
                'routes': [asdict(route) for route in visitor.routes],
                'has_security_scheme': visitor.has_security_scheme,
                'security_schemes': list(visitor.security_schemes)
            }
        except Exception as e:
            return {
                'routes': [],
                'has_security_scheme': False,
                'error': f'LibCST parsing error: {str(e)}'
            }
    
    def _analyze_with_ast(self, source_code: str) -> Dict[str, Any]:
        """Fallback analysis using standard ast module."""
        # Simplified fallback implementation
        # This won't be as accurate but provides basic functionality
        routes = []
        has_security_scheme = False
        
        try:
            tree = ast.parse(source_code)
            
            for node in ast.walk(tree):
                if isinstance(node, ast.FunctionDef):
                    # Look for FastAPI route decorators
                    for decorator in node.decorator_list:
                        if isinstance(decorator, ast.Call):
                            if isinstance(decorator.func, ast.Attribute):
                                method = decorator.func.attr.upper()
                                if method in ['GET', 'POST', 'PUT', 'DELETE', 'PATCH']:
                                    # Basic auth detection: check for 'Depends' in args
                                    has_auth = self._check_depends_in_args_ast(node)
                                    
                                    routes.append({
                                        'name': node.name,
                                        'path': '/',  # Simplified
                                        'method': method,
                                        'authenticated': has_auth,
                                        'auth_method': 'dependency_injection' if has_auth else None,
                                        'line_number': node.lineno
                                    })
            
            return {
                'routes': routes,
                'has_security_scheme': has_security_scheme,
                'note': 'Using fallback AST parser - install libcst for better analysis'
            }
        except Exception as e:
            return {
                'routes': [],
                'has_security_scheme': False,
                'error': f'AST parsing error: {str(e)}'
            }
    
    def _check_depends_in_args_ast(self, func_node: ast.FunctionDef) -> bool:
        """Check if function has Depends() in arguments (AST fallback)."""
        for arg in func_node.args.defaults:
            if isinstance(arg, ast.Call):
                if isinstance(arg.func, ast.Name):
                    if arg.func.id == 'Depends':
                        return True
        return False
    
    def analyze_module(self, module_path: str) -> Dict[str, Any]:
        """Analyze a Python module (directory with __init__.py or single file)."""
        full_path = os.path.join(self.workspace, module_path)
        
        if os.path.isfile(full_path):
            return self.analyze_file(module_path)
        
        # If it's a module directory, analyze all Python files
        all_routes = []
        has_security_scheme = False
        errors = []
        
        if os.path.isdir(full_path):
            for root, dirs, files in os.walk(full_path):
                for file in files:
                    if file.endswith('.py'):
                        file_path = os.path.relpath(os.path.join(root, file), self.workspace)
                        result = self.analyze_file(file_path)
                        
                        if 'error' in result:
                            errors.append(result['error'])
                        else:
                            all_routes.extend(result.get('routes', []))
                            has_security_scheme = has_security_scheme or result.get('has_security_scheme', False)
        
        return {
            'routes': all_routes,
            'has_security_scheme': has_security_scheme,
            'errors': errors if errors else None
        }


# ============================================================================
# JSON-RPC Protocol Implementation
# ============================================================================

def handle_request(request: Dict[str, Any], workspace: str) -> Dict[str, Any]:
    """Handle a JSON-RPC request and return a response."""
    method = request.get("method", "")
    params = request.get("params", {})
    request_id = request.get("id", 1)
    
    try:
        if method == "library.metadata":
            result = {
                "name": "fastapi",
                "version": "1.0.0",
                "description": "FastAPI authentication checker with libcst",
                "functions": [
                    "app_selector",
                    "route_selector",
                    "authenticated_routes",
                    "unauthenticated_routes"
                ],
                "checks": [
                    "all_routes_authenticated",
                    "route_has_authentication",
                    "has_authentication_scheme",
                    "post_routes_authenticated",
                    "get_routes_authenticated"
                ]
            }
        elif method == "library.doc":
            result = get_library_documentation()
        elif method == "library.call":
            result = handle_call(params, workspace)
        elif method == "library.check":
            result = handle_check(params, workspace)
        else:
            return error_response(request_id, -32601, f"Unknown method: {method}")
        
        return success_response(request_id, result)
    except Exception as e:
        return error_response(request_id, -32000, str(e))


def get_library_documentation() -> Dict[str, Any]:
    """Return comprehensive library documentation."""
    return {
        "name": "fastapi",
        "description": "FastAPI authentication analysis library using libcst for deep code inspection",
        "version": "1.0.0",
        "functions": [
            {
                "name": "app_selector",
                "description": "Select FastAPI app instances in a module",
                "parameters": [
                    {"name": "path", "type": "string", "description": "Module path"}
                ],
                "return_type": "Scope",
                "example": "fastapi.app_selector('app/api')"
            },
            {
                "name": "route_selector",
                "description": "Select routes by HTTP method",
                "parameters": [
                    {"name": "scope", "type": "Scope", "description": "Module scope"},
                    {"name": "method", "type": "string", "description": "HTTP method (GET, POST, etc.)"}
                ],
                "return_type": "List",
                "example": "fastapi.route_selector(app, 'POST')"
            },
            {
                "name": "authenticated_routes",
                "description": "Get all authenticated routes in a scope",
                "parameters": [
                    {"name": "scope", "type": "Scope", "description": "Module scope"}
                ],
                "return_type": "List",
                "example": "fastapi.authenticated_routes(app)"
            },
            {
                "name": "unauthenticated_routes",
                "description": "Get all unauthenticated routes in a scope",
                "parameters": [
                    {"name": "scope", "type": "Scope", "description": "Module scope"}
                ],
                "return_type": "List",
                "example": "fastapi.unauthenticated_routes(app)"
            }
        ],
        "checks": [
            {
                "name": "all_routes_authenticated",
                "description": "Verify that all routes in scope have authentication",
                "parameters": [
                    {"name": "scope", "type": "Scope", "description": "Module scope"}
                ],
                "return_type": "CheckResult",
                "example": "check fastapi.all_routes_authenticated(api_module)"
            },
            {
                "name": "post_routes_authenticated",
                "description": "Verify that all POST routes have authentication",
                "parameters": [
                    {"name": "scope", "type": "Scope", "description": "Module scope"}
                ],
                "return_type": "CheckResult",
                "example": "check fastapi.post_routes_authenticated(api_module)"
            },
            {
                "name": "get_routes_authenticated",
                "description": "Verify that all GET routes have authentication",
                "parameters": [
                    {"name": "scope", "type": "Scope", "description": "Module scope"}
                ],
                "return_type": "CheckResult",
                "example": "check fastapi.get_routes_authenticated(api_module)"
            },
            {
                "name": "has_authentication_scheme",
                "description": "Check if module defines authentication schemes",
                "parameters": [
                    {"name": "scope", "type": "Scope", "description": "Module scope"}
                ],
                "return_type": "CheckResult",
                "example": "check fastapi.has_authentication_scheme(auth_module)"
            }
        ]
    }


def handle_call(params: Dict[str, Any], workspace: str) -> Dict[str, Any]:
    """Handle a library function call (selector)."""
    function = params.get("function", "")
    args = params.get("args", [])
    
    analyzer = FastAPIAuthAnalyzer(workspace)
    
    if function == "app_selector":
        # Select FastAPI app module
        path = extract_string(args[0]) if args else ""
        return create_module_scope(workspace, path)
    
    elif function == "route_selector":
        # Select routes by HTTP method
        scope = extract_scope(args[0]) if args else None
        method = extract_string(args[1]).upper() if len(args) > 1 else "GET"
        
        if scope is None:
            raise ValueError("First argument must be a scope")
        
        result = analyze_scope(analyzer, scope)
        routes = [r for r in result['routes'] if r['method'] == method]
        
        return {"List": [{"String": r['name']} for r in routes]}
    
    elif function == "authenticated_routes":
        scope = extract_scope(args[0]) if args else None
        if scope is None:
            raise ValueError("First argument must be a scope")
        
        result = analyze_scope(analyzer, scope)
        auth_routes = [r for r in result['routes'] if r['authenticated']]
        
        return {"List": [{"String": r['name']} for r in auth_routes]}
    
    elif function == "unauthenticated_routes":
        scope = extract_scope(args[0]) if args else None
        if scope is None:
            raise ValueError("First argument must be a scope")
        
        result = analyze_scope(analyzer, scope)
        unauth_routes = [r for r in result['routes'] if not r['authenticated']]
        
        return {"List": [{"String": r['name']} for r in unauth_routes]}
    
    else:
        raise ValueError(f"Unknown function: {function}")


def handle_check(params: Dict[str, Any], workspace: str) -> Dict[str, Any]:
    """Handle a library check function."""
    function = params.get("function", "")
    args = params.get("args", [])
    
    analyzer = FastAPIAuthAnalyzer(workspace)
    
    if function == "all_routes_authenticated":
        scope = extract_scope(args[0]) if args else None
        if scope is None:
            return {"Error": "First argument must be a scope"}
        
        result = analyze_scope(analyzer, scope)
        
        if 'error' in result:
            return {"Error": result['error']}
        
        routes = result.get('routes', [])
        if not routes:
            return {"Pass": None}  # No routes, so trivially pass
        
        unauth_routes = [r for r in routes if not r['authenticated']]
        
        if unauth_routes:
            route_names = ', '.join([f"{r['method']} {r['path']} ({r['name']})" for r in unauth_routes])
            return {"Fail": f"Found {len(unauth_routes)} unauthenticated routes: {route_names}"}
        
        return {"Pass": None}
    
    elif function == "post_routes_authenticated":
        return check_method_authenticated(analyzer, args, "POST")
    
    elif function == "get_routes_authenticated":
        return check_method_authenticated(analyzer, args, "GET")
    
    elif function == "has_authentication_scheme":
        scope = extract_scope(args[0]) if args else None
        if scope is None:
            return {"Error": "First argument must be a scope"}
        
        result = analyze_scope(analyzer, scope)
        
        if 'error' in result:
            return {"Error": result['error']}
        
        if result.get('has_security_scheme', False):
            return {"Pass": None}
        else:
            return {"Fail": "No authentication scheme found (OAuth2PasswordBearer, HTTPBearer, etc.)"}
    
    elif function == "route_has_authentication":
        # Check specific route
        route_name = extract_string(args[0]) if args else ""
        scope = extract_scope(args[1]) if len(args) > 1 else None
        
        if scope is None:
            return {"Error": "Second argument must be a scope"}
        
        result = analyze_scope(analyzer, scope)
        routes = result.get('routes', [])
        
        for route in routes:
            if route['name'] == route_name:
                if route['authenticated']:
                    return {"Pass": None}
                else:
                    return {"Fail": f"Route '{route_name}' is not authenticated"}
        
        return {"Error": f"Route '{route_name}' not found"}
    
    else:
        raise ValueError(f"Unknown check: {function}")


def check_method_authenticated(analyzer: FastAPIAuthAnalyzer, args: List[Any], method: str) -> Dict[str, Any]:
    """Helper to check if all routes of a specific method are authenticated."""
    scope = extract_scope(args[0]) if args else None
    if scope is None:
        return {"Error": "First argument must be a scope"}
    
    result = analyze_scope(analyzer, scope)
    
    if 'error' in result:
        return {"Error": result['error']}
    
    routes = result.get('routes', [])
    method_routes = [r for r in routes if r['method'] == method]
    
    if not method_routes:
        return {"Pass": None}  # No routes of this method
    
    unauth_routes = [r for r in method_routes if not r['authenticated']]
    
    if unauth_routes:
        route_names = ', '.join([f"{r['path']} ({r['name']})" for r in unauth_routes])
        return {"Fail": f"Found {len(unauth_routes)} unauthenticated {method} routes: {route_names}"}
    
    return {"Pass": None}


def analyze_scope(analyzer: FastAPIAuthAnalyzer, scope: Dict[str, Any]) -> Dict[str, Any]:
    """Analyze all files in a scope."""
    paths = scope.get('paths', [])
    
    all_routes = []
    has_security_scheme = False
    errors = []
    
    for path in paths:
        # Make path relative to workspace
        rel_path = path
        if path.startswith(analyzer.workspace):
            rel_path = os.path.relpath(path, analyzer.workspace)
        
        result = analyzer.analyze_file(rel_path)
        
        if 'error' in result:
            errors.append(result['error'])
        else:
            all_routes.extend(result.get('routes', []))
            has_security_scheme = has_security_scheme or result.get('has_security_scheme', False)
    
    return {
        'routes': all_routes,
        'has_security_scheme': has_security_scheme,
        'errors': errors if errors else None
    }


def create_module_scope(workspace: str, module_path: str) -> Dict[str, Any]:
    """Create a scope for a Python module."""
    full_path = os.path.join(workspace, module_path)
    
    paths = []
    if os.path.isfile(full_path):
        paths = [full_path]
    elif os.path.isdir(full_path):
        for root, dirs, files in os.walk(full_path):
            for file in files:
                if file.endswith('.py'):
                    paths.append(os.path.join(root, file))
    
    return {
        "Scope": {
            "kind": {"Folder": module_path},
            "paths": paths,
            "resolved": True
        }
    }


# ============================================================================
# Helper Functions
# ============================================================================

def extract_string(value: Any) -> str:
    """Extract a string from a Value JSON representation."""
    if isinstance(value, str):
        return value
    if isinstance(value, dict):
        if "String" in value:
            return value["String"]
    return str(value)


def extract_int(value: Any) -> int:
    """Extract an integer from a Value JSON representation."""
    if isinstance(value, int):
        return value
    if isinstance(value, dict):
        if "Int" in value:
            return value["Int"]
    return int(value)


def extract_scope(value: Any) -> Optional[Dict[str, Any]]:
    """Extract a Scope from a Value JSON representation."""
    if isinstance(value, dict):
        if "Scope" in value:
            return value["Scope"]
    return None


def success_response(request_id: Any, result: Any) -> Dict[str, Any]:
    """Create a JSON-RPC success response."""
    return {
        "jsonrpc": "2.0",
        "result": result,
        "id": request_id
    }


def error_response(request_id: Any, code: int, message: str) -> Dict[str, Any]:
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
    # Get workspace from environment or use current directory
    workspace = os.environ.get('HIELEMENTS_WORKSPACE', os.getcwd())
    
    for line in sys.stdin:
        line = line.strip()
        if not line:
            continue
        
        try:
            request = json.loads(line)
            response = handle_request(request, workspace)
            print(json.dumps(response), flush=True)
        except json.JSONDecodeError as e:
            # Per JSON-RPC 2.0 spec, use null for id when it cannot be determined
            error = error_response(None, -32700, f"Parse error: {e}")
            print(json.dumps(error), flush=True)


if __name__ == "__main__":
    main()
