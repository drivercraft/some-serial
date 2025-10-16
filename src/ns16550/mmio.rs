//! NS16550 MMIO 版本实现
//!
//! 适用于嵌入式平台的内存映射 IO 版本

use crate::ns16550::registers::*;
use core::ptr::NonNull;
use heapless::Vec;
use rdif_serial::{
    Config, ConfigError, DataBits, Parity, Register, Serial, StopBits, TransferError,
};
use rdif_serial::{InterruptMask, LineStatus};

/// NS16550 MMIO 版本驱动
#[derive(Clone, Debug)]
#[repr(align(64))]
pub struct Ns16550Mmio {
    rcv_fifo: Vec<u8, 64>,
    base: NonNull<u8>,
    clock_freq: u32,
    err: Option<TransferError>,
    is_tx_empty_int_enabled: bool,
}

unsafe impl Send for Ns16550Mmio {}
unsafe impl Sync for Ns16550Mmio {}

impl Ns16550Mmio {
    /// 创建新的 NS16550 MMIO 实例
    ///
    /// # Arguments
    /// * `base` - MMIO 基地址
    /// * `clock_freq` - 输入时钟频率 (Hz)
    pub fn new(base: NonNull<u8>, clock_freq: u32) -> Serial<Self> {
        Serial::new(Self {
            base,
            clock_freq,
            rcv_fifo: Vec::new(),
            err: None,
            is_tx_empty_int_enabled: false,
        })
    }

    /// 读取寄存器
    #[inline]
    fn read_reg(&self, offset: u8) -> u8 {
        unsafe {
            let addr = self.base.as_ptr().add((offset as usize) * 4);
            core::ptr::read_volatile(addr)
        }
    }

    /// 写入寄存器
    #[inline]
    fn write_reg(&mut self, offset: u8, value: u8) {
        unsafe {
            let addr = self.base.as_ptr().add((offset as usize) * 4);
            core::ptr::write_volatile(addr, value);
        }
    }

    /// 检查是否为 16550+（支持 FIFO）
    pub fn is_16550_plus(&self) -> bool {
        // 通过读取 IIR 寄存器的 FIFO 位来判断
        // IIR 的位7-6在 16550+ 中会显示 FIFO 启用状态
        let iir = self.read_reg(UART_IIR);
        (iir & UART_IIR_FIFO_MASK) == UART_IIR_FIFO_ENABLE
    }

    /// 设置波特率
    fn set_baudrate_internal(&mut self, baudrate: u32) -> Result<(), ConfigError> {
        if baudrate == 0 || self.clock_freq == 0 {
            return Err(ConfigError::InvalidBaudrate);
        }

        let divisor = self.clock_freq / (16 * baudrate);
        if divisor == 0 || divisor > 0xFFFF {
            return Err(ConfigError::InvalidBaudrate);
        }

        // 保存原始 LCR
        let original_lcr = self.read_reg(UART_LCR);

        // 设置 DLAB 以访问波特率除数寄存器
        self.write_reg(UART_LCR, original_lcr | UART_LCR_DLAB);

        // 设置除数
        self.write_reg(UART_DLL, (divisor & 0xFF) as u8);
        self.write_reg(UART_DLH, ((divisor >> 8) & 0xFF) as u8);

        // 恢复原始 LCR
        self.write_reg(UART_LCR, original_lcr);

        Ok(())
    }

    /// 设置数据位
    fn set_data_bits_internal(&mut self, bits: DataBits) -> Result<(), ConfigError> {
        let wlen = match bits {
            DataBits::Five => UART_LCR_WLEN5,
            DataBits::Six => UART_LCR_WLEN6,
            DataBits::Seven => UART_LCR_WLEN7,
            DataBits::Eight => UART_LCR_WLEN8,
        };

        let original_lcr = self.read_reg(UART_LCR);
        self.write_reg(UART_LCR, (original_lcr & !UART_LCR_WLEN_MASK) | wlen);

        Ok(())
    }

    /// 设置停止位
    fn set_stop_bits_internal(&mut self, bits: StopBits) -> Result<(), ConfigError> {
        let original_lcr = self.read_reg(UART_LCR);
        match bits {
            StopBits::One => {
                self.write_reg(UART_LCR, original_lcr & !UART_LCR_STOP);
            }
            StopBits::Two => {
                self.write_reg(UART_LCR, original_lcr | UART_LCR_STOP);
            }
        }
        Ok(())
    }

    /// 设置奇偶校验
    fn set_parity_internal(&mut self, parity: Parity) -> Result<(), ConfigError> {
        let original_lcr = self.read_reg(UART_LCR);

        let new_lcr = match parity {
            Parity::None => original_lcr & !(UART_LCR_PARITY | UART_LCR_EPAR | UART_LCR_SPAR),
            Parity::Odd => (original_lcr | UART_LCR_PARITY) & !(UART_LCR_EPAR | UART_LCR_SPAR),
            Parity::Even => (original_lcr | UART_LCR_PARITY | UART_LCR_EPAR) & !UART_LCR_SPAR,
            Parity::Mark => original_lcr | UART_LCR_PARITY | UART_LCR_SPAR,
            Parity::Space => original_lcr | UART_LCR_PARITY | UART_LCR_EPAR | UART_LCR_SPAR,
        };

        self.write_reg(UART_LCR, new_lcr);
        Ok(())
    }

    /// 启用或禁用 FIFO
    pub fn enable_fifo(&mut self, enable: bool) {
        if enable && self.is_16550_plus() {
            self.write_reg(
                UART_FCR,
                UART_FCR_ENABLE_FIFO
                    | UART_FCR_CLEAR_RCVR
                    | UART_FCR_CLEAR_XMIT
                    | UART_FCR_TRIGGER_1,
            );
        } else {
            self.write_reg(UART_FCR, 0);
        }
    }

    /// 设置 FIFO 触发级别
    pub fn set_fifo_trigger_level(&mut self, level: u8) {
        if !self.is_16550_plus() {
            return;
        }

        let trigger_value = match level {
            0..=3 => UART_FCR_TRIGGER_1,
            4..=7 => UART_FCR_TRIGGER_4,
            8..=11 => UART_FCR_TRIGGER_8,
            _ => UART_FCR_TRIGGER_14,
        };

        // 保留其他 FIFO 设置
        let current_fcr = self.read_reg(UART_FCR);
        self.write_reg(
            UART_FCR,
            (current_fcr & !UART_FCR_TRIGGER_MASK) | trigger_value,
        );
    }

    /// 初始化 UART
    fn init(&mut self) {
        // 禁用中断
        self.write_reg(UART_IER, 0);

        // 确保传输器启用
        let original_mcr = self.read_reg(UART_MCR);
        self.write_reg(UART_MCR, original_mcr | UART_MCR_DTR | UART_MCR_RTS);
    }

    /// 清空接收 FIFO
    pub fn clear_receive_fifo(&mut self) {
        if self.is_16550_plus() {
            self.write_reg(UART_FCR, UART_FCR_ENABLE_FIFO | UART_FCR_CLEAR_RCVR);
        }
        self.rcv_fifo.clear();
    }

    /// 清空发送 FIFO
    pub fn clear_transmit_fifo(&mut self) {
        if self.is_16550_plus() {
            self.write_reg(UART_FCR, UART_FCR_ENABLE_FIFO | UART_FCR_CLEAR_XMIT);
        }
    }

    /// 检查 FIFO 是否启用
    pub fn is_fifo_enabled(&self) -> bool {
        if !self.is_16550_plus() {
            return false;
        }
        // 通过检查 IIR 的 FIFO 位来判断
        (self.read_reg(UART_IIR) & UART_IIR_FIFO_MASK) == UART_IIR_FIFO_ENABLE
    }
}

impl Register for Ns16550Mmio {
    fn write_byte(&mut self, byte: u8) {
        self.write_reg(UART_THR, byte);
        if self.is_tx_empty_int_enabled {
            // 启用 THRE 中断
            let ier = self.read_reg(UART_IER);
            self.write_reg(UART_IER, ier | UART_IER_THRI);
        }
    }

    fn read_byte(&mut self) -> Result<u8, TransferError> {
        if let Some(b) = self.rcv_fifo.pop() {
            return Ok(b);
        }
        if let Some(e) = self.err.take() {
            return Err(e);
        }
        // 读取数据
        Ok(self.read_reg(UART_RBR))
    }

    fn set_config(&mut self, config: &Config) -> Result<(), ConfigError> {
        // 配置波特率
        if let Some(baudrate) = config.baudrate {
            self.set_baudrate_internal(baudrate)?;
        }

        // 配置数据位
        if let Some(data_bits) = config.data_bits {
            self.set_data_bits_internal(data_bits)?;
        }

        // 配置停止位
        if let Some(stop_bits) = config.stop_bits {
            self.set_stop_bits_internal(stop_bits)?;
        }

        // 配置奇偶校验
        if let Some(parity) = config.parity {
            self.set_parity_internal(parity)?;
        }

        Ok(())
    }

    fn baudrate(&self) -> u32 {
        // 只读方式获取波特率，通过读取 DLL 和 DLH
        // 注意：如果 DLAB 未设置，读取的可能不是除数值
        let dll = self.read_reg(UART_DLL) as u16;
        let dlh = self.read_reg(UART_DLH) as u16;
        let divisor = dll | (dlh << 8);

        if divisor == 0 {
            return 0;
        }

        self.clock_freq / (16 * divisor as u32)
    }

    fn data_bits(&self) -> DataBits {
        let lcr = self.read_reg(UART_LCR);
        match lcr & UART_LCR_WLEN_MASK {
            UART_LCR_WLEN5 => DataBits::Five,
            UART_LCR_WLEN6 => DataBits::Six,
            UART_LCR_WLEN7 => DataBits::Seven,
            UART_LCR_WLEN8 => DataBits::Eight,
            _ => DataBits::Eight, // 默认值
        }
    }

    fn stop_bits(&self) -> StopBits {
        let lcr = self.read_reg(UART_LCR);
        if lcr & UART_LCR_STOP != 0 {
            StopBits::Two
        } else {
            StopBits::One
        }
    }

    fn parity(&self) -> Parity {
        let lcr = self.read_reg(UART_LCR);

        if lcr & UART_LCR_PARITY == 0 {
            Parity::None
        } else if lcr & UART_LCR_SPAR != 0 {
            // Stick parity
            if lcr & UART_LCR_EPAR != 0 {
                Parity::Space
            } else {
                Parity::Mark
            }
        } else {
            // Normal parity
            if lcr & UART_LCR_EPAR != 0 {
                Parity::Even
            } else {
                Parity::Odd
            }
        }
    }

    fn open(&mut self) {
        self.init();
    }

    fn close(&mut self) {
        // 禁用中断
        self.write_reg(UART_IER, 0);

        // 禁用 DTR 和 RTS
        let original_mcr = self.read_reg(UART_MCR);
        self.write_reg(UART_MCR, original_mcr & !(UART_MCR_DTR | UART_MCR_RTS));
    }

    fn clean_interrupt_status(&mut self) -> InterruptMask {
        let iir = self.read_reg(UART_IIR);
        let mut mask = InterruptMask::empty();

        match iir & UART_IIR_INTERRUPT_MASK {
            UART_IIR_RLSI => {
                let lsr = self.read_reg(UART_LSR);
                if lsr & UART_LSR_OE != 0 {
                    let d = self.read_reg(UART_RBR);
                    self.err = Some(TransferError::Overrun(d));
                    mask |= InterruptMask::RX_AVAILABLE;
                }
                if lsr & UART_LSR_PE != 0 {
                    self.err = Some(TransferError::Parity);
                    mask |= InterruptMask::RX_AVAILABLE;
                }
                if lsr & UART_LSR_FE != 0 {
                    self.err = Some(TransferError::Framing);
                    mask |= InterruptMask::RX_AVAILABLE;
                }
                if lsr & UART_LSR_BI != 0 {
                    self.err = Some(TransferError::Break);
                    mask |= InterruptMask::RX_AVAILABLE;
                }
            }
            UART_IIR_RDI | UART_IIR_CTI => {
                // 接收中断/超时中断，读取 RBR 清除
                let d = self.read_reg(UART_RBR);
                mask |= InterruptMask::RX_AVAILABLE;
                if self.rcv_fifo.push(d).is_err() {
                    self.err = Some(TransferError::Overrun(d));
                }
            }
            UART_IIR_THRI => {
                let ier = self.read_reg(UART_IER);
                // 关闭 THRI 使能位
                self.write_reg(UART_IER, ier & !UART_IER_THRI);

                mask |= InterruptMask::TX_EMPTY;
            }
            UART_IIR_MSI => {
                // Modem 状态中断，读取 MSR 清除
                let _ = self.read_reg(UART_MSR);
            }
            _ => {}
        }

        mask
    }

    fn line_status(&mut self) -> LineStatus {
        let lsr = self.read_reg(UART_LSR);
        let mut status = LineStatus::empty();

        if lsr & UART_LSR_DR != 0 {
            status |= LineStatus::DATA_READY;
        }
        if lsr & UART_LSR_THRE != 0 {
            status |= LineStatus::TX_HOLDING_EMPTY;
        }

        status
    }

    fn read_reg(&self, offset: usize) -> u32 {
        self.read_reg(offset as u8) as u32
    }

    fn write_reg(&mut self, offset: usize, value: u32) {
        self.write_reg(offset as u8, value as u8);
    }

    fn get_base(&self) -> usize {
        self.base.as_ptr() as usize
    }

    fn set_base(&mut self, base: usize) {
        self.base = NonNull::new(base as _).unwrap();
    }

    fn clock_freq(&self) -> u32 {
        self.clock_freq
    }

    fn enable_loopback(&mut self) {
        let original_mcr = self.read_reg(UART_MCR);
        self.write_reg(UART_MCR, original_mcr | UART_MCR_LOOP);
    }

    fn disable_loopback(&mut self) {
        let original_mcr = self.read_reg(UART_MCR);
        self.write_reg(UART_MCR, original_mcr & !UART_MCR_LOOP);
    }

    fn is_loopback_enabled(&self) -> bool {
        self.read_reg(UART_MCR) & UART_MCR_LOOP != 0
    }

    fn set_irq_mask(&mut self, mask: InterruptMask) {
        let mut ier = 0;
        self.is_tx_empty_int_enabled = false;

        if mask.contains(InterruptMask::RX_AVAILABLE) {
            ier |= UART_IER_RDI + UART_IER_RLSI;
        }
        if mask.contains(InterruptMask::TX_EMPTY) {
            ier |= UART_IER_THRI;
            self.is_tx_empty_int_enabled = true;
        }

        self.write_reg(UART_IER, ier);
    }

    fn get_irq_mask(&self) -> InterruptMask {
        let ier = self.read_reg(UART_IER);
        let mut mask = InterruptMask::empty();

        if ier & UART_IER_RDI != 0 {
            mask |= InterruptMask::RX_AVAILABLE;
        }
        if self.is_tx_empty_int_enabled {
            mask |= InterruptMask::TX_EMPTY;
        }
        // 错误中断暂不映射到 InterruptMask
        // 用户需要通过状态寄存器检查错误

        mask
    }
}
