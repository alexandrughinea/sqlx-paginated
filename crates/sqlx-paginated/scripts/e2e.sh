#!/bin/bash
set -e

print_header() {
    echo "[INFO] $1"
}

print_success() {
    echo "[PASS] $1"
}

print_error() {
    echo "[FAIL] $1"
}

print_warning() {
    echo "[WARN] $1"
}

check_docker() {
    if ! command -v docker &> /dev/null; then
        print_warning "Docker not found. PostgreSQL tests will be skipped."
        return 1
    fi
    return 0
}

start_postgres() {
    echo "[INFO] Starting PostgreSQL for E2E tests"
    
    if docker ps -a | grep -q sqlx-paginated-test-postgres; then
        echo "[WARN] Container already exists. Removing..."
        docker stop sqlx-paginated-test-postgres 2>/dev/null || true
        docker rm sqlx-paginated-test-postgres 2>/dev/null || true
    fi
    
    docker run --name sqlx-paginated-test-postgres \
        -e POSTGRES_USER=postgres \
        -e POSTGRES_PASSWORD=postgres \
        -e POSTGRES_DB=sqlx_paginated_test \
        -p 5432:5432 \
        -d postgres:15-alpine
    
    echo "[INFO] PostgreSQL container started"
    
    echo "[INFO] Waiting for PostgreSQL to be ready..."
    for i in {1..30}; do
        if docker exec sqlx-paginated-test-postgres pg_isready -U postgres &>/dev/null; then
            echo "[PASS] PostgreSQL is ready"
            return 0
        fi
        echo -n "."
        sleep 1
    done
    
    print_error "PostgreSQL failed to start in time"
    return 1
}

stop_postgres() {
    echo "[INFO] Stopping PostgreSQL"
    docker stop sqlx-paginated-test-postgres 2>/dev/null || true
    docker rm sqlx-paginated-test-postgres 2>/dev/null || true
    echo "[INFO] PostgreSQL stopped"
}

run_unit_tests() {
    cargo test --lib
    print_success "Unit tests passed"
}

run_integration_tests() {
    cargo test --test integration
    print_success "Integration tests passed"
}

run_sqlite_tests() {
    print_header "Running SQLite E2E Tests"
    cargo test --test end_to_end --features sqlite
    print_success "SQLite E2E tests passed"
}

run_postgres_tests() {
    print_header "Running PostgreSQL E2E Tests"
    export TEST_DATABASE_URL="postgres://postgres:postgres@localhost:5432/sqlx_paginated_test"
    cargo test --test end_to_end --features postgres -- --ignored --test-threads=1
    print_success "PostgreSQL E2E tests passed"
}

cleanup() {
    if [ "$POSTGRES_STARTED" = true ]; then
        stop_postgres
    fi
}

trap cleanup EXIT INT TERM

main() {
    POSTGRES_STARTED=false
    SKIP_POSTGRES=false
    FAILED_TESTS=()
    
    while [[ $# -gt 0 ]]; do
        case $1 in
            --skip-postgres)
                SKIP_POSTGRES=true
                shift
                ;;
            --only-unit)
                run_unit_tests
                exit 0
                ;;
            --only-integration)
                run_integration_tests
                exit 0
                ;;
            --only-sqlite)
                run_sqlite_tests
                exit 0
                ;;
            --only-postgres)
                if check_docker && start_postgres; then
                    POSTGRES_STARTED=true
                    run_postgres_tests
                fi
                exit 0
                ;;
            -h|--help)
                echo "Usage: $0 [OPTIONS]"
                echo ""
                echo "Options:"
                echo "  --skip-postgres     Skip PostgreSQL E2E tests"
                echo "  --only-unit         Run only unit tests"
                echo "  --only-integration  Run only integration tests"
                echo "  --only-sqlite       Run only SQLite E2E tests"
                echo "  --only-postgres     Run only PostgreSQL E2E tests"
                echo "  -h, --help          Show this help message"
                echo ""
                echo "Examples:"
                echo "  $0                  # Run all tests"
                echo "  $0 --skip-postgres  # Run all tests except PostgreSQL"
                echo "  $0 --only-unit      # Run only unit tests"
                exit 0
                ;;
            *)
                print_error "Unknown option: $1"
                echo "Use --help for usage information"
                exit 1
                ;;
        esac
    done
    
    echo "[INFO] sqlx-paginated Test Suite Runner"
    
    if ! run_unit_tests; then
        FAILED_TESTS+=("Unit Tests")
    fi
    
    if ! run_integration_tests; then
        FAILED_TESTS+=("Integration Tests")
    fi
    
    if ! run_sqlite_tests; then
        FAILED_TESTS+=("SQLite E2E Tests")
    fi
    
    if [ "$SKIP_POSTGRES" = false ]; then
        if check_docker; then
            if start_postgres; then
                POSTGRES_STARTED=true
                if ! run_postgres_tests; then
                    FAILED_TESTS+=("PostgreSQL E2E Tests")
                fi
            else
                print_error "Failed to start PostgreSQL"
                FAILED_TESTS+=("PostgreSQL E2E Tests (setup failed)")
            fi
        else
            print_warning "Skipping PostgreSQL tests (Docker not available)"
        fi
    else
        print_warning "Skipping PostgreSQL tests (--skip-postgres flag)"
    fi
    
    echo ""
    echo "[INFO] Test Summary"
    
    if [ ${#FAILED_TESTS[@]} -eq 0 ]; then
        print_success "All tests passed!"
        exit 0
    else
        print_error "Some tests failed:"
        for test in "${FAILED_TESTS[@]}"; do
            echo "  - $test"
        done
        exit 1
    fi
}

main "$@"

