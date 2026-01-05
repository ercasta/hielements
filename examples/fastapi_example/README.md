# FastAPI Microservice Example with Authentication

This example demonstrates a complete FastAPI microservice with authentication checking using both:
1. **External Python plugin** with libcst (immediate, production-ready)
2. **WASM plugin** (future, when runtime integration is complete)

## Architecture

```
app/
├── api/
│   └── payments.py      # FastAPI routes (all authenticated)
├── auth/
│   └── __init__.py      # Authentication logic
└── models/
    └── __init__.py      # Pydantic models

payment_api.hie          # Hielements specification
Dockerfile               # Container definition
requirements.txt         # Python dependencies
```

## Features

- ✅ FastAPI application with RESTful API
- ✅ JWT-based authentication
- ✅ OAuth2 password bearer scheme
- ✅ Dependency injection for auth
- ✅ Pydantic models for request/response validation
- ✅ Health check endpoints
- ✅ Containerized with Docker

## API Endpoints

### Authenticated Endpoints

All these endpoints require a valid JWT token:

- `GET /api/payments` - List all payments
- `GET /api/payments/{payment_id}` - Get payment by ID
- `POST /api/payments` - Create new payment
- `POST /api/payments/{payment_id}/cancel` - Cancel payment

### Public Endpoints

- `GET /` - Root/status endpoint
- `GET /health/liveness` - Liveness probe
- `GET /health/readiness` - Readiness probe

## Authentication

All API endpoints use **dependency injection** for authentication:

```python
@app.post("/api/payments")
async def create_payment(
    payment: PaymentRequest,
    current_user: User = Depends(get_current_user)  # ← Authentication
):
    # Only authenticated users can access this
    pass
```

The `get_current_user` dependency:
1. Extracts JWT token from Authorization header
2. Validates token signature
3. Returns User object
4. Raises HTTP 401 if invalid

## Running the Example

### Option 1: Direct Python

```bash
# Install dependencies
pip install -r requirements.txt

# Run server
cd app
python -m uvicorn api.payments:app --reload --port 8080
```

### Option 2: Docker

```bash
# Build container
docker build -t payment-api .

# Run container
docker run -p 8080:8080 payment-api
```

### Option 3: Docker Compose

```bash
docker-compose up
```

## Testing

### Get Token

```bash
# In a real app, you'd authenticate with username/password
# For this example, we'll use a test token

TOKEN="eyJhbGc..."  # Test JWT token
```

### Test Endpoints

```bash
# Health check (no auth)
curl http://localhost:8080/health/liveness

# List payments (requires auth)
curl -H "Authorization: Bearer $TOKEN" \
     http://localhost:8080/api/payments

# Create payment (requires auth)
curl -X POST \
     -H "Authorization: Bearer $TOKEN" \
     -H "Content-Type: application/json" \
     -d '{"amount": 100.00, "currency": "USD"}' \
     http://localhost:8080/api/payments

# Try without auth (should fail with 401)
curl http://localhost:8080/api/payments
```

## Hielements Verification

### Using Built-in Python Library

```bash
# Check with basic Python library (no deep analysis)
hielements check payment_api.hie
```

This validates:
- ✅ Module structure
- ✅ FastAPI imports
- ✅ Function existence
- ✅ Dependencies between modules

### Using External Python Plugin (libcst)

Create `hielements.toml`:

```toml
[libraries]
fastapi = { executable = "python3", args = ["../../plugins/fastapi_plugin.py"] }
```

Update `payment_api.hie` to uncomment the fastapi plugin checks:

```hielements
import fastapi

element payment_service {
    element api {
        scope module<python> = python.module_selector('app/api/payments.py')
        
        # Deep authentication analysis with libcst
        check fastapi.all_routes_authenticated(module)
        check fastapi.post_routes_authenticated(module)
        check fastapi.get_routes_authenticated(module)
    }
}
```

Run check:

```bash
hielements check payment_api.hie
```

Output:
```
✓ All checks passed
  ✓ fastapi.all_routes_authenticated - All routes have authentication
  ✓ fastapi.post_routes_authenticated - All POST routes authenticated
  ✓ fastapi.get_routes_authenticated - All GET routes authenticated
```

### Using WASM Plugin (Future)

When WASM runtime integration is complete:

```toml
[libraries]
fastapi_auth = { path = "../../lib/fastapi_auth.wasm" }
```

```hielements
import python
import fastapi_auth

element payment_service {
    element api {
        scope module<python> = python.module_selector('app/api/payments.py')
        
        # WASM-based authentication analysis (sandboxed)
        check fastapi_auth.all_routes_authenticated(module)
    }
}
```

Benefits:
- 🔒 Sandboxed execution (WASM)
- ⚡ Near-native performance
- 📦 Single .wasm file (cross-platform)

## Authentication Patterns Detected

The plugins detect multiple authentication patterns:

### 1. Dependency Injection (Used in this example)

```python
@app.post("/api/payments")
async def create_payment(
    payment: PaymentRequest,
    current_user: User = Depends(get_current_user)
):
    pass
```

✅ Detected by: `Depends(get_current_user)`

### 2. Security Schemes

```python
from fastapi.security import HTTPBearer

security = HTTPBearer()

@app.post("/api/payments")
async def create_payment(
    token: str = Security(security)
):
    pass
```

✅ Detected by: `Security()` dependency

### 3. Decorator-Based

```python
@app.post("/api/payments")
@requires_auth
async def create_payment():
    pass
```

✅ Detected by: `@requires_auth` decorator

## Project Structure Validation

The `payment_api.hie` specification validates:

```
payment_service
├── api (FastAPI application)
│   ├── Module exists ✓
│   ├── Imports fastapi ✓
│   ├── Has create_payment ✓
│   ├── Has get_payment ✓
│   └── Has list_payments ✓
├── auth (Authentication logic)
│   ├── Module exists ✓
│   ├── Has get_current_user ✓
│   └── Has verify_token ✓
├── models (Data models)
│   ├── Module exists ✓
│   ├── Imports pydantic ✓
│   ├── Has PaymentRequest ✓
│   └── Has PaymentResponse ✓
├── dependencies
│   ├── requirements.txt exists ✓
│   ├── Contains fastapi ✓
│   └── Contains uvicorn ✓
└── container
    └── Dockerfile exists ✓
```

## Common Issues

### Issue: Authentication fails

**Cause**: Invalid or expired JWT token

**Solution**: Generate a new token or fix token validation logic

### Issue: Plugin not found

**Cause**: Plugin path incorrect in hielements.toml

**Solution**: Check path is relative to workspace root

### Issue: libcst import error

**Cause**: libcst not installed

**Solution**: 
```bash
pip install libcst
```

## Next Steps

1. **Add more tests**: Create pytest tests for authentication
2. **Add database**: Connect to PostgreSQL/MongoDB
3. **Add caching**: Use Redis for token caching
4. **Add rate limiting**: Protect against abuse
5. **Add logging**: Structured logging with correlation IDs
6. **Add metrics**: Prometheus metrics for observability

## References

- [FastAPI Documentation](https://fastapi.tiangolo.com/)
- [FastAPI Security](https://fastapi.tiangolo.com/tutorial/security/)
- [OAuth2 with Password](https://fastapi.tiangolo.com/tutorial/security/oauth2-jwt/)
- [Hielements Documentation](../../doc/language_reference.md)
- [External Libraries Guide](../../doc/external_libraries.md)
- [WASM Plugins Guide](../../doc/wasm_plugins.md)

## License

MIT
