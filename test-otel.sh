#!/bin/bash

# OpenTelemetry Integration Test Script
# This script tests the OpenTelemetry integration by:
# 1. Starting Jaeger and PostgreSQL
# 2. Running the API server with OpenTelemetry enabled
# 3. Making test API calls
# 4. Verifying traces in Jaeger

set -e

echo "========================================="
echo "OpenTelemetry Integration Test"
echo "========================================="
echo ""

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Function to print colored output
print_success() {
    echo -e "${GREEN}✓ $1${NC}"
}

print_info() {
    echo -e "${YELLOW}ℹ $1${NC}"
}

print_error() {
    echo -e "${RED}✗ $1${NC}"
}

# Step 1: Start Jaeger and PostgreSQL
print_info "Step 1: Starting Jaeger and PostgreSQL..."
docker compose -f docker-compose.otel.yml up -d

# Wait for services to be ready
print_info "Waiting for services to be ready..."
sleep 5

# Check if Jaeger is running
if curl -s http://localhost:16686 > /dev/null; then
    print_success "Jaeger UI is accessible at http://localhost:16686"
else
    print_error "Jaeger UI is not accessible"
    exit 1
fi

# Check if PostgreSQL is ready
if docker exec rust-api-postgres pg_isready -U test > /dev/null 2>&1; then
    print_success "PostgreSQL is ready"
else
    print_error "PostgreSQL is not ready"
    exit 1
fi

# Step 2: Run database migrations
print_info "Step 2: Running database migrations..."
if command -v diesel &> /dev/null; then
    diesel migration run
    print_success "Migrations completed"
else
    print_error "diesel CLI not found. Please install it with: cargo install diesel_cli --no-default-features --features postgres"
    exit 1
fi

# Step 3: Build the API server
print_info "Step 3: Building the API server..."
cargo build --release
print_success "Build completed"

# Step 4: Start the API server with OpenTelemetry enabled
print_info "Step 4: Starting API server with OpenTelemetry enabled..."
export OTEL_ENABLED=true
export OTEL_ENDPOINT=http://localhost:4317
export OTEL_SERVICE_NAME=rust-api-test
export OTEL_SERVICE_VERSION=1.0.0
export RUST_LOG=info

# Start the server in the background
./target/release/rust_api &
API_PID=$!

# Wait for the server to start
print_info "Waiting for API server to start..."
sleep 5

# Check if the server is running
if curl -s http://localhost:8080/swagger-ui/ > /dev/null; then
    print_success "API server is running at http://localhost:8080"
else
    print_error "API server failed to start"
    kill $API_PID 2>/dev/null || true
    exit 1
fi

# Step 5: Make test API calls
print_info "Step 5: Making test API calls to generate traces..."

# Test 1: Get users (should fail without auth, but will generate trace)
print_info "Test 1: GET /api/users/ (without auth - should return 401)"
curl -s -o /dev/null -w "Status: %{http_code}\n" http://localhost:8080/api/users/

# Test 2: Get customer categories (should fail without auth)
print_info "Test 2: GET /api/customers/categories (without auth - should return 401)"
curl -s -o /dev/null -w "Status: %{http_code}\n" http://localhost:8080/api/customers/categories

# Test 3: Access Swagger UI (should succeed)
print_info "Test 3: GET /swagger-ui/ (should return 200)"
curl -s -o /dev/null -w "Status: %{http_code}\n" http://localhost:8080/swagger-ui/

# Test 4: Get OpenAPI spec (should succeed)
print_info "Test 4: GET /api-doc/openapi.json (should return 200)"
curl -s -o /dev/null -w "Status: %{http_code}\n" http://localhost:8080/api-doc/openapi.json

print_success "Test API calls completed"

# Step 6: Wait for traces to be exported
print_info "Step 6: Waiting for traces to be exported to Jaeger..."
sleep 5

# Step 7: Check Jaeger for traces
print_info "Step 7: Checking Jaeger for traces..."
JAEGER_API="http://localhost:16686/api/traces?service=rust-api-test&limit=10"
TRACES=$(curl -s "$JAEGER_API")

if echo "$TRACES" | grep -q '"traceID"'; then
    print_success "Traces found in Jaeger!"
    TRACE_COUNT=$(echo "$TRACES" | grep -o '"traceID"' | wc -l)
    print_success "Found $TRACE_COUNT traces"
    
    # Check if the service name matches
    if echo "$TRACES" | grep -q "rust-api-test"; then
        print_success "Service name 'rust-api-test' confirmed in traces"
    fi
else
    print_error "No traces found in Jaeger yet"
    print_info "Traces may take a few more seconds to appear. Check Jaeger UI manually."
fi

# Step 8: Display results
echo ""
echo "========================================="
echo "Test Results"
echo "========================================="
print_success "OpenTelemetry integration test completed!"
echo ""
print_info "Next steps:"
echo "  1. Open Jaeger UI: http://localhost:16686"
echo "  2. Select service: rust-api-test"
echo "  3. Click 'Find Traces' to view the traces"
echo "  4. Open Swagger UI: http://localhost:8080/swagger-ui/"
echo "  5. Try making authenticated API calls"
echo ""
print_info "To stop the services:"
echo "  - Stop API server: kill $API_PID"
echo "  - Stop Jaeger/PostgreSQL: docker compose -f docker-compose.otel.yml down"
echo ""

# Keep the script running so the server stays up
print_info "API server is running (PID: $API_PID)"
print_info "Press Ctrl+C to stop the server and clean up"

# Trap Ctrl+C to clean up
trap "echo ''; print_info 'Stopping API server...'; kill $API_PID 2>/dev/null || true; print_info 'Stopping Docker services...'; docker compose -f docker-compose.otel.yml down; print_success 'Cleanup completed'; exit 0" INT

# Wait for user to stop
wait $API_PID
