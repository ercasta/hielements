//! FastAPI Authentication Checker - WASM Plugin
//!
//! This WASM plugin analyzes Python AST (in JSON format) to detect
//! FastAPI routes and verify they have proper authentication.
//!
//! ## Architecture
//!
//! 1. Host (Hielements) parses Python code to AST using RustPython or libcst
//! 2. Host serializes AST to JSON and passes to this WASM plugin
//! 3. Plugin analyzes JSON AST for FastAPI patterns
//! 4. Plugin returns CheckResult (Pass/Fail/Error)
//!
//! ## Authentication Detection Strategies
//!
//! - Decorator-based: @requires_auth, @authenticated
//! - Dependency injection: Depends(get_current_user)
//! - FastAPI Security: Security(oauth2_scheme), HTTPBearer
//! - OAuth2: OAuth2PasswordBearer, OAuth2AuthorizationCodeBearer

use serde::{Deserialize, Serialize};
use std::alloc::{alloc, dealloc, Layout};
use std::slice;

/// Simplified Python AST representation
#[derive(Debug, Deserialize)]
struct Module {
    #[serde(rename = "type")]
    node_type: String,
    body: Vec<Statement>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
enum Statement {
    FunctionDef {
        name: String,
        decorators: Vec<Decorator>,
        parameters: Vec<Parameter>,
        #[serde(default)]
        lineno: usize,
    },
    Assign {
        targets: Vec<Target>,
        value: Box<Expression>,
    },
    #[serde(other)]
    Other,
}

#[derive(Debug, Deserialize)]
struct Decorator {
    #[serde(rename = "type")]
    node_type: String,
    #[serde(default)]
    name: String,
    #[serde(default)]
    func: Option<Box<Expression>>,
    #[serde(default)]
    args: Vec<Expression>,
}

#[derive(Debug, Deserialize)]
struct Parameter {
    name: String,
    #[serde(default)]
    annotation: Option<String>,
    #[serde(default)]
    default: Option<Box<Expression>>,
}

#[derive(Debug, Deserialize)]
struct Target {
    #[serde(rename = "type")]
    node_type: String,
    #[serde(default)]
    name: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
enum Expression {
    Call {
        func: Box<Expression>,
        #[serde(default)]
        args: Vec<Expression>,
    },
    Attribute {
        value: Box<Expression>,
        attr: String,
    },
    Name {
        id: String,
    },
    Constant {
        value: serde_json::Value,
    },
    #[serde(other)]
    Other,
}

/// Route information extracted from AST
#[derive(Debug, Serialize)]
struct Route {
    name: String,
    path: String,
    method: String,  // GET, POST, PUT, DELETE, etc.
    authenticated: bool,
    auth_method: Option<String>,
    line_number: usize,
}

/// Check result types
#[derive(Debug, Serialize)]
#[serde(untagged)]
enum CheckResult {
    Pass { Pass: Option<()> },
    Fail { Fail: String },
    Error { Error: String },
}

/// Analyze a Python module AST for FastAPI routes and authentication
fn analyze_module(module: Module) -> Vec<Route> {
    let mut routes = Vec::new();
    
    for statement in module.body {
        if let Statement::FunctionDef { name, decorators, parameters, lineno } = statement {
            // Check if this is a FastAPI route
            if let Some((method, path)) = extract_route_info(&decorators) {
                // Check for authentication
                let (has_auth, auth_method) = check_authentication(&decorators, &parameters);
                
                routes.push(Route {
                    name,
                    path,
                    method,
                    authenticated: has_auth,
                    auth_method,
                    line_number: lineno,
                });
            }
        }
    }
    
    routes
}

/// Extract HTTP method and path from decorators
fn extract_route_info(decorators: &[Decorator]) -> Option<(String, String)> {
    for decorator in decorators {
        // Look for @app.get(), @app.post(), etc.
        if let Some(ref func) = decorator.func {
            if let Some((method, path)) = match_route_decorator(func, &decorator.args) {
                return Some((method, path));
            }
        }
    }
    None
}

fn match_route_decorator(func: &Expression, args: &[Expression]) -> Option<(String, String)> {
    if let Expression::Attribute { value, attr } = func {
        // Check if it's app.get, app.post, router.get, api.post, etc.
        let method = attr.to_uppercase();
        
        if matches!(method.as_str(), "GET" | "POST" | "PUT" | "DELETE" | "PATCH" | "OPTIONS" | "HEAD") {
            // Extract path from first argument
            let path = if let Some(Expression::Constant { value }) = args.first() {
                value.as_str().unwrap_or("/").to_string()
            } else {
                "/".to_string()
            };
            
            return Some((method, path));
        }
    }
    None
}

/// Check if a route has authentication
fn check_authentication(decorators: &[Decorator], parameters: &[Parameter]) -> (bool, Option<String>) {
    // Strategy 1: Check for authentication decorators
    for decorator in decorators {
        if is_auth_decorator(&decorator.name) {
            return (true, Some("decorator".to_string()));
        }
    }
    
    // Strategy 2: Check for Depends() with auth functions in parameters
    for param in parameters {
        if let Some(ref default) = param.default {
            if let Some(auth_method) = check_auth_dependency(default) {
                return (true, Some(auth_method));
            }
        }
    }
    
    (false, None)
}

fn is_auth_decorator(name: &str) -> bool {
    let auth_decorators = [
        "requires_auth", "authenticated", "login_required",
        "require_auth", "auth_required", "protected",
    ];
    
    auth_decorators.iter().any(|&d| name == d)
}

fn check_auth_dependency(expr: &Expression) -> Option<String> {
    if let Expression::Call { func, args } = expr {
        if let Expression::Name { id } = func.as_ref() {
            // Check for Depends(...)
            if id == "Depends" {
                if let Some(Expression::Name { id: dep_func }) = args.first() {
                    // Check if dependency function suggests authentication
                    let func_name = dep_func.to_lowercase();
                    if func_name.contains("auth")
                        || func_name.contains("user")
                        || func_name.contains("token")
                        || func_name.contains("verify")
                    {
                        return Some("dependency_injection".to_string());
                    }
                }
            }
            
            // Check for Security(...)
            if id == "Security" {
                return Some("security_scheme".to_string());
            }
        }
    }
    None
}

/// Check if all routes have authentication
fn check_all_routes_authenticated(routes: &[Route]) -> CheckResult {
    if routes.is_empty() {
        return CheckResult::Pass { Pass: None };
    }
    
    let unauth_routes: Vec<&Route> = routes
        .iter()
        .filter(|r| !r.authenticated)
        .collect();
    
    if unauth_routes.is_empty() {
        CheckResult::Pass { Pass: None }
    } else {
        let route_list: Vec<String> = unauth_routes
            .iter()
            .map(|r| format!("{} {} (line {})", r.method, r.path, r.line_number))
            .collect();
        
        CheckResult::Fail {
            Fail: format!(
                "Found {} unauthenticated routes: {}",
                unauth_routes.len(),
                route_list.join(", ")
            ),
        }
    }
}

/// Check if routes of a specific method have authentication
fn check_method_routes_authenticated(routes: &[Route], method: &str) -> CheckResult {
    let method_routes: Vec<&Route> = routes
        .iter()
        .filter(|r| r.method == method)
        .collect();
    
    if method_routes.is_empty() {
        return CheckResult::Pass { Pass: None };
    }
    
    let unauth_routes: Vec<&Route> = method_routes
        .iter()
        .filter(|r| !r.authenticated)
        .copied()
        .collect();
    
    if unauth_routes.is_empty() {
        CheckResult::Pass { Pass: None }
    } else {
        let route_list: Vec<String> = unauth_routes
            .iter()
            .map(|r| format!("{} (line {})", r.path, r.line_number))
            .collect();
        
        CheckResult::Fail {
            Fail: format!(
                "Found {} unauthenticated {} routes: {}",
                unauth_routes.len(),
                method,
                route_list.join(", ")
            ),
        }
    }
}

// ============================================================================
// WASM Memory Management and Exports
// ============================================================================

#[no_mangle]
pub extern "C" fn alloc(size: i32) -> *mut u8 {
    let layout = Layout::from_size_align(size as usize, 1).unwrap();
    unsafe { alloc(layout) }
}

#[no_mangle]
pub extern "C" fn dealloc_ptr(ptr: i32, size: i32) {
    let layout = Layout::from_size_align(size as usize, 1).unwrap();
    unsafe { dealloc(ptr as *mut u8, layout) }
}

fn read_string_from_memory(ptr: i32, len: i32) -> String {
    let bytes = unsafe { slice::from_raw_parts(ptr as *const u8, len as usize) };
    String::from_utf8_lossy(bytes).to_string()
}

fn write_string_to_memory(s: &str) -> (i32, i32) {
    let bytes = s.as_bytes();
    let ptr = alloc(bytes.len() as i32);
    unsafe {
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), ptr, bytes.len());
    }
    (ptr as i32, bytes.len() as i32)
}

// ============================================================================
// WASM Exported Functions
// ============================================================================

#[no_mangle]
pub extern "C" fn check_all_routes_authenticated(ast_json_ptr: i32, ast_json_len: i32) -> (i32, i32) {
    let ast_json = read_string_from_memory(ast_json_ptr, ast_json_len);
    
    let result = match serde_json::from_str::<Module>(&ast_json) {
        Ok(module) => {
            let routes = analyze_module(module);
            check_all_routes_authenticated(&routes)
        }
        Err(e) => CheckResult::Error {
            Error: format!("Failed to parse AST JSON: {}", e),
        },
    };
    
    let result_json = serde_json::to_string(&result).unwrap_or_else(|_| {
        r#"{"Error": "Failed to serialize result"}"#.to_string()
    });
    
    write_string_to_memory(&result_json)
}

#[no_mangle]
pub extern "C" fn check_post_routes_authenticated(ast_json_ptr: i32, ast_json_len: i32) -> (i32, i32) {
    let ast_json = read_string_from_memory(ast_json_ptr, ast_json_len);
    
    let result = match serde_json::from_str::<Module>(&ast_json) {
        Ok(module) => {
            let routes = analyze_module(module);
            check_method_routes_authenticated(&routes, "POST")
        }
        Err(e) => CheckResult::Error {
            Error: format!("Failed to parse AST JSON: {}", e),
        },
    };
    
    let result_json = serde_json::to_string(&result).unwrap_or_else(|_| {
        r#"{"Error": "Failed to serialize result"}"#.to_string()
    });
    
    write_string_to_memory(&result_json)
}

#[no_mangle]
pub extern "C" fn check_get_routes_authenticated(ast_json_ptr: i32, ast_json_len: i32) -> (i32, i32) {
    let ast_json = read_string_from_memory(ast_json_ptr, ast_json_len);
    
    let result = match serde_json::from_str::<Module>(&ast_json) {
        Ok(module) => {
            let routes = analyze_module(module);
            check_method_routes_authenticated(&routes, "GET")
        }
        Err(e) => CheckResult::Error {
            Error: format!("Failed to parse AST JSON: {}", e),
        },
    };
    
    let result_json = serde_json::to_string(&result).unwrap_or_else(|_| {
        r#"{"Error": "Failed to serialize result"}"#.to_string()
    });
    
    write_string_to_memory(&result_json)
}

#[no_mangle]
pub extern "C" fn get_routes_info(ast_json_ptr: i32, ast_json_len: i32) -> (i32, i32) {
    let ast_json = read_string_from_memory(ast_json_ptr, ast_json_len);
    
    let result = match serde_json::from_str::<Module>(&ast_json) {
        Ok(module) => {
            let routes = analyze_module(module);
            serde_json::to_string(&routes).unwrap_or_else(|_| "[]".to_string())
        }
        Err(e) => {
            format!(r#"{{"error": "Failed to parse AST JSON: {}"}}"#, e)
        }
    };
    
    write_string_to_memory(&result)
}
