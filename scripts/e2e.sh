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
    cargo test --test end_to_end --features postgres
    print_success "PostgreSQL E2E tests passed"
}

cleanup() {
  true
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
                check_docker && run_postgres_tests
                exit 0
                ;;
            -h|--help)
                echo "Usage: $0 [OPTIONS]"
                echo ""
                echo "Options:"
                echo "  --only-unit         Run only unit tests"
                echo "  --only-integration  Run only integration tests"
                echo "  --only-sqlite       Run only SQLite E2E tests"
                echo "  --only-postgres     Run only PostgreSQL E2E tests"
                echo "  -h, --help          Show this help message"
                echo ""
                echo "Examples:"
                echo "  $0                  # Run all tests"
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
    
    if check_docker && ! run_postgres_tests; then
        FAILED_TESTS+=("PostgreSQL E2E Tests")
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

