set shell := ["bash", "-euc"]

# Run all tests normally and with coverage instrumentation.
default: test test-coverage

# Run the supported feature matrix.
test:
    cargo test --workspace
    cargo test --workspace --all-features

# Measure every implementation with the coverage tooling for its runtime.
test-coverage: \
    test-coverage-rust \
    test-coverage-node \
    test-coverage-wasm \
    test-coverage-python

# Measure the Rust core crate with LLVM coverage.
test-coverage-rust:
    cargo +nightly llvm-cov \
        --package meterbus-wired-datalink \
        --all-features \
        --doctests \
        --fail-under-lines 100 \
        --fail-under-functions 100 \
        --fail-under-regions 100

# Measure the Node.js package with V8 coverage.
test-coverage-node:
    pnpm --dir packages/meterbus-wired-datalink-node/main build
    pnpm --dir packages/meterbus-wired-datalink-node/main test:coverage

# Measure the WebAssembly package with V8 coverage.
test-coverage-wasm:
    pnpm --dir packages/meterbus-wired-datalink-wasm build
    pnpm --dir packages/meterbus-wired-datalink-wasm test:coverage

# Measure the Python package with coverage.py.
test-coverage-python:
    uv run --directory packages/meterbus-wired-datalink-python pytest
