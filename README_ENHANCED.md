# Enhanced Serial Register Interface

本文档描述了基于Linux内核uart_ops结构完善的SerialRegister接口，提供了完整的UART设备控制功能。

## 概述

通过分析Linux内核的uart_ops结构和现有SerialRegister接口的差异，我们设计了一个功能完整的串口控制接口，涵盖了Linux uart_ops的主要功能，同时保持了向后兼容性。

## 主要改进

### 1. 功能完整性对比

**原有接口 (4个方法):**
- `write_byte()` - 写入单字节
- `read_byte()` - 读取单字节
- `can_read()` - 检查是否可读
- `can_write()` - 检查是否可写

**完善后接口 (28个方法):**
- **基础数据传输** (4个) - 保持向后兼容
- **配置管理** (6个) - 波特率、数据位、停止位、奇偶校验
- **流控制** (6个) - RTS/CTS、DTR/DSR、调制解调器状态
- **中断管理** (4个) - 中断使能、状态查询、清除
- **传输状态查询** (6个) - FIFO级别、传输状态、错误状态
- **FIFO管理** (5个) - FIFO使能、触发级别、缓冲清空
- **DMA控制** (3个) - 高级DMA传输功能
- **电源管理** (2个) - 低功耗模式控制
- **寄存器访问** (3个) - 底层寄存器操作

### 2. 类型安全

添加了完整的类型定义：

```rust
// 配置枚举
pub enum DataBits { Five, Six, Seven, Eight }
pub enum StopBits { One, Two }
pub enum Parity { None, Even, Odd, Mark, Space }
pub enum PowerMode { Normal, LowPower, Off }
pub enum DmaDirection { Tx, Rx, Both }

// 位标志类型
pub struct InterruptMask: u32 { ... }
pub struct LineStatus: u32 { ... }
pub struct ModemStatus: u32 { ... }
pub struct DmaStatus: u32 { ... }

// 错误处理
pub enum SerialError { ... }
```

### 3. 寄存器抽象层

实现了`RegisterAccess` trait，提供统一的寄存器访问接口：

```rust
pub trait RegisterAccess: Clone + Send + Sync {
    // 基础访问
    unsafe fn read_reg_unsafe(&self, offset: usize) -> u32;
    unsafe fn write_reg_unsafe(&self, offset: usize, value: u32);

    // 安全访问（带内存屏障）
    fn read_reg_sync(&self, offset: usize) -> u32;
    fn write_reg_sync(&self, offset: usize, value: u32);

    // 位操作辅助
    fn modify_reg(&self, offset: usize, mask: u32, set: u32);
    fn set_reg_bits(&self, offset: usize, bits: u32);
    fn clear_reg_bits(&self, offset: usize, bits: u32);
    fn is_reg_bit_set(&self, offset: usize, bit: u8) -> bool;

    // 超时等待
    fn wait_for_bit_set(&self, offset: usize, bit: u8, timeout_us: u32) -> Result<(), SerialError>;
    fn wait_for_bit_clear(&self, offset: usize, bit: u8, timeout_us: u32) -> Result<(), SerialError>;

    // 时间戳
    fn get_timestamp_us(&self) -> u32;
}
```

### 4. 标准UART寄存器布局

定义了兼容16550、PL011等常见UART芯片的标准寄存器布局：

```rust
#[repr(C)]
pub struct UartRegisters {
    pub data: Volatile<u32>,           // 0x00 - 数据寄存器
    pub int_enable: Volatile<u32>,     // 0x04 - 中断使能
    pub int_ident_fifo: Volatile<u32>, // 0x08 - 中断标识/FIFO控制
    pub line_ctrl: Volatile<u32>,      // 0x0C - 线路控制
    pub modem_ctrl: Volatile<u32>,     // 0x10 - 调制解调器控制
    pub line_status: Volatile<u32>,    // 0x14 - 线路状态
    pub modem_status: Volatile<u32>,   // 0x18 - 调制解调器状态
    pub scratch: Volatile<u32>,        // 0x1C - 暂存寄存器
    // ... 扩展寄存器
}
```

### 5. 默认实现

为所有`RegisterAccess + BaudrateSupport`的实现者提供了完整的默认实现：

```rust
impl<T: RegisterAccess + BaudrateSupport> SerialRegister for T {
    // 完整的28个方法的默认实现
}
```

## 使用示例

### 基础配置

```rust
use some_serial::*;

// 创建UART实例
let uart = MemoryMappedUart::new(0x1000_0000, 18_432_000);

// 配置串口参数
uart.set_baudrate(115200)?;
uart.set_data_bits(DataBits::Eight)?;
uart.set_parity(Parity::None)?;
uart.set_stop_bits(StopBits::One)?;
uart.apply_config()?;

// 启用FIFO
uart.enable_fifo(true);
uart.set_fifo_trigger_level(8, 8);
```

### 流控制

```rust
// 启用硬件流控制
uart.set_rts(true);
uart.set_dtr(true);

// 查询流控制状态
let cts = uart.get_cts();
let dsr = uart.get_dsr();
let modem_status = uart.get_modem_status();
```

### 中断管理

```rust
// 启用中断
let mask = InterruptMask::RX_AVAILABLE | InterruptMask::TX_EMPTY;
uart.enable_interrupts(mask);

// 查询中断状态
let status = uart.get_interrupt_status();
```

### 状态查询

```rust
// 传输状态
let tx_empty = uart.is_tx_empty();
let rx_level = uart.get_rx_fifo_level();

// 错误状态
let line_status = uart.get_line_status();
if line_status.contains(LineStatus::PARITY_ERROR) {
    uart.clear_error();
}
```

### 直接寄存器访问

```rust
// 读取寄存器
let line_ctrl = uart.read_reg(0x0C / 4);

// 修改寄存器
uart.modify_reg(0x0C / 4, 0x03, 0x03); // 设置8数据位

// 位操作
uart.set_reg_bits(0x04 / 4, 0x01); // 设置第0位
uart.clear_reg_bits(0x04 / 4, 0x02); // 清除第1位
```

## 辅助工具

### 配置验证

```rust
// 验证配置兼容性
let is_valid = validate_serial_config(
    DataBits::Eight,
    StopBits::One,
    Parity::None
);

// 格式化配置信息
let config_str = format_serial_config(
    115200,
    DataBits::Eight,
    StopBits::One,
    Parity::None
);
// 结果: "115200 baud, 8-data bits, 1-stop bits, no-parity"
```

### 波特率计算

```rust
// 验证波特率
let is_valid = is_valid_baudrate(115200);

// FIFO触发级别推荐
let trigger_level = recommended_fifo_trigger_level(64); // 返回4
```

## 实现新UART芯片

要为新UART芯片实现支持，只需实现`RegisterAccess`和`BaudrateSupport` traits：

```rust
#[derive(Clone)]
pub struct MyUart {
    base: *mut u8,
    clock_freq: u32,
    // ... 其他字段
}

impl RegisterAccess for MyUart {
    unsafe fn read_reg_unsafe(&self, offset: usize) -> u32 {
        // 实现具体的寄存器读取
    }

    unsafe fn write_reg_unsafe(&self, offset: usize, value: u32) {
        // 实现具体的寄存器写入
    }

    fn get_timestamp_us(&self) -> u32 {
        // 实现时间戳获取
    }
}

impl BaudrateSupport for MyUart {
    fn calculate_baudrate_divisor(&self, baudrate: u32) -> u16 {
        // 实现波特率计算
    }

    fn calculate_baudrate_from_divisor(&self, divisor: u16) -> u32 {
        // 实现反向波特率计算
    }
}

// 现在MyUart自动获得了完整的SerialRegister接口！
```

## 向后兼容性

原有代码无需修改即可继续使用：

```rust
// 原有代码仍然有效
uart.write_byte(b'H');
uart.read_byte();
let can_read = uart.can_read();
let can_write = uart.can_write();
```

## 设计原则

1. **Linux兼容性** - 基于Linux uart_ops设计，保持与内核驱动的兼容性
2. **类型安全** - 使用强类型枚举和位标志，避免魔法数字
3. **错误处理** - 完整的Result类型错误处理
4. **可扩展性** - 易于添加新UART芯片支持
5. **性能考虑** - 提供unsafe快速路径和安全默认实现
6. **内存安全** - 适当的内存屏障和volatile操作
7. **向后兼容** - 不破坏现有API

## 文件结构

```
src/
├── lib.rs                    # 主要接口定义
├── volatile.rs              # Volatile类型实现
├── registers.rs             # 寄存器布局定义
├── access.rs                # RegisterAccess trait
├── default_impl.rs          # 默认实现
├── utils.rs                 # 辅助函数
└── examples/
    └── memory_mapped_uart.rs # 示例实现

tests/
└── test.rs                  # 测试用例

examples/
└── uart_usage.rs            # 使用示例
```

## 总结

完善后的SerialRegister接口提供了：

- ✅ **功能完整性** - 覆盖Linux uart_ops的主要功能
- ✅ **通用性** - 支持不同UART芯片的统一接口
- ✅ **可扩展性** - 易于添加新的UART类型支持
- ✅ **向后兼容** - 现有代码无需修改
- ✅ **类型安全** - 强类型API和错误处理
- ✅ **性能优化** - 提供快速路径和安全默认实现
- ✅ **工具支持** - 配置验证、格式化等辅助函数

这个接口设计为Rust嵌入式和bare-metal串口编程提供了一个强大、安全、易用的解决方案。