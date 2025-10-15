#![no_std]
#![no_main]
#![feature(used_with_arg)]

extern crate alloc;
extern crate bare_test;

#[bare_test::tests]
mod tests {
    use core::sync::atomic::AtomicBool;

    use super::*;
    use alloc::sync::Arc;
    use bare_test::{
        GetIrqConfig,
        globals::{PlatformInfoKind, global_val},
        irq::{IrqHandleResult, IrqInfo, IrqParam},
        mem::iomap,
    };
    use log::{debug, info};
    use some_serial::pl011;
    use some_serial::{
        DataBits, DmaDirection, InterruptMask, LineStatus, ModemStatus, Parity, PowerMode,
        SerialRegister, StopBits, format_serial_config, validate_serial_config,
    };

    #[test]
    fn test_pl011() {
        info!("test uart");

        let (addr, size, irq_info) = get_uart(&["arm,pl011"]);

        let base = iomap(addr.into(), size);

        debug!("UART base address: {:p}, size: {:#x}", base, size);

        // 创建 PL011 实例，使用默认配置
        let uart = pl011::Pl011::new_no_clock(base.as_ptr() as _);

        info!(
            "UART created with auto-detected clock frequency: {} Hz",
            uart.clock_frequency()
        );

        info!("Testing new SerialRegister interface...");

        // 打开并初始化 UART
        if let Err(e) = uart.open() {
            info!("UART open failed: {:?}", e);
        } else {
            info!("UART opened successfully");
        }

        // 测试配置功能 - 使用新的配置接口
        info!("Testing configuration methods");

        let config = some_serial::Config::new()
            .baudrate(115200)
            .data_bits(DataBits::Eight)
            .stop_bits(StopBits::One)
            .parity(Parity::None);

        if let Err(e) = uart.set_config(&config) {
            info!("Set config failed: {:?}", e);
        } else {
            let actual_baud = uart.baudrate();
            let actual_data_bits = uart.data_bits();
            let actual_stop_bits = uart.stop_bits();
            let actual_parity = uart.parity();

            info!("Configuration applied successfully:");
            info!("  Baudrate: {}", actual_baud);
            info!("  Data bits: {:?}", actual_data_bits);
            info!("  Stop bits: {:?}", actual_stop_bits);
            info!("  Parity: {:?}", actual_parity);
        }

        // 测试FIFO功能（使用 PL011 特有的方法）
        info!("Testing FIFO operations");
        uart.enable_fifo(true);
        uart.set_fifo_trigger_level(8, 8);

        // 测试流控制（使用 PL011 特有的方法）
        info!("Testing flow control");
        uart.set_rts(true);
        uart.set_dtr(true);

        let cts = uart.get_cts();
        let dsr = uart.get_dsr();
        info!("CTS: {}, DSR: {}", cts, dsr);

        // 测试中断控制
        info!("Testing interrupt control");
        uart.enable_interrupts(InterruptMask::RX_AVAILABLE | InterruptMask::TX_EMPTY);
        let int_status = uart.get_interrupt_status();
        info!("Interrupt status: {:?}", int_status);

        // 测试状态查询
        info!("Testing status queries");
        let line_status = uart.line_status();
        let modem_status = uart.get_modem_status();
        info!("Line status: {:?}", line_status);
        info!("Modem status: {:?}", modem_status);

        // 测试 FIFO 级别查询
        info!("Testing FIFO levels");
        let tx_level = uart.get_tx_fifo_level();
        let rx_level = uart.get_rx_fifo_level();
        info!("TX FIFO level: {}, RX FIFO level: {}", tx_level, rx_level);

        // 测试寄存器访问
        info!("Testing register access");
        let base_addr = uart.get_base();
        info!("UART base address: 0x{:x}", base_addr);

        // 测试错误清除
        uart.clear_error();

        // 测试基本的读写操作（如果硬件可用）
        info!("Testing basic read/write operations");

        // 尝试写入一个测试字节
        if let Err(e) = uart.write_byte(b'T') {
            info!("Write byte failed: {:?}", e);
        } else {
            info!("Write byte successful");
        }

        // 尝试读取一个字节（可能有超时）
        match uart.read_byte() {
            Ok(byte) => info!("Read byte successful: 0x{:02x} ('{}')", byte, byte as char),
            Err(e) => info!("Read byte failed: {:?}", e),
        }

        info!("SerialRegister interface test completed");
    }

    #[test]
    fn test_serial_config_validation() {
        info!("Testing serial configuration validation");

        // 测试有效配置
        assert!(validate_serial_config(
            DataBits::Eight,
            StopBits::One,
            Parity::None
        ));
        assert!(validate_serial_config(
            DataBits::Seven,
            StopBits::One,
            Parity::Even
        ));
        assert!(validate_serial_config(
            DataBits::Five,
            StopBits::Two,
            Parity::None
        ));

        // 测试无效配置
        assert!(!validate_serial_config(
            DataBits::Eight,
            StopBits::Two,
            Parity::Even
        ));

        info!("Configuration validation test passed");
    }

    #[test]
    fn test_serial_config_formatting() {
        info!("Testing serial configuration formatting");

        let config_str = format_serial_config(115200, DataBits::Eight, StopBits::One, Parity::None);
        info!("Config string: {}", config_str);

        assert!(config_str.contains("115200 baud"));
        assert!(config_str.contains("8-data bits"));
        assert!(config_str.contains("1-stop bits"));
        assert!(config_str.contains("no-parity"));

        info!("Configuration formatting test passed");
    }

    #[test]
    fn test_bit_flags_operations() {
        info!("Testing bit flags operations");

        // 测试中断掩码
        let mask = InterruptMask::RX_AVAILABLE | InterruptMask::TX_EMPTY;
        assert!(mask.contains(InterruptMask::RX_AVAILABLE));
        assert!(mask.contains(InterruptMask::TX_EMPTY));
        assert!(!mask.contains(InterruptMask::MODEM_STATUS));

        // 测试线路状态
        let status = LineStatus::DATA_READY | LineStatus::TX_HOLDING_EMPTY;
        assert!(status.contains(LineStatus::DATA_READY));
        assert!(status.contains(LineStatus::TX_HOLDING_EMPTY));

        // 测试调制解调器状态
        let modem_status = ModemStatus::CTS | ModemStatus::DSR;
        assert!(modem_status.contains(ModemStatus::CTS));
        assert!(modem_status.contains(ModemStatus::DSR));

        info!("Bit flags operations test passed");
    }

    fn get_uart(cmp: &[&str]) -> (usize, usize, IrqInfo) {
        let PlatformInfoKind::DeviceTree(fdt) = &global_val().platform_info;
        let fdt = fdt.get();

        let chosen = fdt.chosen().unwrap().debugcon().unwrap();

        for node in fdt.find_compatible(cmp) {
            if node.name() == chosen.name() {
                continue;
            }

            let addr = node.reg().unwrap().next().unwrap();

            let size = addr.size.unwrap_or(0x1000);

            let irq_info = node.irq_info().unwrap();

            return (addr.address as usize, size, irq_info);
        }

        panic!("No matching UART node found");
    }
}
