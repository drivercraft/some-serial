#![no_std]
#![no_main]
#![feature(used_with_arg)]

extern crate alloc;
extern crate bare_test;

#[bare_test::tests]
mod tests {

    use alloc::vec;
    use alloc::vec::Vec;
    use core::ptr::NonNull;
    use rdif_serial::Interface as _;

    use super::*;
    use bare_test::{
        GetIrqConfig,
        globals::{PlatformInfoKind, global_val},
        irq::IrqInfo,
        mem::iomap,
    };
    use log::{debug, info};
    use some_serial::pl011;
    use some_serial::{DataBits, Parity, StopBits};

    #[derive(Debug)]
    struct SInfo {
        base: NonNull<u8>,
        clk: u32,
        #[allow(dead_code)]
        irq: IrqInfo,
    }

    #[test]
    fn test_pl011() {
        info!("test uart");

        // 首先进行设备检测
        detect_uart_devices();

        // 尝试获取第二个 PL011 设备用于测试
        let info = match get_secondary_uart() {
            Some(info) => {
                info!("Using secondary PL011 for loopback testing");
                info
            }
            None => {
                info!("No secondary PL011 found, falling back to primary PL011");
                get_uart(&["arm,pl011"])
            }
        };

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

        let mut tx = uart.take_tx().expect("Failed to take TX");
        let mut rx = uart.take_rx().expect("Failed to take RX");
        tx.send(b"Hello, UART!\n");

        // 测试回环功能
        info!("Testing loopback functionality");

        // 确保开始时回环模式是禁用的
        uart.disable_loopback();
        if uart.is_loopback_enabled() {
            info!("Warning: Loopback was already enabled");
        }

        // 启用回环模式
        info!("Enabling loopback mode");
        uart.enable_loopback();

        if !uart.is_loopback_enabled() {
            info!("Error: Failed to enable loopback mode");
            return;
        }

        info!("Loopback mode enabled successfully");

        // 等待一小段时间让回环模式稳定
        for _ in 0..1000 {
            core::hint::spin_loop();
        }

        // 测试字符串的回环传输
        let test_strings: &[&[u8]] = &[
            b"Hello, Loopback!",
            b"Test123",
            b"Special chars: !@#$%^&*()",
            b"Multi\nline\ntest",
            b"The quick brown fox jumps over the lazy dog",
        ];

        for (i, test_str) in test_strings.iter().enumerate() {
            info!(
                "Testing string {}: {:?}",
                i + 1,
                core::str::from_utf8(test_str).unwrap_or("[invalid utf8]")
            );

            // 发送测试字符串
            let bytes_written = (test_str);
            info!("Sent {} bytes", bytes_written);

             

            // 读取回环的数据
            let mut read_buf = [0u8; 64];
            let mut total_read = 0;
            let mut attempts = 0;
            let max_attempts = 10000;

            while total_read < bytes_written && attempts < max_attempts {
                match rx.recive(&mut read_buf[total_read..]) {
                    Ok(bytes_read) => {
                        if bytes_read == 0 {
                            attempts += 1;
                            for _ in 0..100 {
                                core::hint::spin_loop();
                            }
                        } else {
                            total_read += bytes_read;
                            attempts = 0;
                        }
                    }
                    Err(e) => {
                        info!("Read error: {:?}", e);
                        break;
                    }
                }
            }

            info!("Read {} bytes", total_read);

            // 比较发送和接收的数据
            if total_read == bytes_written {
                let sent_data = &test_str[..bytes_written];
                let received_data = &read_buf[..total_read];

                if sent_data == received_data {
                    info!("✓ Loopback test {} passed: data matches", i + 1);
                } else {
                    info!("✗ Loopback test {} failed: data mismatch", i + 1);
                    info!("  Sent:    {:?}", sent_data);
                    info!("  Received: {:?}", received_data);
                }
            } else {
                info!(
                    "✗ Loopback test {} failed: length mismatch (sent: {}, received: {})",
                    i + 1,
                    bytes_written,
                    total_read
                );
            }

            // 添加测试间隔
            for _ in 0..1000 {
                core::hint::spin_loop();
            }
        }

        // 禁用回环模式
        info!("Disabling loopback mode");
        uart.disable_loopback();

        if uart.is_loopback_enabled() {
            info!("Warning: Loopback mode is still enabled");
        } else {
            info!("Loopback mode disabled successfully");
        }

        info!("SerialRegister interface test completed");
    }

    #[test]
    fn test_pl011_loopback_stress() {
        info!("Testing PL011 loopback stress test");

        // 尝试获取第二个 PL011 设备用于测试
        let info = match get_secondary_uart() {
            Some(info) => {
                info!("Using secondary PL011 for stress testing");
                info
            }
            None => {
                info!("No secondary PL011 found, falling back to primary PL011");
                get_uart(&["arm,pl011"])
            }
        };
        let mut uart = pl011::Pl011::new_raw(info.base, info.clk);

        uart.open();

        // 配置 UART
        let config = some_serial::Config::new()
            .baudrate(115200)
            .data_bits(DataBits::Eight)
            .stop_bits(StopBits::One)
            .parity(Parity::None);

        if let Err(e) = uart.set_config(&config) {
            info!("Failed to configure UART: {:?}", e);
            return;
        }

        // 启用回环模式
        uart.enable_loopback();
        if !uart.is_loopback_enabled() {
            info!("Failed to enable loopback mode");
            return;
        }

        info!("Starting stress test with multiple iterations");

        // 压力测试：多次不同长度的数据传输
        let test_data: &[(&str, &[u8])] = &[
            ("Single byte", b"A"),
            ("Short string", b"Hello"),
            ("Medium string", b"This is a medium length test string"),
            ("Long string", b"This is a very long test string that contains multiple words and should test the buffering capabilities of the UART interface thoroughly"),
            ("Numbers", b"012345678901234567890123456789"),
            ("Mixed content", b"Mix3d c0nt3nt w1th num8ers & symb0ls!@#"),
            ("Whitespace", b"   \t\n\r   "),
            ("Repeated chars", b"AAAAAAAAAAAAAAAAAAAAAAAAAAAA"),
        ];

        let mut total_tests = 0;
        let mut passed_tests = 0;

        for (test_name, data) in test_data.iter() {
            total_tests += 1;
            info!("Testing {}: {} bytes", test_name, data.len());

            // 运行多次以确保稳定性
            for iteration in 0..5 {
                // 清空接收缓冲区
                while uart.line_status().can_read() {
                    let _ = uart.read_byte();
                }

                // 发送数据
                let bytes_written = uart.write_buf(*data);

                // 等待数据可读
                let mut wait_count = 0;
                while !uart.line_status().can_read() && wait_count < 50000 {
                    core::hint::spin_loop();
                    wait_count += 1;
                }

                // 读取数据
                let mut read_buf = vec![0u8; data.len() + 10]; // 多分配一些空间
                let mut total_read = 0;
                let mut attempts = 0;
                let max_attempts = 5000;

                while total_read < bytes_written && attempts < max_attempts {
                    match uart.read_buf(&mut read_buf[total_read..]) {
                        Ok(0) => {
                            attempts += 1;
                            for _ in 0..100 {
                                core::hint::spin_loop();
                            }
                        }
                        Ok(bytes_read) => {
                            total_read += bytes_read;
                            attempts = 0;
                        }
                        Err(e) => {
                            info!("Read error during iteration {}: {:?}", iteration, e);
                            break;
                        }
                    }
                }

                // 验证数据
                if total_read == bytes_written {
                    let sent = &data[..bytes_written];
                    let received = &read_buf[..total_read];
                    if sent == received {
                        if iteration == 0 {
                            passed_tests += 1;
                        }
                    } else {
                        info!(
                            "✗ {} iteration {} failed: data mismatch",
                            test_name, iteration
                        );
                        info!("  Sent:    {:?}", sent);
                        info!("  Received: {:?}", received);
                    }
                } else {
                    info!(
                        "✗ {} iteration {} failed: length mismatch (sent: {}, received: {})",
                        test_name, iteration, bytes_written, total_read
                    );
                }

                // 测试间隔
                for _ in 0..1000 {
                    core::hint::spin_loop();
                }
            }
        }

        info!(
            "Stress test completed: {}/{} test cases passed",
            passed_tests, total_tests
        );

        // 禁用回环模式
        uart.disable_loopback();
        info!("Loopback stress test completed");
    }

    #[test]
    fn test_pl011_loopback_edge_cases() {
        info!("Testing PL011 loopback edge cases");

        // 尝试获取第二个 PL011 设备用于测试
        let info = match get_secondary_uart() {
            Some(info) => {
                info!("Using secondary PL011 for edge case testing");
                info
            }
            None => {
                info!("No secondary PL011 found, falling back to primary PL011");
                get_uart(&["arm,pl011"])
            }
        };
        let mut uart = pl011::Pl011::new_raw(info.base, info.clk);

        uart.open();
        uart.enable_loopback();

        // 测试空数据
        info!("Testing empty data");
        let empty_data: &[u8] = b"";
        let bytes_written = uart.write_buf(empty_data);
        info!("Empty data: wrote {} bytes", bytes_written);

        // 测试单个字节
        info!("Testing single byte patterns");
        let single_bytes = [0x00, 0x7F, 0xFF, 0x55, 0xAA];
        for &byte in &single_bytes {
            let test_data = [byte];

            // 清空缓冲区
            while uart.line_status().can_read() {
                let _ = uart.read_byte();
            }

            let bytes_written = uart.write_buf(&test_data);

            // 等待并读取
            let mut read_byte = 0;
            let mut attempts = 0;
            while attempts < 5000 {
                if uart.line_status().can_read() {
                    match uart.read_byte() {
                        Ok(b) => {
                            read_byte = b;
                            break;
                        }
                        Err(e) => {
                            info!("Error reading byte 0x{:02X}: {:?}", byte, e);
                            break;
                        }
                    }
                }
                attempts += 1;
                for _ in 0..50 {
                    core::hint::spin_loop();
                }
            }

            if bytes_written == 1 && read_byte == byte {
                info!("✓ Single byte 0x{:02X} test passed", byte);
            } else {
                info!(
                    "✗ Single byte 0x{:02X} test failed (sent: 1, received: {}, value: 0x{:02X})",
                    byte, bytes_written, read_byte
                );
            }
        }

        // 测试回环模式切换
        info!("Testing loopback mode toggle");

        // 发送数据但禁用回环
        uart.disable_loopback();
        let test_data = b"Toggle Test";
        let _ = uart.write_buf(test_data);

        // 等待一段时间，确保没有数据被回环
        for _ in 0..10000 {
            core::hint::spin_loop();
        }

        let mut has_data = false;
        for _ in 0..100 {
            if uart.line_status().can_read() {
                has_data = true;
                break;
            }
            for _ in 0..100 {
                core::hint::spin_loop();
            }
        }

        if !has_data {
            info!("✓ Loopback disabled correctly - no data received");
        } else {
            info!("? Data detected when loopback disabled - this may be expected on some hardware");
        }

        // 重新启用回环并发送数据
        uart.enable_loopback();

        // 清空缓冲区
        while uart.line_status().can_read() {
            let _ = uart.read_byte();
        }

        let _ = uart.write_buf(test_data);

        // 等待数据
        let mut read_buf = [0u8; 64];
        let mut total_read = 0;
        let mut attempts = 0;

        while total_read < test_data.len() && attempts < 5000 {
            match uart.read_buf(&mut read_buf[total_read..]) {
                Ok(0) => {
                    attempts += 1;
                    for _ in 0..100 {
                        core::hint::spin_loop();
                    }
                }
                Ok(bytes_read) => {
                    total_read += bytes_read;
                    attempts = 0;
                }
                Err(_) => break,
            }
        }

        if total_read == test_data.len() {
            info!(
                "✓ Loopback re-enabled correctly - received {} bytes",
                total_read
            );
        } else {
            info!(
                "✗ Loopback re-enable test failed - expected {}, received {}",
                test_data.len(),
                total_read
            );
        }

        uart.disable_loopback();
        info!("Edge cases test completed");
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

    fn detect_uart_devices() {
        let PlatformInfoKind::DeviceTree(fdt) = &global_val().platform_info;
        let fdt = fdt.get();

        // 查找所有 PL011 兼容的设备
        let pl011_devices = fdt.find_compatible(&["arm,pl011"]).collect::<Vec<_>>();

        info!("=== UART Device Detection ===");
        info!("Found {} PL011 devices in device tree", pl011_devices.len());

        // 打印所有 PL011 设备的信息
        for (i, node) in pl011_devices.iter().enumerate() {
            if let Some(addr) = node.reg().and_then(|mut reg| reg.next()) {
                info!("  PL011 {}: address 0x{:x}", i, addr.address);
            } else {
                info!("  PL011 {}: no address found", i);
            }
        }
        info!("=== End Device Detection ===");
    }

    fn get_secondary_uart() -> Option<SInfo> {
        let PlatformInfoKind::DeviceTree(fdt) = &global_val().platform_info;
        let fdt = fdt.get();

        // 查找所有 PL011 兼容的设备
        let pl011_devices = fdt.find_compatible(&["arm,pl011"]).collect::<Vec<_>>();

        // 如果有多个 PL011 设备，返回第二个（非 stdout 的）
        if pl011_devices.len() > 1 {
            let node = &pl011_devices[1]; // 使用第二个设备

            let addr = node.reg().unwrap().next().unwrap();
            let size = addr.size.unwrap_or(0x1000);

            let irq_info = node.irq_info().unwrap();
            let base = iomap((addr.address as usize).into(), size);

            // 如果没有时钟信息，使用默认值
            let clk = node
                .clocks()
                .next()
                .and_then(|clk| clk.clock_frequency)
                .unwrap_or(24_000_000); // 默认 24MHz

            info!(
                "Using secondary PL011 at address 0x{:x}, clock: {} Hz",
                addr.address, clk
            );

            Some(SInfo {
                base,
                clk,
                irq: irq_info,
            })
        } else {
            info!(
                "No secondary PL011 device found, only {} PL011 devices available",
                pl011_devices.len()
            );
            None
        }
    }
}
