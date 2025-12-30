//! Sample WASM plugin for Hielements
//!
//! This is a simple example of a Hielements library compiled to WebAssembly.
//! It demonstrates how to create selectors and checks that run in a sandboxed environment.

use serde_json::{json, Value};
use std::alloc::{alloc, dealloc, Layout};
use std::slice;
use std::str;

/// Allocate memory for passing strings to/from WASM
#[no_mangle]
pub extern "C" fn allocate(size: i32) -> *mut u8 {
    let layout = Layout::from_size_align(size as usize, 1).unwrap();
    unsafe { alloc(layout) }
}

/// Deallocate memory
#[no_mangle]
pub extern "C" fn deallocate(ptr: *mut u8, size: i32) {
    let layout = Layout::from_size_align(size as usize, 1).unwrap();
    unsafe { dealloc(ptr, layout) };
}

/// Library call function - handles selectors
#[no_mangle]
pub extern "C" fn library_call(ptr: *const u8, len: i32) -> (i32, i32) {
    let input = unsafe {
        let bytes = slice::from_raw_parts(ptr, len as usize);
        str::from_utf8_unchecked(bytes)
    };
    
    let result = handle_call(input);
    string_to_ptr(&result)
}

/// Library check function - handles checks
#[no_mangle]
pub extern "C" fn library_check(ptr: *const u8, len: i32) -> (i32, i32) {
    let input = unsafe {
        let bytes = slice::from_raw_parts(ptr, len as usize);
        str::from_utf8_unchecked(bytes)
    };
    
    let result = handle_check(input);
    string_to_ptr(&result)
}

/// Convert a string to a pointer and length pair
fn string_to_ptr(s: &str) -> (i32, i32) {
    let bytes = s.as_bytes();
    let len = bytes.len() as i32;
    let ptr = allocate(len);
    unsafe {
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), ptr, bytes.len());
    }
    (ptr as i32, len)
}

/// Handle a library call (selector)
fn handle_call(input: &str) -> String {
    let request: Value = match serde_json::from_str(input) {
        Ok(v) => v,
        Err(e) => return json!({"Error": format!("Invalid JSON: {}", e)}).to_string(),
    };
    
    let function = request["function"].as_str().unwrap_or("");
    let args = request["args"].as_array().unwrap_or(&vec![]);
    let workspace = request["workspace"].as_str().unwrap_or(".");
    
    match function {
        "simple_selector" => {
            // Return a simple scope
            let path = args.get(0)
                .and_then(|v| v.get("String"))
                .and_then(|v| v.as_str())
                .unwrap_or("src");
            
            let result = json!({
                "Scope": {
                    "kind": {"Folder": path},
                    "paths": [format!("{}/{}", workspace, path)],
                    "resolved": true
                }
            });
            result.to_string()
        }
        "echo_selector" => {
            // Echo back the first argument as a string value
            let arg = args.get(0).cloned().unwrap_or(json!(null));
            json!({"String": format!("Echo: {:?}", arg)}).to_string()
        }
        _ => json!({"Error": format!("Unknown function: {}", function)}).to_string(),
    }
}

/// Handle a library check
fn handle_check(input: &str) -> String {
    let request: Value = match serde_json::from_str(input) {
        Ok(v) => v,
        Err(e) => return json!({"Error": format!("Invalid JSON: {}", e)}).to_string(),
    };
    
    let function = request["function"].as_str().unwrap_or("");
    let args = request["args"].as_array().unwrap_or(&vec![]);
    
    match function {
        "always_pass" => {
            json!({"Pass": null}).to_string()
        }
        "always_fail" => {
            let message = args.get(0)
                .and_then(|v| v.get("String"))
                .and_then(|v| v.as_str())
                .unwrap_or("Check failed");
            json!({"Fail": message}).to_string()
        }
        "check_scope_size" => {
            // Check if scope has fewer than N files
            let scope = args.get(0).and_then(|v| v.get("Scope"));
            let max_size = args.get(1)
                .and_then(|v| v.get("Int"))
                .and_then(|v| v.as_i64())
                .unwrap_or(100);
            
            if let Some(scope_obj) = scope {
                let paths = scope_obj.get("paths")
                    .and_then(|v| v.as_array())
                    .map(|a| a.len())
                    .unwrap_or(0);
                
                if paths <= max_size as usize {
                    json!({"Pass": null}).to_string()
                } else {
                    json!({"Fail": format!("Scope has {} files, expected <= {}", paths, max_size)}).to_string()
                }
            } else {
                json!({"Error": "First argument must be a scope"}).to_string()
            }
        }
        _ => json!({"Error": format!("Unknown check: {}", function)}).to_string(),
    }
}
