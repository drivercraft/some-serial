#![no_std]

extern crate alloc;

use bitflags::bitflags;

pub mod pl011;

// ============================================================================
// 错误类型定义
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SerialError {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransferError {
    Overrun,
    Parity,
    Framing,
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

// ============================================================================
// 扩展的SerialRegister接口
// ============================================================================

// ============================================================================
// 配置验证和格式化函数
// ============================================================================

/// 验证串口配置是否有效
pub fn validate_serial_config(data_bits: DataBits, stop_bits: StopBits, parity: Parity) -> bool {
    match (data_bits, stop_bits, parity) {
        // 8 数据位不支持 2 停止位（除非有奇偶校验）
        (DataBits::Eight, StopBits::Two, Parity::None) => false,

        // 5 数据位不支持 1.5 停止位（我们的枚举中没有这个）
        (DataBits::Five, StopBits::Two, _) => {
            // 5 数据位通常配合 1.5 或 2 停止位使用
            matches!(parity, Parity::Even | Parity::Odd)
        }

        // 其他组合都是有效的
        _ => true,
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

/// 格式化串口配置为可读字符串
pub fn format_serial_config(
    baudrate: u32,
    data_bits: DataBits,
    stop_bits: StopBits,
    parity: Parity,
) -> alloc::string::String {
    use alloc::format;

    let data_bits_str = match data_bits {
        DataBits::Five => "5",
        DataBits::Six => "6",
        DataBits::Seven => "7",
        DataBits::Eight => "8",
    };

    let stop_bits_str = match stop_bits {
        StopBits::One => "1",
        StopBits::Two => "2",
    };

    let parity_str = match parity {
        Parity::None => "no-parity",
        Parity::Even => "even-parity",
        Parity::Odd => "odd-parity",
        Parity::Mark => "mark-parity",
        Parity::Space => "space-parity",
    };

    format!(
        "{} baud, {}-data bits, {}-stop bits, {}",
        baudrate, data_bits_str, stop_bits_str, parity_str
    )
}

pub trait SerialRegister: Send + Sync {
    // ==================== 基础数据传输 ====================
    fn write_byte(&mut self, byte: u8) -> Result<(), TransferError>;
    fn read_byte(&self) -> Result<u8, TransferError>;

    // ==================== 配置管理 ====================
    fn set_config(&mut self, config: &Config) -> Result<(), SerialError>;

    fn baudrate(&self) -> u32;
    fn data_bits(&self) -> DataBits;
    fn stop_bits(&self) -> StopBits;
    fn parity(&self) -> Parity;

    fn open(&mut self) -> Result<(), SerialError>;
    fn close(&mut self) -> Result<(), SerialError>;

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
    fn set_base(&mut self, base: usize);

    fn read_buf(&mut self, buf: &mut [u8]) -> Result<usize, TransferError> {
        let mut read_count = 0;
        for byte in buf.iter_mut() {
            if !self.line_status().can_read() {
                break;
            }

            match self.read_byte() {
                Ok(b) => {
                    *byte = b;
                    read_count += 1;
                }
                Err(e) => return Err(e),
            }
        }
        Ok(read_count)
    }

    fn write_buf(&mut self, buf: &[u8]) -> Result<usize, TransferError> {
        let mut write_count = 0;
        for &byte in buf.iter() {
            if !self.line_status().can_write() {
                break;
            }

            match self.write_byte(byte) {
                Ok(()) => write_count += 1,
                Err(e) => return Err(e),
            }
        }
        Ok(write_count)
    }
}
