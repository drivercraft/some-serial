//! NS16550/16450 UART 寄存器定义
//!
//! 参考Linux内核 drivers/tty/serial/8250/8250.h
//! 使用 const 定义寄存器偏移和位标志

#![allow(dead_code)]

// 寄存器偏移 (相对于基地址)
/// UART_RBR: 接收缓冲寄存器 (Receiver Buffer Register)
/// 只读，读取时会清除接收中断。
pub const UART_RBR: u8 = 0x00;

/// UART_THR: 发送保持寄存器 (Transmitter Holding Register)
/// 只写，写入数据会触发发送。
pub const UART_THR: u8 = 0x00;

/// UART_DLL: 除数锁存低字节 (Divisor Latch LSB)
/// 可读可写，设置波特率除数低8位，需先设置LCR.DLAB=1。
pub const UART_DLL: u8 = 0x00;

/// UART_IER: 中断使能寄存器 (Interrupt Enable Register)
/// 可读可写，控制各类中断使能。
pub const UART_IER: u8 = 0x01;

/// UART_DLH: 除数锁存高字节 (Divisor Latch MSB)
/// 可读可写，设置波特率除数高8位，需先设置LCR.DLAB=1。
pub const UART_DLH: u8 = 0x01;

/// UART_IIR: 中断标识寄存器 (Interrupt Identification Register)
/// 只读，查询当前挂起的中断类型，读取不会清除中断。
pub const UART_IIR: u8 = 0x02;

/// UART_FCR: FIFO控制寄存器 (FIFO Control Register)
/// 只写，控制FIFO使能、清空等。
pub const UART_FCR: u8 = 0x02;

/// UART_LCR: 线路控制寄存器 (Line Control Register)
/// 可读可写，配置数据位、停止位、校验、DLAB等。
pub const UART_LCR: u8 = 0x03;

/// UART_MCR: 调制解调器控制寄存器 (Modem Control Register)
/// 可读可写，控制RTS/DTR/环回等。
pub const UART_MCR: u8 = 0x04;

/// UART_LSR: 线路状态寄存器 (Line Status Register)
/// 只读，反映收发状态和错误，读取可清除部分错误中断。
pub const UART_LSR: u8 = 0x05;

/// UART_MSR: 调制解调器状态寄存器 (Modem Status Register)
/// 只读，反映调制解调器信号状态，读取可清除部分调制解调器中断。
pub const UART_MSR: u8 = 0x06;

/// UART_SCR: 临时寄存器 (Scratch Register)
/// 可读可写，用户自定义用途，无实际硬件功能。
pub const UART_SCR: u8 = 0x07;

// IER (Interrupt Enable Register) 位定义
pub const UART_IER_RDI: u8 = 0x01; // Enable Received Data Available Interrupt
pub const UART_IER_THRI: u8 = 0x02; // Enable Transmitter Holding Register Empty Interrupt
pub const UART_IER_RLSI: u8 = 0x04; // Enable Receiver Line Status Interrupt
pub const UART_IER_MSI: u8 = 0x08; // Enable Modem Status Interrupt

// IIR (Interrupt Identification Register) 位定义
pub const UART_IIR_NO_INT: u8 = 0x01; // No interrupts pending
pub const UART_IIR_ID: u8 = 0x0E; // Interrupt ID mask
pub const UART_IIR_RLSI: u8 = 0x06; // Receiver Line Status Interrupt
pub const UART_IIR_RDI: u8 = 0x04; // Received Data Available Interrupt
pub const UART_IIR_CTI: u8 = 0x0C; // Character Timeout Indicator
pub const UART_IIR_THRI: u8 = 0x02; // Transmitter Holding Register Empty Interrupt
pub const UART_IIR_MSI: u8 = 0x00; // Modem Status Interrupt
pub const UART_IIR_FIFO_ENABLE: u8 = 0xC0; // FIFO Enable bits
pub const UART_IIR_FIFO_MASK: u8 = 0xC0; // FIFO bits mask

// FCR (FIFO Control Register) 位定义
pub const UART_FCR_ENABLE_FIFO: u8 = 0x01; // Enable FIFO
pub const UART_FCR_CLEAR_RCVR: u8 = 0x02; // Clear receiver FIFO
pub const UART_FCR_CLEAR_XMIT: u8 = 0x04; // Clear transmitter FIFO
pub const UART_FCR_DMA_SELECT: u8 = 0x08; // DMA mode select
pub const UART_FCR_TRIGGER_MASK: u8 = 0xC0; // Trigger level mask
pub const UART_FCR_TRIGGER_1: u8 = 0x00; // 1 byte trigger
pub const UART_FCR_TRIGGER_4: u8 = 0x40; // 4 byte trigger
pub const UART_FCR_TRIGGER_8: u8 = 0x80; // 8 byte trigger
pub const UART_FCR_TRIGGER_14: u8 = 0xC0; // 14 byte trigger

// LCR (Line Control Register) 位定义
pub const UART_LCR_WLEN5: u8 = 0x00; // 5 bits
pub const UART_LCR_WLEN6: u8 = 0x01; // 6 bits
pub const UART_LCR_WLEN7: u8 = 0x02; // 7 bits
pub const UART_LCR_WLEN8: u8 = 0x03; // 8 bits
pub const UART_LCR_STOP: u8 = 0x04; // Stop bits: 0=1 bit, 1=2 bits
pub const UART_LCR_PARITY: u8 = 0x08; // Parity enable
pub const UART_LCR_EPAR: u8 = 0x10; // Even parity
pub const UART_LCR_SPAR: u8 = 0x20; // Stick parity
pub const UART_LCR_SBRK: u8 = 0x40; // Set Break
pub const UART_LCR_DLAB: u8 = 0x80; // Divisor latch access bit

// MCR (Modem Control Register) 位定义
pub const UART_MCR_DTR: u8 = 0x01; // Data Terminal Ready
pub const UART_MCR_RTS: u8 = 0x02; // Request to Send
pub const UART_MCR_OUT1: u8 = 0x04; // Out 1
pub const UART_MCR_OUT2: u8 = 0x08; // Out 2
pub const UART_MCR_LOOP: u8 = 0x10; // Enable loopback test mode

// LSR (Line Status Register) 位定义
pub const UART_LSR_DR: u8 = 0x01; // Data ready
pub const UART_LSR_OE: u8 = 0x02; // Overrun error
pub const UART_LSR_PE: u8 = 0x04; // Parity error
pub const UART_LSR_FE: u8 = 0x08; // Framing error
pub const UART_LSR_BI: u8 = 0x10; // Break interrupt
pub const UART_LSR_THRE: u8 = 0x20; // Transmitter holding register empty
pub const UART_LSR_TEMT: u8 = 0x40; // Transmitter empty
pub const UART_LSR_FIFOE: u8 = 0x80; // Fifo error indication

// MSR (Modem Status Register) 位定义
pub const UART_MSR_DCTS: u8 = 0x01; // Delta CTS
pub const UART_MSR_DDSR: u8 = 0x02; // Delta DSR
pub const UART_MSR_TERI: u8 = 0x04; // Trail edge ring indicator
pub const UART_MSR_DDCD: u8 = 0x08; // Delta DCD
pub const UART_MSR_CTS: u8 = 0x10; // Clear to Send
pub const UART_MSR_DSR: u8 = 0x20; // Data Set Ready
pub const UART_MSR_RI: u8 = 0x40; // Ring Indicator
pub const UART_MSR_DCD: u8 = 0x80; // Data Carrier Detect

// 默认波特率除数（假设输入时钟 1.8432MHz）
pub const UART_DEFAULT_BAUD_RATE: u32 = 9600;
pub const UART_INPUT_CLOCK: u32 = 1_843_200;
pub const UART_DEFAULT_DIVISOR: u16 = (UART_INPUT_CLOCK / (16 * UART_DEFAULT_BAUD_RATE)) as u16;

// FIFO 深度
pub const UART_FIFO_SIZE: u8 = 16;

// 通用寄存器访问掩码
pub const UART_LCR_WLEN_MASK: u8 = 0x03;
pub const UART_IIR_INTERRUPT_MASK: u8 = 0x0E;
pub const UART_MCR_MODEM_MASK: u8 = 0x0F;
pub const UART_LSR_ERROR_MASK: u8 = 0x1E;
pub const UART_MSR_DELTA_MASK: u8 = 0x0F;
pub const UART_MSR_STATUS_MASK: u8 = 0xF0;
