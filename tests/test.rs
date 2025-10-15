#![no_std]
#![no_main]
#![feature(used_with_arg)]

extern crate alloc;
extern crate bare_test;

#[bare_test::tests]
mod tests {

    use core::ptr::NonNull;

    use super::*;
    use bare_test::{
        GetIrqConfig,
        globals::{PlatformInfoKind, global_val},
        irq::IrqInfo,
        mem::iomap,
    };
    use log::{debug, info};
    use some_serial::pl011;
    use some_serial::{DataBits, InterruptMask, LineStatus, Parity, SerialRegister, StopBits};

    #[derive(Debug)]
    struct SInfo {
        base: NonNull<u8>,
        clk: u32,
        irq: IrqInfo,
    }

    #[test]
    fn test_pl011() {
        info!("test uart");

        let info = get_uart(&["arm,pl011"]);

        debug!("UART base {info:#x?}");

        // 创建 PL011 实例，使用默认配置
        let mut uart = pl011::Pl011::new(info.base, info.clk);

        info!("Testing new SerialRegister interface...");

        uart.open();

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

        // 测试基本的读写操作（如果硬件可用）
        info!("Testing basic read/write operations");

        uart.write_buf(b"Hello, UART!\n");

        info!("SerialRegister interface test completed");
    }

    fn get_uart(cmp: &[&str]) -> SInfo {
        let PlatformInfoKind::DeviceTree(fdt) = &global_val().platform_info;
        let fdt = fdt.get();
        let node = fdt.find_compatible(cmp).next().unwrap();

        let addr = node.reg().unwrap().next().unwrap();

        let size = addr.size.unwrap_or(0x1000);

        let irq_info = node.irq_info().unwrap();

        let base = iomap((addr.address as usize).into(), size);

        let clk = node.clocks().next().unwrap().clock_frequency.unwrap();

        SInfo {
            base,
            clk,
            irq: irq_info,
        }
    }
}
