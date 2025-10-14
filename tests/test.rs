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

        let uart = pl011::new(base.as_ptr() as _);

        info!("Testing new SerialRegister interface...");

        // 测试基础功能（保持向后兼容）
        info!("Testing basic read/write operations");

        // 测试配置功能
        info!("Testing configuration methods");
        if let Err(e) = uart.set_baudrate(115200) {
            info!("Set baudrate failed: {:?}", e);
        }

        if let Err(e) = uart.set_data_bits(DataBits::Eight) {
            info!("Set data bits failed: {:?}", e);
        }

        if let Err(e) = uart.set_stop_bits(StopBits::One) {
            info!("Set stop bits failed: {:?}", e);
        }

        if let Err(e) = uart.set_parity(Parity::None) {
            info!("Set parity failed: {:?}", e);
        }

        // 测试FIFO功能
        info!("Testing FIFO operations");
        uart.enable_fifo(true);
        uart.set_fifo_trigger_level(8, 8);

        // 测试流控制
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
        let line_status = uart.get_line_status();
        let modem_status = uart.get_modem_status();
        info!("Line status: {:?}", line_status);
        info!("Modem status: {:?}", modem_status);

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
