# FastAPI Microservices Authentication Pattern - Implementation Options

## Problem Statement

Create a Hielements library that contains a FastAPI microservices pattern that checks that POST and GET endpoints are authenticated. Ideally use WASM for safety, but also leverage existing parsing/code analysis libraries like libcst.

## Requirements Analysis

1. **Pattern Definition**: Define a reusable pattern for FastAPI microservices
2. **Authentication Checks**: Verify that POST and GET endpoints have authentication
3. **Code Analysis**: Parse and analyze Python FastAPI code
4. **Safety**: Prefer sandboxed execution (WASM) when possible
5. **Practicality**: Leverage existing Python analysis tools (libcst, ast)

## Implementation Options

### Option 1: Pure Hielements Pattern (No Code Analysis)

**Description**: Create a pattern that relies on existing Hielements libraries (files, python) without deep code analysis.

**Implementation**:
- Pattern defines structure expectations for FastAPI services
- Uses basic Python library checks (function_exists, module structure)
- Checks for presence of authentication decorators by file/folder convention
- Requires manual annotations or conventions in code structure

**Pros**:
- ✅ Simplest implementation
- ✅ No external dependencies
- ✅ Fast execution
- ✅ Works immediately with current Hielements
- ✅ Easy to understand and maintain

**Cons**:
- ❌ Cannot deeply inspect FastAPI route decorators
- ❌ Cannot verify authentication on specific endpoints
- ❌ Relies on conventions rather than actual code analysis
- ❌ Limited enforcement capabilities
- ❌ May produce false positives/negatives

**When to Use**:
- Quick architectural documentation
- High-level service structure validation
- When deep code analysis isn't required
- Greenfield projects with strict conventions

**Implementation Effort**: Low (2-4 hours)

---

### Option 2: External Python Plugin with libcst/ast

**Description**: Create an external process plugin in Python that uses libcst or ast to parse and analyze FastAPI code.

**Implementation**:
- Python plugin using libcst for CST (Concrete Syntax Tree) analysis
- Detect FastAPI route decorators (@app.get, @app.post, etc.)
- Identify authentication decorators/dependencies (Depends, Security, etc.)
- Return detailed information about authenticated vs unauthenticated routes
- Full JSON-RPC protocol implementation

**Pros**:
- ✅ Deep code analysis capabilities
- ✅ Can actually verify authentication on specific endpoints
- ✅ Leverage mature Python parsing libraries (libcst, ast)
- ✅ Production-ready (external plugins fully supported)
- ✅ Can detect various authentication patterns:
  - Decorator-based (@requires_auth)
  - Dependency injection (Depends(get_current_user))
  - FastAPI Security schemes
- ✅ Flexible and easy to extend
- ✅ Fast development iteration
- ✅ Can provide detailed error messages

**Cons**:
- ❌ Requires Python runtime
- ❌ No sandboxing (runs with full OS permissions)
- ❌ Process spawning overhead
- ❌ Potential security concerns with untrusted code
- ❌ Larger resource footprint

**When to Use**:
- Production systems requiring accurate authentication checks
- When deep FastAPI code analysis is needed
- When quick development and iteration is important
- Brownfield projects with existing FastAPI codebases

**Implementation Effort**: Medium (1-2 days)

**Technical Details**:

```python
# Example libcst visitor for FastAPI analysis
import libcst as cst

class FastAPIAuthVisitor(cst.CSTTransformer):
    def __init__(self):
        self.routes = []
        
    def visit_FunctionDef(self, node):
        # Check decorators for @app.get, @app.post, etc.
        for decorator in node.decorators:
            if self._is_route_decorator(decorator):
                has_auth = self._check_authentication(node)
                self.routes.append({
                    'name': node.name.value,
                    'method': self._get_http_method(decorator),
                    'path': self._get_route_path(decorator),
                    'authenticated': has_auth
                })
```

---

### Option 3: WASM Plugin (Future-Ready)

**Description**: Create a WASM plugin in Rust that performs FastAPI code analysis in a sandboxed environment.

**Implementation**:
- Rust plugin compiled to WASM (wasm32-unknown-unknown target)
- Use tree-sitter-python or similar for parsing
- Implement AST traversal in Rust
- Export library_call and library_check functions
- Full sandboxing via WebAssembly

**Pros**:
- ✅ Strong security sandboxing
- ✅ Near-native performance (faster than Python)
- ✅ Single .wasm file distribution
- ✅ No runtime dependencies
- ✅ Cross-platform (one file for all platforms)
- ✅ Memory-safe execution
- ✅ Controlled resource access

**Cons**:
- ❌ WASM runtime integration still in progress
- ❌ More complex development (Rust vs Python)
- ❌ Limited ecosystem for Python parsing in Rust
- ❌ Longer development time
- ❌ Harder to debug and iterate
- ❌ Cannot use libcst (Python-only)
- ❌ Not immediately usable (infrastructure ready, runtime pending)

**When to Use**:
- When WASM runtime integration is complete
- Security-critical environments
- When performance is paramount
- Distribution to untrusted users
- Long-term production deployment

**Implementation Effort**: High (3-5 days, after WASM runtime is ready)

**Technical Details**:

```rust
// Rust WASM plugin structure
use tree_sitter;
use tree_sitter_python;

#[no_mangle]
pub extern "C" fn library_check(input_ptr: i32, input_len: i32) -> (i32, i32) {
    // Parse Python code with tree-sitter
    // Traverse AST looking for FastAPI patterns
    // Check for authentication decorators
    // Return Pass/Fail result
}
```

---

### Option 4: Hybrid Approach (Recommended)

**Description**: Create both a pattern and an external Python plugin, with a migration path to WASM.

**Implementation**:
1. **Phase 1 (Immediate)**: 
   - Create FastAPI pattern definition (Option 1)
   - Implement external Python plugin with libcst (Option 2)
   - Document both approaches
   
2. **Phase 2 (Future)**:
   - When WASM runtime is ready, create WASM version (Option 3)
   - Provide migration guide
   - Keep external plugin as fallback

**Pros**:
- ✅ Best of both worlds: immediate usability + future safety
- ✅ Pattern provides architectural blueprint
- ✅ Plugin provides actual enforcement
- ✅ Clear migration path to WASM
- ✅ Flexibility for different use cases
- ✅ Demonstrates Hielements' extensibility

**Cons**:
- ❌ More initial development effort
- ❌ Need to maintain multiple implementations
- ❌ Users must choose which approach to use

**When to Use**:
- Library/framework projects
- When both immediate use and long-term security matter
- Educational purposes (showing multiple approaches)
- Demonstrating Hielements patterns best practices

**Implementation Effort**: Medium-High (2-3 days)

---

## Recommended Implementation: Option 4 (Hybrid)

### Rationale

1. **Immediate Value**: External Python plugin provides real authentication checking now
2. **Best Practices**: Pattern demonstrates proper Hielements architecture
3. **Future-Proof**: Clear path to WASM when runtime is ready
4. **Educational**: Shows the full spectrum of Hielements capabilities
5. **Practical**: Leverages libcst for accurate Python analysis

### Implementation Plan

#### 1. Create FastAPI Pattern (`patterns/infrastructure/fastapi_microservice.hie`)

```hielements
pattern fastapi_microservice {
    element api {
        scope module<python>
        ref app_instance: FastAPIApp
        ref routes: RouteDefinitions
        
        requires descendant check fastapi.all_routes_authenticated(module)
    }
    
    element auth {
        scope module<python>
        ref auth_middleware: AuthMiddleware
        ref dependencies: AuthDependencies
        
        check fastapi.has_authentication_scheme(module)
    }
    
    element models {
        scope module<python>
        ref request_models: PydanticModels
        ref response_models: PydanticModels
    }
    
    ## Container requirements
    element container {
        scope dockerfile
        ref exposed_port: integer
        
        check files.exists(container.dockerfile, 'HEALTHCHECK')
    }
}
```

#### 2. Create FastAPI Plugin (`examples/plugins/fastapi_plugin.py`)

Features:
- Parse FastAPI route decorators using libcst
- Detect authentication patterns:
  - Decorator-based: `@requires_auth`, `@authenticated`
  - Dependency injection: `Depends(get_current_user)`
  - FastAPI Security: `Security(oauth2_scheme)`
  - HTTPBearer, OAuth2PasswordBearer, etc.
- Selectors:
  - `fastapi.app_selector(path)` - Find FastAPI app instances
  - `fastapi.route_selector(app, method)` - Find routes by HTTP method
  - `fastapi.authenticated_routes(app)` - Get authenticated routes
  - `fastapi.unauthenticated_routes(app)` - Get routes without auth
- Checks:
  - `fastapi.all_routes_authenticated(scope)` - Verify all routes have auth
  - `fastapi.route_has_authentication(route, method)` - Check specific route
  - `fastapi.has_authentication_scheme(scope)` - Verify auth is configured

#### 3. Create Example Implementation

```hielements
import files
import fastapi

element payment_api implements fastapi_microservice {
    scope api_mod<python> binds fastapi_microservice.api.module = 
        fastapi.app_selector('app/api')
    
    scope auth_mod<python> binds fastapi_microservice.auth.module = 
        fastapi.module_selector('app/auth')
    
    scope models_mod<python> binds fastapi_microservice.models.module = 
        fastapi.module_selector('app/models')
    
    scope dockerfile binds fastapi_microservice.container.dockerfile = 
        files.file_selector('Dockerfile')
    
    ## Verify authentication on all endpoints
    check fastapi.all_routes_authenticated(api_mod)
    check fastapi.has_authentication_scheme(auth_mod)
}
```

#### 4. WASM Migration Path (Documentation)

Document in `doc/fastapi_wasm_migration.md`:
- When to migrate to WASM
- How to compile Rust WASM plugin
- Configuration changes needed
- Performance benchmarks
- Security benefits

---

## Authentication Detection Strategies

### 1. Decorator-Based Detection

```python
@app.post("/api/payment")
@requires_auth
async def create_payment():
    pass
```

**Detection**: Look for `@requires_auth`, `@authenticated`, `@login_required` decorators

### 2. Dependency Injection Detection

```python
@app.post("/api/payment")
async def create_payment(user: User = Depends(get_current_user)):
    pass
```

**Detection**: Look for `Depends()` with authentication functions

### 3. FastAPI Security Detection

```python
oauth2_scheme = OAuth2PasswordBearer(tokenUrl="token")

@app.post("/api/payment")
async def create_payment(token: str = Depends(oauth2_scheme)):
    pass
```

**Detection**: Look for Security(), OAuth2PasswordBearer, HTTPBearer, etc.

### 4. Middleware Detection

```python
app.add_middleware(AuthenticationMiddleware)
```

**Detection**: Check for authentication middleware registration

---

## Testing Strategy

### Unit Tests
- Test each authentication detection pattern
- Test edge cases (missing auth, partial auth, etc.)
- Test various FastAPI versions

### Integration Tests
- Test with real FastAPI projects
- Test false positive scenarios
- Test false negative scenarios

### Example Test Cases

```python
# Test 1: Detect @requires_auth decorator
test_code_1 = '''
@app.post("/api/payment")
@requires_auth
async def create_payment():
    pass
'''

# Test 2: Detect Depends(get_current_user)
test_code_2 = '''
@app.post("/api/payment")
async def create_payment(user: User = Depends(get_current_user)):
    pass
'''

# Test 3: Missing authentication (should fail)
test_code_3 = '''
@app.post("/api/payment")
async def create_payment():
    pass
'''
```

---

## Documentation Requirements

1. **Pattern Documentation**: Add to pattern catalog
2. **Plugin Documentation**: Add to library catalog
3. **Usage Guide**: How to use the pattern
4. **Migration Guide**: External → WASM migration path
5. **Example Projects**: Real-world FastAPI examples
6. **Security Best Practices**: Authentication patterns in FastAPI

---

## Timeline

### Phase 1: Pattern + External Plugin (Immediate)
- Day 1:
  - Create pattern definition
  - Implement basic plugin structure
  - Add libcst-based route detection
- Day 2:
  - Add authentication detection logic
  - Create example implementations
  - Write tests
- Day 3:
  - Documentation
  - Integration with pattern catalog
  - Polish and review

### Phase 2: WASM Implementation (Future)
- After WASM runtime integration is complete
- Week 1: Rust plugin development
- Week 2: Testing and optimization
- Week 3: Documentation and migration guide

---

## Success Criteria

1. ✅ Pattern can be imported and implemented in .hie files
2. ✅ Plugin accurately detects authenticated FastAPI routes
3. ✅ Plugin correctly identifies unauthenticated routes
4. ✅ False positive rate < 5%
5. ✅ False negative rate < 2%
6. ✅ Documentation is clear and comprehensive
7. ✅ Example implementations work correctly
8. ✅ Tests pass with 100% coverage
9. ✅ Pattern appears in generated pattern catalog
10. ✅ Plugin appears in generated library catalog

---

## Conclusion

**Recommended Approach**: Option 4 (Hybrid)

Implement both a Hielements pattern and an external Python plugin with libcst. This provides:
- Immediate usability with accurate authentication checking
- Best practices demonstration
- Clear architecture via patterns
- Migration path to WASM when ready

The external Python plugin is production-ready and provides the deep code analysis needed for accurate authentication verification, while the pattern provides architectural guidance and documentation. When WASM runtime integration is complete, we can add a WASM version for enhanced security without breaking existing users.
