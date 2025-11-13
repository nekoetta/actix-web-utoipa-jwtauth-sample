# actix-web API Server

A REST API server built with Rust, featuring LDAP authentication, JWT-based authentication, and OpenTelemetry observability support.

- [actix-web API Server](#actix-web-api-server)
  - [Architecture Overview](#architecture-overview)
  - [Technology Stack](#technology-stack)
  - [Key Features](#key-features)
  - [Module Structure](#module-structure)
  - [Environment Variables](#environment-variables)
  - [Getting Started (Docker)](#getting-started-docker)
  - [Getting Started (Without Docker)](#getting-started-without-docker)
  - [Development](#development)
  - [Running Tests](#running-tests)
  - [Generating OpenAPI Specification](#generating-openapi-specification)
  - [Development Guidelines](#development-guidelines)
  - [Troubleshooting](#troubleshooting)
  - [TODO](#todo)

## Architecture Overview

This API server adopts a layered architecture.

```
┌─────────────────────────────────────────┐
│      Presentation Layer                 │
│  (Actix-web Handlers + Middleware)      │
│  - JWT Authentication Middleware        │
│  - OpenTelemetry Tracing                │
│  - CORS Configuration                   │
├─────────────────────────────────────────┤
│      Application Layer                  │
│     (Use Cases + Validation)            │
│  - Business Logic                       │
│  - Input Validation                     │
├─────────────────────────────────────────┤
│         Domain Layer                    │
│        (Models + Traits)                │
│  - Domain Model Definitions             │
│  - Common Traits                        │
├─────────────────────────────────────────┤
│      Infrastructure Layer               │
│  (Diesel + LDAP + OpenTelemetry)        │
│  - Database Access                      │
│  - LDAP Authentication                  │
│  - Telemetry Export                     │
└─────────────────────────────────────────┘
```

### Authentication Flow

```
Client → POST /login → LDAP Auth → JWT Issue → Client
                          ↓
                    Save User Info to DB
                          
Client → GET /api/* → JWT Validation → Handler Execution
                        ↓
                   401 Unauthorized (on failure)
```

### Data Flow

```
HTTP Request
    ↓
Middleware (JWT Auth, Tracing)
    ↓
Handler (services/api/*.rs)
    ↓
Use Case (models/*/usecases.rs)
    ↓
Diesel ORM
    ↓
PostgreSQL
```

## Technology Stack

### Core Technologies

- **Language**: Rust (Edition 2021)
- **Web Framework**: Actix-web 4.x
  - High-performance async web framework
  - Authentication and logging via middleware
- **ORM**: Diesel 2.0
  - Type-safe query builder
  - Migration management
- **Database**: PostgreSQL
  - Connection pool management with r2d2

### Authentication & Security

- **JWT**: jsonwebtoken
  - Token signing with HS256 algorithm
  - 7-day expiration
- **LDAP**: ldap3
  - Active Directory integration
  - Simple Bind authentication
- **Validation**: validator 0.16
  - Declarative validation rules

### API Specification & Documentation

- **OpenAPI**: utoipa 3.x
  - Auto-generate OpenAPI 3.0 spec from code
  - Interactive documentation with Swagger UI
- **Swagger UI**: utoipa-swagger-ui
  - Accessible at /swagger-ui/

### Observability

- **OpenTelemetry**: opentelemetry, opentelemetry-otlp
  - Distributed tracing
  - Metrics collection
  - Export in OTLP format
- **Logging**: tracing, tracing-subscriber
  - Structured logging
  - OpenTelemetry integration

### Development & Testing

- **Testing**: cargo test
  - Unit tests
  - Integration tests (tests/)
- **Hot Reload**: cargo-watch
  - Auto-restart on file changes

## Key Features

### Authentication

- **LDAP Authentication**: Active Directory integration
- **JWT Authentication**: Stateless token-based authentication
- **Group Filtering**: Deny login for specific groups (Partner)

### API Features

- **User Management**: Retrieve user list
- **Customer Category Management**: CRUD operations
- **Validation**: Input data validation
- **Error Handling**: Unified error responses

### Observability

- **Distributed Tracing**: Trace HTTP requests, DB queries, authentication
- **Metrics**: Request count, response time, error rate
- **Structured Logging**: JSON-formatted logs

## Module Structure

| Tree                              | Description                                                                                           |
| --------------------------------- | ----------------------------------------------------------------------------------------------------- |
| ── src                            |                                                                                                       |
| ├── bin                           |                                                                                                       |
| │  └── generate_openapi_schema.rs | # Binary for generating OpenAPI specification                                                         |
| ├── config.rs                     | # Deserialize environment variables and .env file into Config struct                                  |
| ├── errors.rs                     | # Manage API errors                                                                                   |
| ├── lib.rs                        | # Top-level library module for DB connection setup                                                    |
| ├── main.rs                       | # Top-level module to start actix-web server                                                          |
| ├── middleware.rs                 | # Define middleware such as JWT authentication                                                        |
| │── models                        | # Place modules under models                                                                          |
| │  ├── users                      | # Place modules under each model (e.g., users)                                                        |
| │  │  └── usecases.rs             | # Define minimal structs and methods for DB access (get, insert, etc.)                                |
| │  └── users.rs                   | # Define struct for each model (all columns)                                                          |
| ├── models.rs                     | # Declare modules under models (each model)                                                           |
| ├── schema.rs                     | # Auto-generated DB schema by diesel migration run                                                    |
| ├── services                      | # Declare modules for access points                                                                   |
| │  ├── api                        | # Define authenticated endpoints under api module                                                     |
| │  ├── api.rs                     | # Add authentication middleware and aggregate endpoint definitions under api directory                |
| │  └── auth.rs                    | # Define login-related endpoints (no authentication required)                                         |
| ├── services.rs                   | # Declare modules for each scope of access points                                                     |
| ├── swagger.rs                    | # Define swagger UI access point                                                                      |
| └── traits.rs                     | # Define traits                                                                                       |
| ── tests                          | # Place integration tests                                                                             |

## Environment Variables

Set the following values in a .env file or as runtime environment variables:

### Database Configuration

- DATABASE_URL
  - Example: DATABASE_URL=postgres://postgres:P%40ssw0rd@localhost/development
- TEST_DATABASE_URL
  - Example: TEST_DATABASE_URL=postgres://postgres:P%40ssw0rd@localhost:5433/test

### Authentication Configuration

- JWT_SECRET
  - Example: JWT_SECRET="18 A6 77 73 7F 72 44 6C 26 84 0B 19 75 E0 07 FA 73 A4 A8 82 21 C7 99 AC 0D C6 A5 FE D0 E4 E0 E6"
- LDAP_URI=ldap://ad.example.com
- LDAP_UID_COLUMN=cn
- LDAP_FILTER="(objectCategory=CN=Person*)"
- LDAP_USER_DN="cn=users,dc=example,dc=com"
- LDAP_GUARD_FILTER="(objectCategory=CN=Group*)"

### OpenTelemetry Configuration (Optional)

Enable distributed tracing and metrics collection with OpenTelemetry.

- OTEL_ENABLED
  - Enable/disable OpenTelemetry
  - Default: false
  - Example: OTEL_ENABLED=true
  
- OTEL_ENDPOINT
  - OTLP exporter endpoint URL
  - Default: http://localhost:4317
  - Example: OTEL_ENDPOINT=http://jaeger:4317
  
- OTEL_SERVICE_NAME
  - Service name (displayed in traces)
  - Default: rust-api
  - Example: OTEL_SERVICE_NAME=my-api-server
  
- OTEL_SERVICE_VERSION
  - Service version
  - Default: 0.1.0
  - Example: OTEL_SERVICE_VERSION=1.0.0

#### OpenTelemetry Backend Configuration Examples

##### Using Jaeger

Start Jaeger with docker-compose:

```yaml
services:
  jaeger:
    image: jaegertracing/all-in-one:latest
    ports:
      - "16686:16686"  # Jaeger UI
      - "4317:4317"    # OTLP gRPC receiver
    environment:
      - COLLECTOR_OTLP_ENABLED=true
```

Environment variable configuration:
```bash
OTEL_ENABLED=true
OTEL_ENDPOINT=http://localhost:4317
OTEL_SERVICE_NAME=rust-api
```

Access Jaeger UI: http://localhost:16686

##### Using Grafana Tempo

Start Tempo with docker-compose:

```yaml
services:
  tempo:
    image: grafana/tempo:latest
    command: [ "-config.file=/etc/tempo.yaml" ]
    volumes:
      - ./tempo.yaml:/etc/tempo.yaml
    ports:
      - "4317:4317"    # OTLP gRPC
      - "3200:3200"    # Tempo HTTP
```

tempo.yaml configuration example:
```yaml
server:
  http_listen_port: 3200

distributor:
  receivers:
    otlp:
      protocols:
        grpc:
          endpoint: 0.0.0.0:4317

storage:
  trace:
    backend: local
    local:
      path: /tmp/tempo/traces
```

Environment variable configuration:
```bash
OTEL_ENABLED=true
OTEL_ENDPOINT=http://localhost:4317
OTEL_SERVICE_NAME=rust-api
```

##### Using OpenTelemetry Collector

For more flexible configuration, use OpenTelemetry Collector:

```yaml
services:
  otel-collector:
    image: otel/opentelemetry-collector:latest
    command: ["--config=/etc/otel-collector-config.yaml"]
    volumes:
      - ./otel-collector-config.yaml:/etc/otel-collector-config.yaml
    ports:
      - "4317:4317"    # OTLP gRPC
      - "4318:4318"    # OTLP HTTP
```

Environment variable configuration:
```bash
OTEL_ENABLED=true
OTEL_ENDPOINT=http://localhost:4317
OTEL_SERVICE_NAME=rust-api
```

## Getting Started (Docker)

1. Ensure Docker Desktop or docker/docker-compose is available
2. Configure environment variables (create .env file)
3. Start with the following command:
    ```bash
    docker compose up -d
    ```

## Getting Started (Without Docker)

1. Install Rust: <https://www.rust-lang.org/tools/install>
2. Prepare a PostgreSQL server running on localhost:5432 (or use docker-compose)
3. Install diesel_cli:
    ```bash
    cargo install diesel_cli --no-default-features --features postgres
    ```
4. Initialize diesel:
    ```bash
    diesel setup
    ```
5. Run migrations:
    ```bash
    diesel migration run
    ```
6. Install cargo-watch for HMR:
    ```bash
    cargo install cargo-watch
    ```
7. Start the actix-web server:
    ```bash
    RUST_BACKTRACE=1 RUST_LOG=debug cargo watch -x run
    ```

## Development

1. Use VSCode
2. Install the rust-analyzer extension
3. When adding API definitions, update utoipa schema definitions and reflect in swagger.rs
4. Access [http://localhost:8080/swagger-ui/] to verify API functionality by making actual requests
5. Don't forget to [generate OpenAPI specification](#generating-openapi-specification)

## Running Tests

### Prerequisites

Before running tests, start the test PostgreSQL database:

```bash
# Start test database with Docker
docker compose -f docker-compose.test.yml up -d

# Verify database is running
docker ps | grep rust-api-test-db
```

### Basic Test Execution

Run all tests:
```bash
cargo test
```

Run tests with detailed logs:
```bash
RUST_BACKTRACE=1 RUST_LOG=debug cargo test
```

Run specific test file:
```bash
# JWT authentication tests
cargo test --test jwt_auth

# Customer category tests
cargo test --test customer_categories

# User error case tests
cargo test --test users_error_cases

# Authentication error case tests
cargo test --test auth_error_cases

# LDAP mock tests
cargo test --test ldap_mock_tests
```

Run specific test function:
```bash
cargo test test_jwt_auth_wrapper -- --nocapture
```

### Test Coverage Measurement

#### Initial Setup

```bash
# Install cargo-llvm-cov
cargo install cargo-llvm-cov

# Install LLVM tools
rustup component add llvm-tools-preview
```

#### Generate Coverage Report

Display coverage in text format:
```bash
cargo llvm-cov --test jwt_auth --test customer_categories --test users_error_cases --test auth_error_cases --test ldap_mock_tests -- --test-threads=1
```

Generate HTML coverage report:
```bash
cargo llvm-cov --test jwt_auth --test customer_categories --test users_error_cases --test auth_error_cases --test ldap_mock_tests --html -- --test-threads=1
```

HTML report is generated at `target/llvm-cov/html/index.html`. Open in browser:
```bash
# Linux
xdg-open target/llvm-cov/html/index.html

# macOS
open target/llvm-cov/html/index.html

# Windows
start target/llvm-cov/html/index.html
```

Output coverage in JSON format (for CI/CD):
```bash
cargo llvm-cov --test jwt_auth --test customer_categories --test users_error_cases --test auth_error_cases --test ldap_mock_tests --json --output-path coverage.json -- --test-threads=1
```

Output coverage in LCOV format (for integration with other tools):
```bash
cargo llvm-cov --test jwt_auth --test customer_categories --test users_error_cases --test auth_error_cases --test ldap_mock_tests --lcov --output-path lcov.info -- --test-threads=1
```

### Test Suite Overview

This project includes the following test suites (42 tests total):

| Test Suite | Test Count | Description |
|------------|-----------|-------------|
| **jwt_auth** | 11 | JWT authentication middleware, tracing middleware, request data creation tests |
| **customer_categories** | 8 | Customer category API error cases, validation, authentication tests |
| **users_error_cases** | 6 | User API error cases, pagination tests |
| **auth_error_cases** | 7 | Authentication endpoint validation, error handling tests |
| **ldap_mock_tests** | 10 | LDAP authentication logic, JWT generation, user creation flow tests |

### Coverage Goals

Current coverage status:

- **Overall Coverage**: 58.18%
- **Middleware**: 87.29% ✅
- **Data Models**: 100% ✅
- **API Endpoints**: 73-83% ✅
- **Authentication Service**: 27.56% ⚠️ (due to LDAP dependency)

See `test-coverage-report.md` for detailed coverage report.

### Testing Best Practices

#### 1. Database Tests

Tests automatically run migrations and use a fresh database state:

```rust
#[actix_web::test]
async fn test_insert_category() {
    let pool = rust_api::create_test_connection_pool();
    // Test code
}
```

#### 2. Testing Authenticated Endpoints

Generate JWT token for testing:

```rust
fn create_valid_token() -> String {
    let claims = UserClaims {
        id: 1,
        username: "testuser".into(),
        exp: (chrono::Utc::now() + chrono::Duration::days(7)).timestamp()
    };
    // Generate token
}

#[actix_web::test]
async fn test_protected_endpoint() {
    let token = create_valid_token();
    let req = test::TestRequest::get()
        .uri("/api/protected")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();
    // Execute test
}
```

#### 3. Testing Error Cases

Test various error cases including validation, authentication, and database errors:

```rust
#[actix_web::test]
async fn test_validation_error() {
    let data = NewCategoryBody {
        name: "a".repeat(256), // Name too long
    };
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status().as_u16(), 400); // Bad Request
}
```

### Troubleshooting Tests

#### Cannot connect to test database

```bash
# Check if database is running
docker ps | grep rust-api-test-db

# Restart database
docker compose -f docker-compose.test.yml down
docker compose -f docker-compose.test.yml up -d

# Test connection
psql postgres://test:test@localhost:5432/test_db -c "SELECT 1"
```

#### Migration errors

```bash
# Reset test database
docker compose -f docker-compose.test.yml down -v
docker compose -f docker-compose.test.yml up -d
```

#### Specific test failures

```bash
# Run with detailed logs
RUST_BACKTRACE=1 RUST_LOG=debug cargo test test_name -- --nocapture

# Run single-threaded (avoid concurrency issues)
cargo test -- --test-threads=1
```

## Generating OpenAPI Specification

1. Run the following command:
    ```bash
    cargo run --bin generate_openapi_schema
    ```
2. openapi_schema.json will be generated

## Development Guidelines

### Coding Conventions

#### Naming Rules

- **Functions & Variables**: snake_case
  ```rust
  fn insert_new_user() { }
  let user_name = "example";
  ```

- **Types & Structs & Enums**: PascalCase
  ```rust
  struct User { }
  enum ServiceError { }
  ```

- **Constants**: SCREAMING_SNAKE_CASE
  ```rust
  const API_PREFIX: &str = "/api";
  ```

#### Module Organization

- **models/**: Domain models and business logic
  - `models/{entity}.rs`: Struct definitions
  - `models/{entity}/usecases.rs`: CRUD operations and business logic

- **services/**: API endpoints
  - `services/auth.rs`: Endpoints without authentication
  - `services/api/*.rs`: Endpoints requiring authentication

#### Error Handling

- Avoid using `expect()`, use `?` operator instead
  ```rust
  // ❌ Avoid
  let conn = pool.get().expect("couldn't get db connection");
  
  // ✅ Recommended
  let conn = pool.get()?;
  ```

- Use custom error type `ServiceError`
  ```rust
  pub enum ServiceError {
      InternalServerError,
      ValidationError { value: ValidationErrors }
  }
  ```

#### Validation

- Use declarative validation with `validator` crate
  ```rust
  #[derive(Validate)]
  pub struct CategoryValidator {
      #[validate(length(max = 255, message = "Category name must be 255 characters or less"))]
      pub name: String,
  }
  ```

- Implement `IntoValidator` trait
  ```rust
  impl IntoValidator<CategoryValidator> for CustomerCategory {
      fn validator(&self) -> CategoryValidator {
          CategoryValidator { name: self.name.clone() }
      }
  }
  ```

#### Database Access

- Use Diesel's type-safe query builder
  ```rust
  use crate::schema::users::dsl::*;
  
  users
      .filter(login_id.eq(user_login_id))
      .first::<User>(conn)
  ```

- Use transactions appropriately
  ```rust
  conn.transaction::<_, diesel::result::Error, _>(|conn| {
      // Multiple operations
      Ok(())
  })
  ```

### Adding OpenTelemetry Tracing

#### 1. Adding Tracing to Use Case Functions

Use `#[instrument]` attribute to trace function execution:

```rust
use tracing::instrument;

#[instrument(skip(conn), fields(db.operation = "insert_user"))]
pub fn insert_new_user(
    conn: &mut DbConnection,
    user: NewUser,
) -> QueryResult<User> {
    // Implementation
}
```

**Parameter explanation**:
- `skip(conn)`: Parameters to exclude from trace (e.g., DB connection)
- `fields(...)`: Add custom attributes

#### 2. Adding Tracing to Handlers

```rust
use tracing::{info, error};

#[utoipa::path(/* ... */)]
pub async fn get_users(pool: web::Data<DbPool>) -> Result<impl Responder, ServiceError> {
    info!("Fetching all users");
    
    let result = web::block(move || {
        let mut conn = pool.get()?;
        users::usecases::get_all_users(&mut conn)
    })
    .await?;
    
    info!("Successfully fetched {} users", result.len());
    Ok(web::Json(result))
}
```

#### 3. Tracing Errors

```rust
match some_operation() {
    Ok(result) => {
        tracing::info!("Operation succeeded");
        result
    }
    Err(e) => {
        tracing::error!("Operation failed: {:?}", e);
        return Err(ServiceError::InternalServerError);
    }
}
```

#### 4. Creating Custom Spans

For more detailed tracing:

```rust
use tracing::{info_span, Instrument};

async fn complex_operation() {
    let span = info_span!("ldap_authentication", user = %username);
    
    async {
        // LDAP authentication process
        info!("Binding to LDAP server");
        // ...
        info!("Searching user in LDAP");
        // ...
    }
    .instrument(span)
    .await
}
```

### Adding New API Endpoints

1. **Define Model** (`models/{entity}.rs`)
   ```rust
   #[derive(Queryable, Serialize, ToSchema)]
   pub struct MyEntity {
       pub id: i32,
       pub name: String,
   }
   ```

2. **Implement Use Case** (`models/{entity}/usecases.rs`)
   ```rust
   #[instrument(skip(conn))]
   pub fn get_all(conn: &mut DbConnection) -> QueryResult<Vec<MyEntity>> {
       use crate::schema::my_entities::dsl::*;
       my_entities.load::<MyEntity>(conn)
   }
   ```

3. **Implement Handler** (`services/api/{entity}.rs`)
   ```rust
   #[utoipa::path(
       get,
       path = "/api/my-entities",
       responses(
           (status = 200, description = "Success", body = [MyEntity])
       )
   )]
   pub async fn get_all(pool: web::Data<DbPool>) -> Result<impl Responder, ServiceError> {
       // Implementation
   }
   ```

4. **Add to Swagger Definition** (`swagger.rs`)
   ```rust
   #[derive(OpenApi)]
   #[openapi(
       paths(
           services::api::my_entities::get_all,
       ),
       components(schemas(MyEntity))
   )]
   struct ApiDoc;
   ```

5. **Register Routing** (`services/api.rs` or `main.rs`)
   ```rust
   .service(
       web::scope("/api")
           .service(my_entities::get_all)
   )
   ```

6. **Create Tests** (`tests/my_entities.rs`)
   ```rust
   #[actix_web::test]
   async fn test_get_all() {
       // Test implementation
   }
   ```

## Troubleshooting

### Common Issues and Solutions

#### 1. Database Connection Error

**Symptom**:
```
Error: couldn't get db connection from pool
```

**Causes**:
- PostgreSQL is not running
- DATABASE_URL is not configured correctly
- Connection pool exhausted

**Solutions**:
```bash
# Check if PostgreSQL is running
docker ps | grep postgres

# Check environment variable
echo $DATABASE_URL

# Check .env file
cat .env

# Test database connection
psql $DATABASE_URL -c "SELECT 1"
```

#### 2. Migration Error

**Symptom**:
```
Error: Migration xxx has already been run
```

**Solutions**:
```bash
# Check migration status
diesel migration list

# Revert migration
diesel migration revert

# Run migration again
diesel migration run
```

#### 3. JWT Authentication Error

**Symptom**:
```
401 Unauthorized
```

**Causes**:
- Token is invalid or expired
- JWT_SECRET is not configured correctly
- Authorization header format is incorrect

**Solutions**:
```bash
# Check JWT_SECRET
echo $JWT_SECRET

# Verify token (decode at jwt.io)
# Authorization header format: Bearer <token>

# Check logs for details
RUST_LOG=debug cargo run
```

#### 4. LDAP Authentication Error

**Symptom**:
```
LDAP bind failed
```

**Causes**:
- Cannot connect to LDAP server
- Invalid credentials
- Incorrect LDAP configuration

**Solutions**:
```bash
# Test LDAP connection
ldapsearch -H $LDAP_URI -D "cn=user,dc=example,dc=com" -W

# Check environment variables
echo $LDAP_URI
echo $LDAP_USER_DN
echo $LDAP_FILTER

# Check debug logs
RUST_LOG=debug cargo run
```

#### 5. OpenTelemetry Export Error

**Symptom**:
```
OpenTelemetry trace error occurred
```

**Causes**:
- Cannot connect to OTLP endpoint
- Jaeger/Tempo is not running

**Solutions**:
```bash
# Check if backend is running
docker ps | grep jaeger

# Check endpoint
echo $OTEL_ENDPOINT

# Disable OpenTelemetry and start
OTEL_ENABLED=false cargo run

# Check network connection
curl -v http://localhost:4317
```

#### 6. Compilation Error

**Symptom**:
```
error[E0433]: failed to resolve: use of undeclared crate or module
```

**Solutions**:
```bash
# Update dependencies
cargo update

# Clean build
cargo clean
cargo build

# Delete and regenerate Cargo.lock
rm Cargo.lock
cargo build
```

#### 7. Test Failure

**Symptom**:
```
test result: FAILED
```

**Solutions**:
```bash
# Check test database
echo $TEST_DATABASE_URL

# Reset test database
diesel database reset --database-url $TEST_DATABASE_URL

# Run tests with detailed logs
RUST_BACKTRACE=1 RUST_LOG=debug cargo test -- --nocapture

# Run specific test only
cargo test test_name -- --nocapture
```

#### 8. Performance Issues

**Symptoms**:
- Slow responses
- Timeouts

**Solutions**:
```bash
# Check traces with OpenTelemetry
# Identify slow queries in Jaeger UI

# Optimize database queries
# Check execution plan with EXPLAIN ANALYZE

# Adjust connection pool settings
# Check r2d2 configuration in Cargo.toml

# Check OpenTelemetry overhead
OTEL_ENABLED=false cargo run  # Compare with disabled
```

### Log Level Configuration

Detailed logs for development:
```bash
RUST_LOG=debug cargo run
```

Warning and above for production:
```bash
RUST_LOG=warn cargo run
```

Module-specific log levels:
```bash
RUST_LOG=actix_web=info,diesel=debug,my_app=trace cargo run
```

### Debugging Tips

1. **RUST_BACKTRACE**: Display stack trace
   ```bash
   RUST_BACKTRACE=1 cargo run
   ```

2. **cargo-expand**: Check macro expansion
   ```bash
   cargo install cargo-expand
   cargo expand
   ```

3. **Swagger UI**: Verify API functionality
   - http://localhost:8080/swagger-ui/

4. **Jaeger UI**: Check traces
   - http://localhost:16686

## OpenTelemetry Verification

For OpenTelemetry integration verification and performance testing, refer to the following documents:

### Quick Start

```bash
# Run automated test script for all verifications
./test-otel.sh

# Verify metrics collection
./verify-metrics.sh

# Performance comparison benchmark
./benchmark-otel.sh
```

### Detailed Documentation

- **[OTEL_TESTING.md](OTEL_TESTING.md)** - Comprehensive OpenTelemetry verification guide
- **[MANUAL_OTEL_TEST.md](MANUAL_OTEL_TEST.md)** - Manual testing procedures
- **[METRICS_SETUP.md](METRICS_SETUP.md)** - Metrics collection and Prometheus integration
- **[PERFORMANCE_TEST.md](PERFORMANCE_TEST.md)** - Performance testing guide
- **[OTEL_VERIFICATION_SUMMARY.md](OTEL_VERIFICATION_SUMMARY.md)** - Verification completion summary

### Key URLs

- **Jaeger UI**: http://localhost:16686 - View traces
- **Swagger UI**: http://localhost:8080/swagger-ui/ - API specification and testing
- **API Server**: http://localhost:8080

## TODO

Specify constants for tag and context_path. <https://github.com/juhaku/utoipa/issues/518>
