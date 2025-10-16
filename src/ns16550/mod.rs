//! NS16550/16450 UART 驱动模块
//!
//! 提供两种访问方式：
//! - IO Port 版本（x86_64 架构）
//! - MMIO 版本（通用嵌入式平台）

// 公共寄存器定义
mod registers;

#[cfg(target_arch = "x86_64")]
pub mod pio;

// MMIO 版本（通用）
pub mod mmio;

#[cfg(target_arch = "x86_64")]
pub use pio::*;

pub use mmio::*;
