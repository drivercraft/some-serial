#![no_std]

use core::ptr::NonNull;

use bitflags::bitflags;
use mbarrier::rmb;

pub mod pl011;

// ============================================================================
// 错误类型定义
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigError {
    /// 无效的波特率
    InvalidBaudrate,
    /// 不支持的数据位配置
    UnsupportedDataBits,
    /// 不支持的停止位配置
    UnsupportedStopBits,
    /// 不支持的奇偶校验配置
    UnsupportedParity,
    /// 寄存器访问错误
    RegisterError,
    /// 超时错误
    Timeout,
}

#[derive(thiserror::Error, Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransferError {
    #[error("Data overrun")]
    Overrun(u8),
    #[error("Parity error")]
    Parity,
    #[error("Framing error")]
    Framing,
    #[error("Break condition")]
    Break,
}

// ============================================================================
// 配置枚举类型
// ============================================================================

/// 数据位配置
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum DataBits {
    Five = 5,
    Six = 6,
    Seven = 7,
    Eight = 8,
}

/// 停止位配置
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum StopBits {
    One = 1,
    Two = 2,
}

/// 奇偶校验配置
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Parity {
    None,
    Even,
    Odd,
    Mark,
    Space,
}

// ============================================================================
// 状态标志类型
// ============================================================================

bitflags! {
    /// 中断状态标志
    #[derive(Debug, Clone, Copy)]
    pub struct InterruptMask: u32 {
        const RX_AVAILABLE = 0x01;
        const TX_EMPTY = 0x02;
        const RX_LINE_STATUS = 0x04;
        const MODEM_STATUS = 0x08;
        const CHARACTER_TIMEOUT = 0x10;
    }
}

bitflags! {
    /// 线路状态标志
    #[derive(Debug, Clone, Copy)]
    pub struct LineStatus: u32 {
        const DATA_READY = 0x01;
        const TX_HOLDING_EMPTY = 0x20;
    }
}

impl LineStatus {
    pub fn can_read(&self) -> bool {
        self.contains(LineStatus::DATA_READY)
    }

    pub fn can_write(&self) -> bool {
        self.contains(LineStatus::TX_HOLDING_EMPTY)
    }
}

#[derive(Debug, Clone, Default)]
pub struct Config {
    pub baudrate: Option<u32>,
    pub data_bits: Option<DataBits>,
    pub stop_bits: Option<StopBits>,
    pub parity: Option<Parity>,
}

impl Config {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn baudrate(mut self, baudrate: u32) -> Self {
        self.baudrate = Some(baudrate);
        self
    }

    pub fn data_bits(mut self, data_bits: DataBits) -> Self {
        self.data_bits = Some(data_bits);
        self
    }

    pub fn stop_bits(mut self, stop_bits: StopBits) -> Self {
        self.stop_bits = Some(stop_bits);
        self
    }

    pub fn parity(mut self, parity: Parity) -> Self {
        self.parity = Some(parity);
        self
    }
}

pub trait SerialRegister: Send + Sync {
    // ==================== 基础数据传输 ====================
    fn write_byte(&mut self, byte: u8);
    fn read_byte(&self) -> Result<u8, TransferError>;

    // ==================== 配置管理 ====================
    fn set_config(&mut self, config: &Config) -> Result<(), ConfigError>;

    fn baudrate(&self) -> u32;
    fn data_bits(&self) -> DataBits;
    fn stop_bits(&self) -> StopBits;
    fn parity(&self) -> Parity;
    fn clock_freq(&self) -> u32;

    fn open(&mut self);
    fn close(&mut self);

    // ==================== 回环控制 ====================
    /// 启用回环模式
    fn enable_loopback(&mut self);
    /// 禁用回环模式
    fn disable_loopback(&mut self);
    /// 检查回环模式是否启用
    fn is_loopback_enabled(&self) -> bool;

    // ==================== 中断管理 ====================
    /// 使能中断
    fn enable_interrupts(&mut self, mask: InterruptMask);
    /// 禁用中断
    fn disable_interrupts(&mut self, mask: InterruptMask);
    /// 获取并清除所有中断状态
    fn clean_interrupt_status(&mut self) -> InterruptMask;

    // ==================== 传输状态查询 ====================

    /// 获取线路状态
    fn line_status(&self) -> LineStatus;

    // ==================== 底层寄存器访问 ====================
    /// 直接读取寄存器
    fn read_reg(&self, offset: usize) -> u32;
    /// 直接写入寄存器
    fn write_reg(&mut self, offset: usize, value: u32);

    fn get_base(&self) -> usize;
    fn set_base(&mut self, base: NonNull<u8>);

    fn read_buf(&mut self, buf: &mut [u8]) -> Result<usize, TransferError> {
        let mut read_count = 0;
        let mut overrun = false;
        for byte in buf.iter_mut() {
            if !self.line_status().can_read() {
                if overrun {
                    return Err(TransferError::Overrun(0));
                }
                break;
            }
            rmb();
            match self.read_byte() {
                Ok(b) => *byte = b,
                Err(TransferError::Overrun(u8)) => {
                    overrun = true;
                    *byte = u8;
                }
                Err(e) => return Err(e),
            }

            *byte = self.read_byte()?;
            read_count += 1;
        }

        Ok(read_count)
    }

    fn write_buf(&mut self, buf: &[u8]) -> usize {
        let mut write_count = 0;
        for &byte in buf.iter() {
            if !self.line_status().can_write() {
                break;
            }
            rmb();
            self.write_byte(byte);
            write_count += 1;
        }
        write_count
    }
}
