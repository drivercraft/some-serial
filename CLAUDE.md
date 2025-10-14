# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a Rust `no_std` serial communication library that provides abstract interfaces for UART/serial devices. The library is designed for embedded and bare-metal systems, particularly targeting ARM64 platforms with device tree support.

## Architecture

### Core Components

- **SerialRegister Trait** (`src/lib.rs:8-14`): Abstract interface for serial port operations
  - `write_byte(&self, byte: u8)`: Write a single byte
  - `read_byte(&self) -> u8`: Read a single byte
  - `can_read(&self) -> bool`: Check if data is available
  - `can_write(&self) -> bool`: Check if ready to write
  - `enable_interrupts(&self)`: Enable interrupt generation

- **PL011 UART Support**: The test code references `some_serial::pl011` implementation for ARM PL011 UART

### Target Platform

- **Primary**: ARM64 (`aarch64-unknown-none-softfloat`)
- **Secondary**: x86_64 for development/testing
- **Environment**: Bare-metal with device tree support

## Development Commands

### Build and Test

```bash
# Build for default target (x86_64)
cargo build

# Build for ARM64 bare-metal target
cargo build --target aarch64-unknown-none-softfloat

# Run tests (bare-metal environment)
cargo test

# Build and run specific test
cargo test --target aarch64-unknown-none-softfloat
```

### Code Quality

```bash
# Format code
cargo fmt

# Run linter
cargo clippy

# Check without building
cargo check
```

## Dependencies

### Core Dependencies
- `bitflags = "2.8"`: Bit flag manipulation
- `serial-async = "0.2"`: Asynchronous serial operations

### Platform-Specific
- `x86_64 = "0.15"`: x86_64 architecture support (conditional)
- `bare-test = "0.6"`: Testing framework for bare-metal (dev-dependency)
- `log = "0.4"`: Logging support (dev-dependency)

### Build Dependencies
- `bare-test-macros = "0.2"`: Build macros for bare-metal testing

## Testing Framework

This project uses `bare-test` for bare-metal unit testing:
- Tests are located in `tests/test.rs`
- Custom test harness disabled (`harness = false`)
- Tests run in bare-metal environment with device tree support
- Build script configures test environment with `bare_test_macros::build_test_setup!()`

## Development Notes

### Toolchain Requirements
- Requires nightly Rust toolchain
- Components: rust-src, rustfmt, clippy, llvm-tools
- Defined in `rust-toolchain.toml`

### Code Conventions
- `no_std` environment - no standard library
- Uses `extern crate alloc` for heap allocations
- Follows Rust 2024 edition
- Traits require `Clone + Send + Sync` bounds

### Device Tree Integration
- Test framework integrates with device tree for hardware discovery
- Uses `bare_test::globals::global_val()` for platform information
- UART devices discovered via compatible strings in device tree