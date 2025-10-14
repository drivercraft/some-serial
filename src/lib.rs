#![no_std]

extern crate alloc;

use bitflags::bitflags;

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

/// 电源模式配置
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PowerMode {
    Normal,
    LowPower,
    Off,
}

// ============================================================================
// 状态标志类型
// ============================================================================

bitflags! {
    /// 中断使能掩码
    pub struct InterruptMask: u32 {
        const RX_AVAILABLE = 0x01;
        const TX_EMPTY = 0x02;
        const RX_LINE_STATUS = 0x04;
        const MODEM_STATUS = 0x08;
        const CHARACTER_TIMEOUT = 0x10;
        const ALL = 0x1F;
    }
}

bitflags! {
    /// 中断状态标志
    pub struct InterruptStatus: u32 {
        const RX_AVAILABLE = 0x01;
        const TX_EMPTY = 0x02;
        const RX_LINE_STATUS = 0x04;
        const MODEM_STATUS = 0x08;
        const CHARACTER_TIMEOUT = 0x10;
        const PENDING = 0x01;
    }
}

bitflags! {
    /// 线路状态标志
    pub struct LineStatus: u32 {
        const DATA_READY = 0x01;
        const OVERRUN_ERROR = 0x02;
        const PARITY_ERROR = 0x04;
        const FRAMING_ERROR = 0x08;
        const BREAK_INTERRUPT = 0x10;
        const TX_HOLDING_EMPTY = 0x20;
        const TX_EMPTY = 0x40;
        const FIFO_ERROR = 0x80;
    }
}

// ============================================================================
// 扩展的SerialRegister接口
// ============================================================================

pub trait SerialRegister: Clone + Send + Sync {
    // ==================== 基础数据传输（保持向后兼容） ====================
    fn write_byte(&self, byte: u8);
    fn read_byte(&self) -> u8;
    fn can_read(&self) -> bool;
    fn can_write(&self) -> bool;

    // ==================== 配置管理 ====================
    /// 设置波特率
    fn set_baudrate(&self, baudrate: u32) -> Result<(), SerialError>;
    /// 获取当前波特率
    fn get_baudrate(&self) -> u32;

    /// 设置数据位数
    fn set_data_bits(&self, bits: DataBits) -> Result<(), SerialError>;
    /// 设置停止位数
    fn set_stop_bits(&self, bits: StopBits) -> Result<(), SerialError>;
    /// 设置奇偶校验
    fn set_parity(&self, parity: Parity) -> Result<(), SerialError>;

    fn open(&self) -> Result<(), SerialError>;
    fn close(&self) -> Result<(), SerialError>;

    // ==================== 中断管理 ====================
    /// 使能中断
    fn enable_interrupts(&self, mask: InterruptMask);
    /// 禁用中断
    fn disable_interrupts(&self, mask: InterruptMask);
    /// 获取中断状态
    fn get_interrupt_status(&self) -> InterruptStatus;
    /// 清除中断状态
    fn clear_interrupt_status(&self, mask: InterruptStatus);

    // ==================== 传输状态查询 ====================
    /// 检查发送寄存器是否为空
    fn is_tx_empty(&self) -> bool;
    /// 检查接收FIFO是否为空
    fn is_rx_empty(&self) -> bool;
    /// 获取发送FIFO级别
    fn get_tx_fifo_level(&self) -> u16;
    /// 获取接收FIFO级别
    fn get_rx_fifo_level(&self) -> u16;

    /// 获取线路状态
    fn get_line_status(&self) -> LineStatus;
    /// 清除错误状态
    fn clear_error(&self);

    // ==================== 底层寄存器访问 ====================
    /// 直接读取寄存器
    fn read_reg(&self, offset: usize) -> u32;
    /// 直接写入寄存器
    fn write_reg(&self, offset: usize, value: u32);
    /// 修改寄存器位
    fn modify_reg(&self, offset: usize, mask: u32, set: u32);

    fn mmio_base(&self) -> usize;
    fn set_mmio_base(&mut self, base: usize);
}
