//! 串口设备使用示例
//!
//! 本示例展示了如何使用完善后的SerialRegister接口来操作UART设备

#![no_std]
#![no_main]

extern crate alloc;

use some_serial::{
    BaudrateSupport, DataBits, DmaDirection, InterruptMask, LineStatus, MemoryMappedUart,
    ModemStatus, Parity, PowerMode, RegisterAccess, SerialRegister, StopBits, format_serial_config,
    is_valid_baudrate, recommended_fifo_trigger_level, validate_serial_config,
};

/// 演示基础串口配置
fn demo_basic_config(uart: &impl SerialRegister) {
    println!("=== 基础串口配置示例 ===");

    // 设置波特率
    if let Err(e) = uart.set_baudrate(115200) {
        println!("设置波特率失败: {:?}", e);
        return;
    }

    // 配置数据格式: 8N1 (8数据位, 无奇偶校验, 1停止位)
    if let Err(e) = uart.set_data_bits(DataBits::Eight) {
        println!("设置数据位失败: {:?}", e);
        return;
    }

    if let Err(e) = uart.set_parity(Parity::None) {
        println!("设置奇偶校验失败: {:?}", e);
        return;
    }

    if let Err(e) = uart.set_stop_bits(StopBits::One) {
        println!("设置停止位失败: {:?}", e);
        return;
    }

    // 应用配置
    if let Err(e) = uart.apply_config() {
        println!("应用配置失败: {:?}", e);
        return;
    }

    println!("基础配置完成");

    // 读取并显示当前配置
    let current_baudrate = uart.get_baudrate();
    let config_str = format_serial_config(
        current_baudrate,
        DataBits::Eight,
        StopBits::One,
        Parity::None,
    );
    println!("当前配置: {}", config_str);
}

/// 演示FIFO配置
fn demo_fifo_config(uart: &impl SerialRegister) {
    println!("\n=== FIFO配置示例 ===");

    // 启用FIFO
    uart.enable_fifo(true);
    println!("FIFO已启用");

    // 设置FIFO触发级别
    let rx_trigger = 8;
    let tx_trigger = 8;
    uart.set_fifo_trigger_level(rx_trigger, tx_trigger);
    println!("FIFO触发级别设置: RX={}, TX={}", rx_trigger, tx_trigger);

    // 查询FIFO状态
    let rx_level = uart.get_rx_fifo_level();
    let tx_level = uart.get_tx_fifo_level();
    println!("当前FIFO级别: RX={}, TX={}", rx_level, tx_level);
}

/// 演示流控制
fn demo_flow_control(uart: &impl SerialRegister) {
    println!("\n=== 流控制示例 ===");

    // 启用RTS和DTR信号
    uart.set_rts(true);
    uart.set_dtr(true);
    println!("RTS和DTR信号已启用");

    // 查询CTS和DSR状态
    let cts = uart.get_cts();
    let dsr = uart.get_dsr();
    let dcd = uart.get_dcd();
    let ri = uart.get_ri();

    println!("调制解调器状态:");
    println!("  CTS (Clear to Send): {}", cts);
    println!("  DSR (Data Set Ready): {}", dsr);
    println!("  DCD (Data Carrier Detect): {}", dcd);
    println!("  RI (Ring Indicator): {}", ri);

    // 获取完整的调制解调器状态
    let modem_status = uart.get_modem_status();
    println!("完整调制解调器状态: {:?}", modem_status);
}

/// 演示中断配置
fn demo_interrupt_config(uart: &impl SerialRegister) {
    println!("\n=== 中断配置示例 ===");

    // 启用接收和发送中断
    let interrupt_mask = InterruptMask::RX_AVAILABLE | InterruptMask::TX_EMPTY;
    uart.enable_interrupts(interrupt_mask);
    println!("已启用接收和发送中断: {:?}", interrupt_mask);

    // 查询中断状态
    let int_status = uart.get_interrupt_status();
    println!("当前中断状态: {:?}", int_status);
}

/// 演示状态查询
fn demo_status_query(uart: &impl SerialRegister) {
    println!("\n=== 状态查询示例 ===");

    // 查询传输状态
    let tx_empty = uart.is_tx_empty();
    let rx_empty = uart.is_rx_empty();
    let can_read = uart.can_read();
    let can_write = uart.can_write();

    println!("传输状态:");
    println!("  发送寄存器为空: {}", tx_empty);
    println!("  接收FIFO为空: {}", rx_empty);
    println!("  可以读取数据: {}", can_read);
    println!("  可以写入数据: {}", can_write);

    // 查询线路状态
    let line_status = uart.line_status();
    println!("线路状态: {:?}", line_status);

    // 如果有错误，清除错误状态
    if line_status.intersects(
        LineStatus::OVERRUN_ERROR
            | LineStatus::PARITY_ERROR
            | LineStatus::FRAMING_ERROR
            | LineStatus::BREAK_INTERRUPT,
    ) {
        println!("检测到错误状态，正在清除...");
        uart.clear_error();
        println!("错误状态已清除");
    }
}

/// 演示电源管理
fn demo_power_management(uart: &impl SerialRegister) {
    println!("\n=== 电源管理示例 ===");

    // 查询当前电源模式
    let current_mode = uart.get_power_mode();
    println!("当前电源模式: {:?}", current_mode);

    // 切换到低功耗模式
    if let Err(e) = uart.set_power_mode(PowerMode::LowPower) {
        println!("设置低功耗模式失败: {:?}", e);
    } else {
        println!("已切换到低功耗模式");
    }

    // 恢复到正常模式
    if let Err(e) = uart.set_power_mode(PowerMode::Normal) {
        println!("恢复正常模式失败: {:?}", e);
    } else {
        println!("已恢复正常模式");
    }
}

/// 演示数据传输
fn demo_data_transfer(uart: &impl SerialRegister) {
    println!("\n=== 数据传输示例 ===");

    // 发送测试数据
    let test_data = b"Hello, Serial!";
    println!("发送数据: {:?}", test_data);

    for &byte in test_data {
        uart.write_byte(byte);
    }

    // 尝试接收数据
    println!("等待接收数据...");

    // 简单的非阻塞接收示例
    let mut received_data = alloc::vec::Vec::new();
    let timeout = 1000; // 模拟超时

    for _ in 0..timeout {
        if uart.can_read() {
            let byte = uart.read_byte();
            received_data.push(byte);
            if byte == b'!' {
                // 假设'!'是结束符
                break;
            }
        }
        // 在实际应用中，这里可能需要延时
    }

    if !received_data.is_empty() {
        println!("接收到数据: {:?}", received_data);
        if let Ok(data_str) = alloc::str::from_utf8(&received_data) {
            println!("接收字符串: {}", data_str);
        }
    } else {
        println!("未接收到数据");
    }
}

/// 演示配置验证
fn demo_config_validation() {
    println!("\n=== 配置验证示例 ===");

    // 测试有效配置
    let valid_configs = [
        (DataBits::Eight, StopBits::One, Parity::None),
        (DataBits::Seven, StopBits::One, Parity::Even),
        (DataBits::Five, StopBits::Two, Parity::None),
    ];

    println!("有效配置验证:");
    for (data_bits, stop_bits, parity) in &valid_configs {
        let is_valid = validate_serial_config(*data_bits, *stop_bits, *parity);
        println!(
            "  {:?} + {:?} + {:?} -> {}",
            data_bits, stop_bits, parity, is_valid
        );
    }

    // 测试无效配置
    let invalid_configs = [
        (DataBits::Eight, StopBits::Two, Parity::Even),
        (DataBits::Five, StopBits::One, Parity::Even),
    ];

    println!("无效配置验证:");
    for (data_bits, stop_bits, parity) in &invalid_configs {
        let is_valid = validate_serial_config(*data_bits, *stop_bits, *parity);
        println!(
            "  {:?} + {:?} + {:?} -> {}",
            data_bits, stop_bits, parity, is_valid
        );
    }

    // 测试波特率验证
    println!("\n波特率验证:");
    let test_baudrates = [9600, 115200, 1000, 0];
    for &baudrate in &test_baudrates {
        let is_valid = is_valid_baudrate(baudrate);
        println!("  {} bps -> {}", baudrate, is_valid);
    }

    // 测试FIFO触发级别推荐
    println!("\nFIFO触发级别推荐:");
    let fifo_sizes = [8, 16, 32, 64, 128, 256];
    for &size in &fifo_sizes {
        let level = recommended_fifo_trigger_level(size);
        println!("  FIFO大小 {} -> 触发级别 {}", size, level);
    }
}

/// 演示直接寄存器访问
fn demo_register_access(uart: &impl SerialRegister) {
    println!("\n=== 直接寄存器访问示例 ===");

    // 读取一些关键寄存器
    let line_ctrl = uart.read_reg(0x0C / 4); // 线路控制寄存器
    let line_status = uart.read_reg(0x14 / 4); // 线路状态寄存器
    let modem_ctrl = uart.read_reg(0x10 / 4); // 调制解调器控制寄存器
    let modem_status = uart.read_reg(0x18 / 4); // 调制解调器状态寄存器

    println!("关键寄存器状态:");
    println!("  线路控制 (0x0C): 0x{:08X}", line_ctrl);
    println!("  线路状态 (0x14): 0x{:08X}", line_status);
    println!("  调制解调器控制 (0x10): 0x{:08X}", modem_ctrl);
    println!("  调制解调器状态 (0x18): 0x{:08X}", modem_status);

    // 演示寄存器位操作
    println!("\n寄存器位操作示例:");

    // 备份原始值
    let original_int_enable = uart.read_reg(0x04 / 4);
    println!("原始中断使能寄存器: 0x{:08X}", original_int_enable);

    // 设置一些位
    uart.modify_reg(0x04 / 4, 0x00, 0x01); // 设置第0位
    let modified1 = uart.read_reg(0x04 / 4);
    println!("设置第0位后: 0x{:08X}", modified1);

    // 设置更多位
    uart.modify_reg(0x04 / 4, 0x00, 0x06); // 设置第1和第2位
    let modified2 = uart.read_reg(0x04 / 4);
    println!("设置第1和第2位后: 0x{:08X}", modified2);

    // 清除一些位
    uart.modify_reg(0x04 / 4, 0x02, 0x00); // 清除第1位
    let modified3 = uart.read_reg(0x04 / 4);
    println!("清除第1位后: 0x{:08X}", modified3);

    // 恢复原始值
    uart.write_reg(0x04 / 4, original_int_enable);
    let restored = uart.read_reg(0x04 / 4);
    println!("恢复原始值: 0x{:08X}", restored);
}

/// 主函数 - 演示所有功能
fn main() {
    println!("=== 串口设备使用示例 ===");
    println!("演示完善后的SerialRegister接口功能\n");

    // 创建一个示例UART实例（使用虚拟地址）
    // 在实际应用中，这里应该是真实的硬件地址
    let uart_base = 0x1000_0000; // 虚拟基地址
    let clock_freq = 18_432_000; // 18.432MHz时钟
    let uart = MemoryMappedUart::new(uart_base, clock_freq);

    // 注意：MemoryMappedUart实现了RegisterAccess和BaudrateSupport，
    // 因此自动获得了SerialRegister的所有功能

    // 运行所有演示
    demo_config_validation();

    demo_basic_config(&uart);
    demo_fifo_config(&uart);
    demo_flow_control(&uart);
    demo_interrupt_config(&uart);
    demo_status_query(&uart);
    demo_power_management(&uart);
    demo_register_access(&uart);

    // 数据传输演示（需要实际硬件）
    println!("\n=== 数据传输演示 ===");
    println!("注意: 此示例使用虚拟地址，实际数据传输需要真实硬件");
    demo_data_transfer(&uart);

    println!("\n=== 演示完成 ===");
    println!("完善后的SerialRegister接口提供了完整的UART控制功能，包括:");
    println!("✓ 基础数据传输（读写字节）");
    println!("✓ 完整的配置管理（波特率、数据位、停止位、奇偶校验）");
    println!("✓ FIFO控制和缓冲管理");
    println!("✓ 硬件流控制（RTS/CTS、DTR/DSR）");
    println!("✓ 中断管理和状态查询");
    println!("✓ 电源管理");
    println!("✓ 直接寄存器访问");
    println!("✓ 配置验证和辅助函数");
    println!("✓ 向后兼容原有接口");
}

/// 当panic发生时的处理函数
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    println!("Panic: {}", info);
    loop {}
}
