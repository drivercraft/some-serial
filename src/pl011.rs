use core::ptr::NonNull;

use crate::Register;
use rdif_serial::{RegisterTransferError as TransferError, Serial, SerialRaw};
use tock_registers::{interfaces::*, register_bitfields, register_structs, registers::*};

use crate::{Config, ConfigError, DataBits, InterruptMask, LineStatus, Parity, StopBits};

register_bitfields! [
    u32,

    /// Data Register
    UARTDR [
        DATA OFFSET(0) NUMBITS(8) [],
        FE OFFSET(8) NUMBITS(1) [],
        PE OFFSET(9) NUMBITS(1) [],
        BE OFFSET(10) NUMBITS(1) [],
        OE OFFSET(11) NUMBITS(1) []
    ],

    /// Receive Status Register / Error Clear Register
    UARTRSR_ECR [
        FE OFFSET(0) NUMBITS(1) [],
        PE OFFSET(1) NUMBITS(1) [],
        BE OFFSET(2) NUMBITS(1) [],
        OE OFFSET(3) NUMBITS(1) []
    ],

    /// Flag Register
    UARTFR [
        CTS OFFSET(0) NUMBITS(1) [],
        DSR OFFSET(1) NUMBITS(1) [],
        DCD OFFSET(2) NUMBITS(1) [],
        BUSY OFFSET(3) NUMBITS(1) [],
        RXFE OFFSET(4) NUMBITS(1) [],
        TXFF OFFSET(5) NUMBITS(1) [],
        RXFF OFFSET(6) NUMBITS(1) [],
        TXFE OFFSET(7) NUMBITS(1) [],
        RI OFFSET(8) NUMBITS(1) []
    ],

    /// Integer Baud Rate Register
    UARTIBRD [
        BAUD_DIVINT OFFSET(0) NUMBITS(16) []
    ],

    /// Fractional Baud Rate Register
    UARTFBRD [
        BAUD_DIVFRAC OFFSET(0) NUMBITS(6) []
    ],

    /// Line Control Register
    UARTLCR_H [
        BRK OFFSET(0) NUMBITS(1) [],
        PEN OFFSET(1) NUMBITS(1) [],
        EPS OFFSET(2) NUMBITS(1) [],
        STP2 OFFSET(3) NUMBITS(1) [],
        FEN OFFSET(4) NUMBITS(1) [],
        WLEN OFFSET(5) NUMBITS(2) [
            FiveBit = 0,
            SixBit = 1,
            SevenBit = 2,
            EightBit = 3
        ],
        SPS OFFSET(7) NUMBITS(1) []
    ],

    /// Control Register
    UARTCR [
        UARTEN OFFSET(0) NUMBITS(1) [],
        SIREN OFFSET(1) NUMBITS(1) [],
        SIRLP OFFSET(2) NUMBITS(1) [],
        LBE OFFSET(7) NUMBITS(1) [],
        TXE OFFSET(8) NUMBITS(1) [],
        RXE OFFSET(9) NUMBITS(1) [],
        DTR OFFSET(10) NUMBITS(1) [],
        RTS OFFSET(11) NUMBITS(1) [],
        OUT1 OFFSET(12) NUMBITS(1) [],
        OUT2 OFFSET(13) NUMBITS(1) [],
        RTSEN OFFSET(14) NUMBITS(1) [],
        CTSEN OFFSET(15) NUMBITS(1) []
    ],

    /// Interrupt FIFO Level Select Register
    UARTIFLS [
        TXIFLSEL OFFSET(0) NUMBITS(3) [],
        RXIFLSEL OFFSET(3) NUMBITS(3) []
    ],

    /// Interrupt Mask Set/Clear Register
    UARTIMSC [
        RIMIM OFFSET(0) NUMBITS(1) [],
        CTSMIM OFFSET(1) NUMBITS(1) [],
        DCDMIM OFFSET(2) NUMBITS(1) [],
        DSRMIM OFFSET(3) NUMBITS(1) [],
        RXIM OFFSET(4) NUMBITS(1) [],
        TXIM OFFSET(5) NUMBITS(1) [],
        RTIM OFFSET(6) NUMBITS(1) [],
        FEIM OFFSET(7) NUMBITS(1) [],
        PEIM OFFSET(8) NUMBITS(1) [],
        BEIM OFFSET(9) NUMBITS(1) [],
        OEIM OFFSET(10) NUMBITS(1) []
    ],

    /// Raw Interrupt Status Register
    UARTRIS [
        RIRMIS OFFSET(0) NUMBITS(1) [],
        CTSRMIS OFFSET(1) NUMBITS(1) [],
        DCDRMIS OFFSET(2) NUMBITS(1) [],
        DSRRMIS OFFSET(3) NUMBITS(1) [],
        RXRIS OFFSET(4) NUMBITS(1) [],
        TXRIS OFFSET(5) NUMBITS(1) [],
        RTRIS OFFSET(6) NUMBITS(1) [],
        FERIS OFFSET(7) NUMBITS(1) [],
        PERIS OFFSET(8) NUMBITS(1) [],
        BERIS OFFSET(9) NUMBITS(1) [],
        OERIS OFFSET(10) NUMBITS(1) []
    ],

    /// Masked Interrupt Status Register
    UARTMIS [
        RIMMIS OFFSET(0) NUMBITS(1) [],
        CTSMIS OFFSET(1) NUMBITS(1) [],
        DCDMIS OFFSET(2) NUMBITS(1) [],
        DSRMIS OFFSET(3) NUMBITS(1) [],
        RXMIS OFFSET(4) NUMBITS(1) [],
        TXMIS OFFSET(5) NUMBITS(1) [],
        RTMIS OFFSET(6) NUMBITS(1) [],
        FEMIS OFFSET(7) NUMBITS(1) [],
        PEMIS OFFSET(8) NUMBITS(1) [],
        BEMIS OFFSET(9) NUMBITS(1) [],
        OEMIS OFFSET(10) NUMBITS(1) []
    ],

    /// Interrupt Clear Register
    UARTICR [
        RIMIC OFFSET(0) NUMBITS(1) [],
        CTSMIC OFFSET(1) NUMBITS(1) [],
        DCDMIC OFFSET(2) NUMBITS(1) [],
        DSRMIC OFFSET(3) NUMBITS(1) [],
        RXIC OFFSET(4) NUMBITS(1) [],
        TXIC OFFSET(5) NUMBITS(1) [],
        RTIC OFFSET(6) NUMBITS(1) [],
        FEIC OFFSET(7) NUMBITS(1) [],
        PEIC OFFSET(8) NUMBITS(1) [],
        BEIC OFFSET(9) NUMBITS(1) [],
        OEIC OFFSET(10) NUMBITS(1) []
    ],

    /// DMA Control Register
    UARTDMACR [
        RXDMAE OFFSET(0) NUMBITS(1) [],
        TXDMAE OFFSET(1) NUMBITS(1) [],
        DMAONERR OFFSET(2) NUMBITS(1) []
    ]
];

register_structs! {
    pub Pl011Registers {
        (0x000 => uartdr: ReadWrite<u32, UARTDR::Register>),
        (0x004 => uartrsr_ecr: ReadWrite<u32, UARTRSR_ECR::Register>),
        (0x008 => _reserved1),
        (0x018 => uartfr: ReadOnly<u32, UARTFR::Register>),
        (0x01c => _reserved2),
        (0x020 => uartilpr: ReadWrite<u32>),
        (0x024 => uartibrd: ReadWrite<u32, UARTIBRD::Register>),
        (0x028 => uartfbrd: ReadWrite<u32, UARTFBRD::Register>),
        (0x02c => uartlcr_h: ReadWrite<u32, UARTLCR_H::Register>),
        (0x030 => uartcr: ReadWrite<u32, UARTCR::Register>),
        (0x034 => uartifls: ReadWrite<u32, UARTIFLS::Register>),
        (0x038 => uartimsc: ReadWrite<u32, UARTIMSC::Register>),
        (0x03c => uartris: ReadOnly<u32, UARTRIS::Register>),
        (0x040 => uartmis: ReadOnly<u32, UARTMIS::Register>),
        (0x044 => uarticr: WriteOnly<u32, UARTICR::Register>),
        (0x048 => uartdmacr: ReadWrite<u32, UARTDMACR::Register>),
        (0x04c => _reserved3),
        (0x1000 => @END),
    }
}

// SAFETY: PL011 寄存器访问是原子的，硬件保证了内存映射寄存器的线程安全
unsafe impl Sync for Pl011Registers {}

/// PL011 UART 驱动结构体
#[derive(Clone)]
pub struct Pl011 {
    base: NonNull<Pl011Registers>,
    clock_freq: u32,
}

unsafe impl Send for Pl011 {}
unsafe impl Sync for Pl011 {}

impl Pl011 {
    /// 创建新的 PL011 实例（仅基地址，使用默认配置）
    ///
    /// # Arguments
    /// * `base` - UART 寄存器基地址
    pub fn new_raw_no_clock(base: NonNull<u8>) -> SerialRaw<Self> {
        // 自动检测时钟频率或使用合理的默认值
        let clock_freq = Self::detect_clock_frequency(base.as_ptr() as usize);
        Self::new_raw(base, clock_freq)
    }

    /// 创建新的 PL011 实例（指定时钟频率）
    ///
    /// # Arguments
    /// * `base` - UART 寄存器基地址
    /// * `clock_freq` - UART 时钟频率 (Hz)
    pub fn new_raw(base: NonNull<u8>, clock_freq: u32) -> SerialRaw<Self> {
        SerialRaw::new(Self {
            base: base.cast(),
            clock_freq,
        })
    }

    pub fn new(base: NonNull<u8>, clock_freq: u32) -> Serial<Self> {
        Serial::new(Self {
            base: base.cast(),
            clock_freq,
        })
    }

    fn registers(&self) -> &Pl011Registers {
        unsafe { self.base.as_ref() }
    }

    /// 自动检测或确定合理的时钟频率
    fn detect_clock_frequency(base: usize) -> u32 {
        // 尝试读取当前波特率设置来反向推算时钟频率
        let registers = unsafe { &*(base as *const Pl011Registers) };

        use tock_registers::interfaces::Readable;
        let ibrd = registers.uartibrd.read(UARTIBRD::BAUD_DIVINT);

        // 如果有设置值，假设波特率为 115200 来估算时钟频率
        if ibrd > 0 && ibrd <= 0xFFFF {
            // 假设波特率为 115200，计算时钟频率
            // FUARTCLK = 16 * BAUDDIV * Baud rate
            let estimated_clock = 16 * ibrd * 115200;

            // 合理的时钟频率范围：1MHz - 100MHz
            if (1_000_000..=100_000_000).contains(&estimated_clock) {
                return estimated_clock;
            }
        }

        // 默认使用 24MHz（最常见）
        24_000_000
    }

    // 内部私有方法，用于配置
    fn set_baudrate_internal(&self, baudrate: u32) -> Result<(), ConfigError> {
        // PL011 波特率计算公式：
        // BAUDDIV = (FUARTCLK / (16 * Baud rate))
        // IBRD = integer(BAUDDIV)
        // FBRD = integer((BAUDDIV - IBRD) * 64 + 0.5)

        let bauddiv = self.clock_freq / (16 * baudrate);
        let remainder = self.clock_freq % (16 * baudrate);
        let fbrd = (remainder * 64 + (16 * baudrate / 2)) / (16 * baudrate);

        if bauddiv == 0 || bauddiv > 0xFFFF {
            return Err(ConfigError::InvalidBaudrate);
        }

        self.registers()
            .uartibrd
            .write(UARTIBRD::BAUD_DIVINT.val(bauddiv));
        self.registers()
            .uartfbrd
            .write(UARTFBRD::BAUD_DIVFRAC.val(fbrd));

        Ok(())
    }

    fn set_data_bits_internal(&self, bits: DataBits) -> Result<(), ConfigError> {
        let wlen = match bits {
            DataBits::Five => UARTLCR_H::WLEN::FiveBit,
            DataBits::Six => UARTLCR_H::WLEN::SixBit,
            DataBits::Seven => UARTLCR_H::WLEN::SevenBit,
            DataBits::Eight => UARTLCR_H::WLEN::EightBit,
        };

        self.registers().uartlcr_h.modify(wlen);
        Ok(())
    }

    fn set_stop_bits_internal(&self, bits: StopBits) -> Result<(), ConfigError> {
        match bits {
            StopBits::One => self.registers().uartlcr_h.modify(UARTLCR_H::STP2::CLEAR),
            StopBits::Two => self.registers().uartlcr_h.modify(UARTLCR_H::STP2::SET),
        }

        Ok(())
    }

    fn set_parity_internal(&self, parity: Parity) -> Result<(), ConfigError> {
        match parity {
            Parity::None => {
                // PEN = 0, 无奇偶校验
                self.registers().uartlcr_h.modify(UARTLCR_H::PEN::CLEAR);
            }
            Parity::Odd => {
                // PEN = 1, EPS = 0 (奇校验), SPS = 0
                self.registers()
                    .uartlcr_h
                    .modify(UARTLCR_H::PEN::SET + UARTLCR_H::EPS::CLEAR + UARTLCR_H::SPS::CLEAR);
            }
            Parity::Even => {
                // PEN = 1, EPS = 1 (偶校验), SPS = 0
                self.registers()
                    .uartlcr_h
                    .modify(UARTLCR_H::PEN::SET + UARTLCR_H::EPS::SET + UARTLCR_H::SPS::CLEAR);
            }
            Parity::Mark => {
                // PEN = 1, SPS = 1, EPS = 0 (奇校验)
                self.registers()
                    .uartlcr_h
                    .modify(UARTLCR_H::PEN::SET + UARTLCR_H::EPS::CLEAR + UARTLCR_H::SPS::SET);
            }
            Parity::Space => {
                // PEN = 1, EPS = 1 (偶校验), SPS = 1
                self.registers()
                    .uartlcr_h
                    .modify(UARTLCR_H::PEN::SET + UARTLCR_H::EPS::SET + UARTLCR_H::SPS::SET);
            }
        }

        Ok(())
    }

    /// 初始化 PL011 UART
    fn init(&self) {
        // 禁用 UART
        self.registers().uartcr.modify(UARTCR::UARTEN::CLEAR);

        // 等待当前传输完成
        while self.registers().uartfr.is_set(UARTFR::BUSY) {
            core::hint::spin_loop();
        }

        // 清除发送 FIFO
        self.registers().uartlcr_h.modify(UARTLCR_H::FEN::CLEAR);

        // 启用 FIFO
        self.registers().uartlcr_h.modify(UARTLCR_H::FEN::SET);

        // 启用 UART
        self.registers()
            .uartcr
            .modify(UARTCR::UARTEN::SET + UARTCR::TXE::SET + UARTCR::RXE::SET);
    }
}

impl Register for Pl011 {
    fn write_byte(&mut self, byte: u8) {
        self.registers().uartdr.write(UARTDR::DATA.val(byte as u32));
    }

    fn read_byte(&self) -> Result<u8, TransferError> {
        let dr = self.registers().uartdr.extract();
        let data = dr.read(UARTDR::DATA) as u8;
        if dr.is_set(UARTDR::FE) {
            return Err(TransferError::Framing);
        }

        if dr.is_set(UARTDR::PE) {
            return Err(TransferError::Parity);
        }

        if dr.is_set(UARTDR::OE) {
            return Err(TransferError::Overrun(data));
        }

        if dr.is_set(UARTDR::BE) {
            return Err(TransferError::Break);
        }

        Ok(data)
    }

    fn set_config(&mut self, config: &Config) -> Result<(), ConfigError> {
        use tock_registers::interfaces::Readable;

        // 根据ARM文档的建议配置流程：
        // 1. 禁用UART
        let original_enable = self.registers().uartcr.is_set(UARTCR::UARTEN); // 保存原始使能状态
        self.registers().uartcr.modify(UARTCR::UARTEN::CLEAR); // 禁用UART

        // 2. 等待当前字符传输完成
        while self.registers().uartfr.is_set(UARTFR::BUSY) {
            core::hint::spin_loop();
        }

        // 3. 刷新发送FIFO（通过设置FEN=0）
        self.registers().uartlcr_h.modify(UARTLCR_H::FEN::CLEAR);

        // 4. 配置各项参数
        if let Some(baudrate) = config.baudrate {
            self.set_baudrate_internal(baudrate)?;
        }
        if let Some(data_bits) = config.data_bits {
            self.set_data_bits_internal(data_bits)?;
        }
        if let Some(stop_bits) = config.stop_bits {
            self.set_stop_bits_internal(stop_bits)?;
        }
        if let Some(parity) = config.parity {
            self.set_parity_internal(parity)?;
        }

        // 5. 重新启用FIFO
        self.registers().uartlcr_h.modify(UARTLCR_H::FEN::SET);

        // 6. 恢复UART使能状态
        if original_enable {
            self.registers().uartcr.modify(UARTCR::UARTEN::SET); // 重新启用UART
        }

        Ok(())
    }

    fn baudrate(&self) -> u32 {
        let ibrd = self.registers().uartibrd.read(UARTIBRD::BAUD_DIVINT);
        let fbrd = self.registers().uartfbrd.read(UARTFBRD::BAUD_DIVFRAC);

        // 反向计算波特率
        // Baud rate = FUARTCLK / (16 * (IBRD + FBRD/64))
        let divisor = ibrd * 64 + fbrd;
        if divisor == 0 {
            return 0;
        }

        self.clock_freq * 64 / (16 * divisor)
    }

    fn data_bits(&self) -> DataBits {
        let wlen = self.registers().uartlcr_h.read(UARTLCR_H::WLEN);

        match wlen {
            0 => DataBits::Five,
            1 => DataBits::Six,
            2 => DataBits::Seven,
            3 => DataBits::Eight,
            _ => DataBits::Eight, // 默认值
        }
    }

    fn stop_bits(&self) -> StopBits {
        if self.registers().uartlcr_h.is_set(UARTLCR_H::STP2) {
            StopBits::Two
        } else {
            StopBits::One
        }
    }

    fn parity(&self) -> Parity {
        if !self.registers().uartlcr_h.is_set(UARTLCR_H::PEN) {
            Parity::None
        } else if self.registers().uartlcr_h.is_set(UARTLCR_H::SPS) {
            // Stick parity
            if self.registers().uartlcr_h.is_set(UARTLCR_H::EPS) {
                Parity::Space
            } else {
                Parity::Mark
            }
        } else {
            // Normal parity
            if self.registers().uartlcr_h.is_set(UARTLCR_H::EPS) {
                Parity::Even
            } else {
                Parity::Odd
            }
        }
    }

    fn open(&mut self) {
        self.init()
    }

    fn close(&mut self) {
        // 禁用 UART
        self.registers().uartcr.modify(UARTCR::UARTEN::CLEAR);
    }

    fn enable_interrupts(&mut self, mask: InterruptMask) {
        let mut imsc = 0u32;

        if mask.contains(InterruptMask::RX_AVAILABLE) {
            imsc |= 1 << 4; // RXIM
        }
        if mask.contains(InterruptMask::TX_EMPTY) {
            imsc |= 1 << 5; // TXIM
        }
        // if mask.contains(InterruptMask::RX_LINE_STATUS) {
        //     imsc |= (1 << 7) | (1 << 8) | (1 << 9) | (1 << 10); // FEIM, PEIM, BEIM, OEIM
        // }
        // if mask.contains(InterruptMask::MODEM_STATUS) {
        //     imsc |= (1 << 0) | (1 << 1) | (1 << 2) | (1 << 3); // RIMIM, CTSMIM, DCDMIM, DSRMIM
        // }
        // if mask.contains(InterruptMask::CHARACTER_TIMEOUT) {
        //     imsc |= 1 << 6; // RTIM
        // }

        self.registers().uartimsc.set(imsc);
    }

    fn disable_interrupts(&mut self, mask: InterruptMask) {
        let mut imsc = 0u32;

        if mask.contains(InterruptMask::RX_AVAILABLE) {
            imsc |= 1 << 4; // RXIM
        }
        if mask.contains(InterruptMask::TX_EMPTY) {
            imsc |= 1 << 5; // TXIM
        }
        // if mask.contains(InterruptMask::RX_LINE_STATUS) {
        //     imsc |= (1 << 7) | (1 << 8) | (1 << 9) | (1 << 10); // FEIM, PEIM, BEIM, OEIM
        // }
        // if mask.contains(InterruptMask::MODEM_STATUS) {
        //     imsc |= (1 << 0) | (1 << 1) | (1 << 2) | (1 << 3); // RIMIM, CTSMIM, DCDMIM, DSRMIM
        // }
        // if mask.contains(InterruptMask::CHARACTER_TIMEOUT) {
        //     imsc |= 1 << 6; // RTIM
        // }

        // 读取当前值并清除相应的中断掩码位
        let current = self.registers().uartimsc.get();
        self.registers().uartimsc.set(current & !imsc);
    }

    fn clean_interrupt_status(&mut self) -> InterruptMask {
        use tock_registers::interfaces::Readable;

        let mis = self.registers().uartmis.get();
        let mut mask = InterruptMask::empty();

        if mis & (1 << 4) != 0 {
            mask |= InterruptMask::RX_AVAILABLE;
        }
        if mis & (1 << 5) != 0 {
            mask |= InterruptMask::TX_EMPTY;
        }
        // if mis & ((1 << 7) | (1 << 8) | (1 << 9) | (1 << 10)) != 0 {
        //     mask |= InterruptMask::RX_LINE_STATUS;
        // }
        // if mis & ((1 << 0) | (1 << 1) | (1 << 2) | (1 << 3)) != 0 {
        //     mask |= InterruptMask::MODEM_STATUS;
        // }
        // if mis & (1 << 6) != 0 {
        //     mask |= InterruptMask::CHARACTER_TIMEOUT;
        // }

        // 清除所有中断状态
        self.registers().uarticr.set(0x7FF); // 清除所有可清除的中断

        mask
    }

    fn line_status(&self) -> LineStatus {
        use tock_registers::interfaces::Readable;
        let mut status = LineStatus::empty();

        let fr = self.registers().uartfr.extract();

        if !fr.is_set(UARTFR::RXFE) {
            status |= LineStatus::DATA_READY;
        }

        if !fr.is_set(UARTFR::TXFF) {
            status |= LineStatus::TX_HOLDING_EMPTY;
        }
        status
    }

    fn read_reg(&self, offset: usize) -> u32 {
        let addr = unsafe { self.base.cast::<u8>().add(offset) };
        unsafe { addr.cast().read_volatile() }
    }

    fn write_reg(&mut self, offset: usize, value: u32) {
        let addr = unsafe { self.base.cast::<u8>().add(offset) };
        unsafe { addr.cast().write_volatile(value) };
    }

    fn get_base(&self) -> usize {
        self.base.as_ptr() as usize
    }

    fn set_base(&mut self, base: NonNull<u8>) {
        self.base = base.cast();
    }

    fn clock_freq(&self) -> u32 {
        self.clock_freq
    }

    fn enable_loopback(&mut self) {
        self.registers().uartcr.modify(UARTCR::LBE::SET);
    }

    fn disable_loopback(&mut self) {
        self.registers().uartcr.modify(UARTCR::LBE::CLEAR);
    }

    fn is_loopback_enabled(&self) -> bool {
        self.registers().uartcr.is_set(UARTCR::LBE)
    }
}

// 额外的便利方法，用于 FIFO 和流控制
impl Pl011 {
    /// 启用或禁用 FIFO
    pub fn enable_fifo(&self, enable: bool) {
        if enable {
            self.registers().uartlcr_h.modify(UARTLCR_H::FEN::SET);
        } else {
            self.registers().uartlcr_h.modify(UARTLCR_H::FEN::CLEAR);
        }
    }

    /// 设置 FIFO 触发级别
    pub fn set_fifo_trigger_level(&self, rx_level: u8, tx_level: u8) {
        // PL011 FIFO 触发级别：
        // 0b000: 1/8 full
        // 0b001: 1/4 full
        // 0b010: 1/2 full
        // 0b011: 3/4 full
        // 0b100: 7/8 full

        let rx_iflsel = match rx_level {
            0..=2 => 0b000,  // 1/8
            3..=4 => 0b001,  // 1/4
            5..=8 => 0b010,  // 1/2
            9..=12 => 0b011, // 3/4
            _ => 0b100,      // 7/8
        };

        let tx_iflsel = match tx_level {
            0..=2 => 0b000,  // 1/8
            3..=4 => 0b001,  // 1/4
            5..=8 => 0b010,  // 1/2
            9..=12 => 0b011, // 3/4
            _ => 0b100,      // 7/8
        };

        self.registers()
            .uartifls
            .write(UARTIFLS::RXIFLSEL.val(rx_iflsel) + UARTIFLS::TXIFLSEL.val(tx_iflsel));
    }
}

// ModemStatus 现在在 lib.rs 中定义，这里只是导出
