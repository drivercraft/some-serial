# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a Rust `no_std` library that provides a PL011 UART driver implementation for ARM-based systems. The library offers both low-level register access and a high-level serial interface abstraction.

## Architecture

### Core Components

- **`src/lib.rs`**: Main library interface defining:
  - `SerialRegister` trait: Core abstraction for serial communication operations
  - Configuration types (`Config`, `DataBits`, `StopBits`, `Parity`)
  - Error types (`ConfigError`, `TransferError`)
  - Status flag types (`InterruptMask`, `LineStatus`)

- **`src/pl011.rs`**: PL011 UART specific implementation:
  - `Pl011` struct implementing `SerialRegister` trait
  - Register definitions using `tock-registers` for type-safe memory-mapped I/O
  - Hardware-specific initialization and configuration logic

### Key Design Patterns

- **Trait-based abstraction**: `SerialRegister` trait allows for different UART implementations
- **Type-safe register access**: Uses `tock-registers` for compile-time register field validation
- **Error handling**: Comprehensive error types for configuration and transfer operations
- **`no_std` compatibility**: Suitable for bare-metal/embedded environments

## Build System

### Toolchain Requirements

- **Nightly Rust**: Required (see `rust-toolchain.toml`)
- **Components**: rust-src, rustfmt, clippy, llvm-tools

### Build Configuration

- **Target platforms**: Supports `x86_64` and bare-metal targets (`target_os = "none"`)
- **Testing**: Uses `bare-test` framework for bare-metal testing
- **Custom build**: `build.rs` sets up bare-test macros

## Common Development Commands

### Building

```bash
# Build for host target
cargo build

# Build for bare-metal target (aarch64-unknown-none-softfloat)
cargo build --target aarch64-unknown-none-softfloat
```

### Testing

```bash
# Run bare-metal tests
cargo test --test test -- tests --show-output --uboot 

# Run tests for host target (if any)
cargo test --test test -- tests --show-output
```

### Code Quality

```bash
# Format code
cargo fmt

# Run clippy lints
cargo clippy

# Check compilation without building
cargo check
```

## Dependencies

### Core Dependencies

- `tock-registers`: Type-safe register access (0.10)
- `bitflags`: Bit flag types (2.8)
- `thiserror`: Error handling (2.0)
- `dma-api`: DMA operations with alloc feature (0.5)
- `rdif-base`: Base functionality (0.7)

### Platform-specific

- `x86_64`: x86_64 specific support (0.15) - only on x86_64 targets
- `bare-test`: Testing framework for bare-metal (0.7) - dev-dependency

## Testing Framework

Uses `bare-test` for bare-metal testing configuration:
- Test harness disabled in `Cargo.toml`
- QEMU configuration in `bare-test.toml` (graphics disabled)
- Custom test setup via `bare-test-macros` in build script

## Memory Layout

The PL011 driver uses memory-mapped registers:
- Register base address provided via `NonNull<u8>`
- Register offsets defined in `Pl011Registers` struct
- Total register space: 4KB (0x1000 bytes)
- All register access is volatile and atomic

## Key Implementation Details

### UART Configuration Flow

1. Disable UART
2. Wait for transmission completion
3. Flush FIFO
4. Configure baud rate, data bits, stop bits, parity
5. Re-enable FIFO
6. Restore UART enable state

### Baud Rate Calculation

Uses ARM PL011 formula:
```
BAUDDIV = FUARTCLK / (16 * Baud rate)
IBRD = integer(BAUDDIV)
FBRD = integer((BAUDDIV - IBRD) * 64 + 0.5)
```

### Clock Frequency Detection

- Attempts to detect from existing register settings
- Falls back to 24MHz default if detection fails
- Supports 1MHz-100MHz range validation