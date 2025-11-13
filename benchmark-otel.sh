#!/bin/bash

# OpenTelemetry Performance Benchmark Script
# Compares performance with and without OpenTelemetry

set -e

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m'

print_header() {
    echo -e "${BLUE}=========================================${NC}"
    echo -e "${BLUE}$1${NC}"
    echo -e "${BLUE}=========================================${NC}"
}

print_success() {
    echo -e "${GREEN}✓ $1${NC}"
}

print_info() {
    echo -e "${YELLOW}ℹ $1${NC}"
}

print_error() {
    echo -e "${RED}✗ $1${NC}"
}

# Check if required tools are installed
check_requirements() {
    print_info "Checking requirements..."
    
    if ! command -v cargo &> /dev/null; then
        print_error "cargo not found. Please install Rust."
        exit 1
    fi
    
    if ! command -v docker &> /dev/null; then
        print_error "docker not found. Please install Docker."
        exit 1
    fi
    
    if ! command -v curl &> /dev/null; then
        print_error "curl not found. Please install curl."
        exit 1
    fi
    
    # Check for benchmark tools
    BENCH_TOOL=""
    if command -v wrk &> /dev/null; then
        BENCH_TOOL="wrk"
        print_success "Using wrk for benchmarking"
    elif command -v ab &> /dev/null; then
        BENCH_TOOL="ab"
        print_success "Using Apache Bench (ab) for benchmarking"
    elif command -v hey &> /dev/null; then
        BENCH_TOOL="hey"
        print_success "Using hey for benchmarking"
    else
        print_error "No benchmark tool found. Please install one of: wrk, ab (apache2-utils), or hey"
        print_info "Install instructions:"
        print_info "  - wrk: sudo apt-get install wrk (Ubuntu/Debian)"
        print_info "  - ab: sudo apt-get install apache2-utils (Ubuntu/Debian)"
        print_info "  - hey: go install github.com/rakyll/hey@latest"
        exit 1
    fi
}

# Start infrastructure
start_infrastructure() {
    print_info "Starting PostgreSQL and Jaeger..."
    docker compose -f docker-compose.otel.yml up -d
    
    print_info "Waiting for services to be ready..."
    sleep 5
    
    if ! docker exec rust-api-postgres pg_isready -U test > /dev/null 2>&1; then
        print_error "PostgreSQL is not ready"
        exit 1
    fi
    print_success "PostgreSQL is ready"
    
    if ! curl -s http://localhost:16686 > /dev/null; then
        print_error "Jaeger is not accessible"
        exit 1
    fi
    print_success "Jaeger is ready"
}

# Run migrations
run_migrations() {
    print_info "Running database migrations..."
    if command -v diesel &> /dev/null; then
        diesel migration run
        print_success "Migrations completed"
    else
        print_error "diesel CLI not found"
        exit 1
    fi
}

# Build the application
build_app() {
    print_info "Building application in release mode..."
    cargo build --release
    print_success "Build completed"
}

# Run benchmark
run_benchmark() {
    local otel_enabled=$1
    local test_name=$2
    local url="http://localhost:8080/api-doc/openapi.json"
    
    print_header "Benchmark: $test_name"
    
    # Set environment variables
    export OTEL_ENABLED=$otel_enabled
    export OTEL_ENDPOINT=http://localhost:4317
    export OTEL_SERVICE_NAME=rust-api-benchmark
    export RUST_LOG=warn
    
    # Start the server
    print_info "Starting API server (OTEL_ENABLED=$otel_enabled)..."
    ./target/release/rust_api &
    SERVER_PID=$!
    
    # Wait for server to start
    sleep 5
    
    # Check if server is running
    if ! curl -s http://localhost:8080/swagger-ui/ > /dev/null; then
        print_error "Server failed to start"
        kill $SERVER_PID 2>/dev/null || true
        return 1
    fi
    print_success "Server is running (PID: $SERVER_PID)"
    
    # Warm up
    print_info "Warming up..."
    for i in {1..10}; do
        curl -s $url > /dev/null
    done
    sleep 2
    
    # Run benchmark based on available tool
    print_info "Running benchmark..."
    
    case $BENCH_TOOL in
        wrk)
            wrk -t4 -c100 -d30s --latency $url | tee "benchmark_${test_name}.txt"
            ;;
        ab)
            ab -n 10000 -c 100 $url | tee "benchmark_${test_name}.txt"
            ;;
        hey)
            hey -n 10000 -c 100 $url | tee "benchmark_${test_name}.txt"
            ;;
    esac
    
    print_success "Benchmark completed"
    
    # Stop the server
    print_info "Stopping server..."
    kill $SERVER_PID 2>/dev/null || true
    sleep 2
}

# Parse results
parse_results() {
    print_header "Benchmark Results Summary"
    
    echo ""
    echo "OpenTelemetry DISABLED:"
    echo "------------------------"
    if [ -f "benchmark_otel_disabled.txt" ]; then
        case $BENCH_TOOL in
            wrk)
                grep "Requests/sec:" benchmark_otel_disabled.txt
                grep "Latency" benchmark_otel_disabled.txt | head -1
                ;;
            ab)
                grep "Requests per second:" benchmark_otel_disabled.txt
                grep "Time per request:" benchmark_otel_disabled.txt | head -1
                ;;
            hey)
                grep "Requests/sec:" benchmark_otel_disabled.txt
                grep "Average:" benchmark_otel_disabled.txt
                ;;
        esac
    fi
    
    echo ""
    echo "OpenTelemetry ENABLED:"
    echo "----------------------"
    if [ -f "benchmark_otel_enabled.txt" ]; then
        case $BENCH_TOOL in
            wrk)
                grep "Requests/sec:" benchmark_otel_enabled.txt
                grep "Latency" benchmark_otel_enabled.txt | head -1
                ;;
            ab)
                grep "Requests per second:" benchmark_otel_enabled.txt
                grep "Time per request:" benchmark_otel_enabled.txt | head -1
                ;;
            hey)
                grep "Requests/sec:" benchmark_otel_enabled.txt
                grep "Average:" benchmark_otel_enabled.txt
                ;;
        esac
    fi
    
    echo ""
    print_info "Detailed results saved to:"
    print_info "  - benchmark_otel_disabled.txt"
    print_info "  - benchmark_otel_enabled.txt"
}

# Cleanup
cleanup() {
    print_info "Cleaning up..."
    
    # Kill any running rust_api processes
    pkill -f "target/release/rust_api" 2>/dev/null || true
    
    # Stop Docker services
    docker compose -f docker-compose.otel.yml down
    
    print_success "Cleanup completed"
}

# Main execution
main() {
    print_header "OpenTelemetry Performance Benchmark"
    
    # Trap Ctrl+C
    trap cleanup EXIT INT TERM
    
    check_requirements
    start_infrastructure
    run_migrations
    build_app
    
    # Run benchmarks
    run_benchmark "false" "otel_disabled"
    sleep 5
    run_benchmark "true" "otel_enabled"
    
    # Parse and display results
    parse_results
    
    echo ""
    print_header "Benchmark Complete"
    print_success "Performance comparison completed successfully!"
    echo ""
    print_info "Next steps:"
    echo "  1. Review the detailed results in benchmark_*.txt files"
    echo "  2. Calculate the overhead percentage"
    echo "  3. Check Jaeger UI for traces: http://localhost:16686"
    echo ""
}

# Run main function
main
