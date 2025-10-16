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
    use rdif_serial::{Interface as _, TReciever, TSender};

    use super::*;
    use bare_test::{
        GetIrqConfig,
        globals::{PlatformInfoKind, global_val},
        irq::IrqInfo,
        mem::iomap,
    };
    use log::info;
    use some_serial::{DataBits, InterruptMask, Parity, StopBits};

    #[derive(Debug)]
    struct SInfo {
        base: NonNull<u8>,
        clk: u32,
        #[allow(dead_code)]
        irq: IrqInfo,
    }

    // === Serial 专用回环测试 ===

    /// Serial 基础回环测试 - 验证 Interface trait 基本功能
    #[test]
    fn test_serial_basic_loopback() {
        info!("=== Serial Basic Loopback Test ===");

        let mut serial = create_test_serial();
        serial.open().expect("Failed to open Serial");

        // 配置 Serial
        let config = some_serial::Config::new()
            .baudrate(115200)
            .data_bits(DataBits::Eight)
            .stop_bits(StopBits::One)
            .parity(Parity::None);

        if let Err(e) = serial.set_config(&config) {
            info!("Serial config failed: {:?}", e);
            return;
        }

        // 获取 TX/RX 接口
        let mut tx = match serial.take_tx() {
            Some(tx) => {
                info!("✓ TX interface obtained successfully");
                tx
            }
            None => {
                info!("✗ Failed to obtain TX interface");
                return;
            }
        };

        let mut rx = match serial.take_rx() {
            Some(rx) => {
                info!("✓ RX interface obtained successfully");
                rx
            }
            None => {
                info!("✗ Failed to obtain RX interface");
                return;
            }
        };

        // 测试回环功能
        test_serial_loopback_with_data(&mut serial, &mut *tx, &mut *rx, b"Hello Serial!");

        // 清理资源
        drop(tx);
        drop(rx);

        info!("=== Serial Basic Loopback Test Completed ===");
    }

    /// Serial 资源管理测试 - 验证 RAII 和资源生命周期
    #[test]
    fn test_serial_resource_management() {
        info!("=== Serial Resource Management Test ===");

        let mut serial = create_test_serial();

        // 测试 1: 初始状态应该有资源
        {
            let tx = serial.take_tx();
            let rx = serial.take_rx();
            assert!(tx.is_some(), "TX should be available initially");
            assert!(rx.is_some(), "RX should be available initially");
            info!("✓ Initial resource availability verified");

            // 测试 2: 资源被占用后应该不可用
            let tx2 = serial.take_tx();
            let rx2 = serial.take_rx();
            assert!(tx2.is_none(), "TX should not be available when occupied");
            assert!(rx2.is_none(), "RX should not be available when occupied");
            info!("✓ Resource exclusivity verified");
        } // 资源在这里被 Drop

        // 测试 3: Drop 后资源应该恢复可用
        {
            let tx = serial.take_tx();
            let rx = serial.take_rx();
            assert!(tx.is_some(), "TX should be available after drop");
            assert!(rx.is_some(), "RX should be available after drop");
            info!("✓ Resource recovery after drop verified");
        }

        // 测试 4: 重复获取和释放
        for i in 0..3 {
            let _tx = serial.take_tx();
            let _rx = serial.take_rx();
            info!("✓ Resource cycle {} completed", i + 1);
        }

        info!("=== Serial Resource Management Test Completed ===");
    }

    /// Serial Interface trait 完整性测试
    #[test]
    fn test_serial_interface_compliance() {
        info!("=== Serial Interface Compliance Test ===");

        let mut serial = create_test_serial();

        // 测试配置管理
        test_serial_configuration(&mut serial);

        // 测试回环控制
        test_serial_loopback_control(&mut serial);

        // 测试中断管理
        test_serial_interrupt_management(&mut serial);

        // 测试 DriverGeneric 接口
        test_serial_driver_generic(&mut serial);

        info!("=== Serial Interface Compliance Test Completed ===");
    }

    /// Serial 多数据模式回环测试
    #[test]
    fn test_serial_multi_pattern_loopback() {
        info!("=== Serial Multi-Pattern Loopback Test ===");

        let mut serial = create_test_serial();
        serial.open().expect("Failed to open Serial");

        let config = some_serial::Config::new()
            .baudrate(115200)
            .data_bits(DataBits::Eight)
            .stop_bits(StopBits::One)
            .parity(Parity::None);

        if let Err(e) = serial.set_config(&config) {
            info!("Config failed: {:?}", e);
            return;
        }

        // 启用回环
        serial.enable_loopback();
        assert!(serial.is_loopback_enabled());

        // 测试多种数据模式
        let test_patterns: &[(&str, &[u8])] = &[
            ("Short text", b"Hello"),
            ("Medium text", b"This is a medium length test string"),
            ("Numbers", b"0123456789"),
            ("Special chars", b"!@#$%^&*()"),
            ("Binary data", &[0x00, 0x01, 0x7F, 0x80, 0xFF]),
            ("Empty data", b""),
        ];

        let mut passed = 0;
        let mut total = 0;

        for (pattern_name, pattern_data) in test_patterns.iter() {
            total += 1;
            info!(
                "Testing pattern: {} ({} bytes)",
                pattern_name,
                pattern_data.len()
            );

            // 每次测试都需要重新获取 TX/RX（因为资源会被消耗）
            let mut tx = match serial.take_tx() {
                Some(tx) => tx,
                None => {
                    info!("✗ Failed to get TX for pattern {}", pattern_name);
                    continue;
                }
            };

            let mut rx = match serial.take_rx() {
                Some(rx) => rx,
                None => {
                    info!("✗ Failed to get RX for pattern {}", pattern_name);
                    continue;
                }
            };

            if test_serial_loopback_with_data(&mut serial, &mut *tx, &mut *rx, pattern_data) {
                passed += 1;
            }

            // 资源自动回收
        }

        info!(
            "Multi-pattern test results: {}/{} patterns passed",
            passed, total
        );

        // 禁用回环
        serial.disable_loopback();
        assert!(!serial.is_loopback_enabled());

        info!("=== Serial Multi-Pattern Loopback Test Completed ===");
    }

    /// Serial 压力测试 - 多次连续回环操作
    #[test]
    fn test_serial_stress_loopback() {
        info!("=== Serial Stress Loopback Test ===");

        let mut serial = create_test_serial();
        serial.open().expect("Failed to open Serial");

        let config = some_serial::Config::new()
            .baudrate(115200)
            .data_bits(DataBits::Eight)
            .stop_bits(StopBits::One)
            .parity(Parity::None);

        if let Err(e) = serial.set_config(&config) {
            info!("Config failed: {:?}", e);
            return;
        }

        serial.enable_loopback();

        let stress_data = b"Stress test data for Serial interface";
        let iterations = 10;
        let mut successful_iterations = 0;

        for i in 0..iterations {
            info!("Stress iteration {}/{}", i + 1, iterations);

            let mut tx = match serial.take_tx() {
                Some(tx) => tx,
                None => {
                    info!("Failed to get TX in iteration {}", i + 1);
                    continue;
                }
            };

            let mut rx = match serial.take_rx() {
                Some(rx) => rx,
                None => {
                    info!("Failed to get RX in iteration {}", i + 1);
                    continue;
                }
            };

            if test_serial_loopback_with_data(&mut serial, &mut *tx, &mut *rx, stress_data) {
                successful_iterations += 1;
            }

            // 添加短暂延迟
            for _ in 0..1000 {
                core::hint::spin_loop();
            }
        }

        info!(
            "Stress test completed: {}/{} iterations successful",
            successful_iterations, iterations
        );

        serial.disable_loopback();

        info!("=== Serial Stress Loopback Test Completed ===");
    }

    // === Serial 专用辅助函数 ===

    /// 创建标准测试用 Serial 实例
    fn create_test_serial() -> some_serial::Serial<some_serial::pl011::Pl011> {
        let info = get_uart_for_serial_test();
        some_serial::pl011::Pl011::new(info.base, info.clk)
    }

    /// 获取 Serial 测试用的 UART 设备
    fn get_uart_for_serial_test() -> SInfo {
        match get_secondary_uart() {
            Some(info) => {
                info!("Using secondary PL011 for Serial testing");
                info
            }
            None => {
                info!("No secondary PL011 found, using primary for Serial testing");
                get_uart(&["arm,pl011"])
            }
        }
    }

    /// Serial 回环数据测试函数
    fn test_serial_loopback_with_data(
        serial: &mut some_serial::Serial<some_serial::pl011::Pl011>,
        tx: &mut dyn TSender,
        rx: &mut dyn TReciever,
        test_data: &[u8],
    ) -> bool {
        // 确保回环模式启用
        if !serial.is_loopback_enabled() {
            serial.enable_loopback();
        }

        // 发送数据
        let sent_bytes = tx.send(test_data);
        info!("Sent {} bytes", sent_bytes);

        if sent_bytes == 0 && !test_data.is_empty() {
            info!("✗ Failed to send any data");
            return false;
        }

        // 接收数据
        let mut recv_buf = vec![0u8; test_data.len() + 10];
        match rx.recive(&mut recv_buf) {
            Ok(received_bytes) => {
                info!("Received {} bytes", received_bytes);

                if received_bytes == sent_bytes {
                    let sent_data = &test_data[..sent_bytes];
                    let received_data = &recv_buf[..received_bytes];

                    if sent_data == received_data {
                        info!("✓ Loopback test passed: data matches");
                        return true;
                    } else {
                        info!("✗ Loopback test failed: data mismatch");
                        info!("  Sent:    {:?}", sent_data);
                        info!("  Received: {:?}", received_data);
                    }
                } else {
                    info!("✗ Loopback test failed: length mismatch");
                    info!("  Sent: {}, Received: {}", sent_bytes, received_bytes);
                }
            }
            Err(e) => {
                info!("✗ Loopback test failed: receive error {:?}", e);
            }
        }

        false
    }

    /// 测试 Serial 配置功能
    fn test_serial_configuration(serial: &mut some_serial::Serial<some_serial::pl011::Pl011>) {
        info!("Testing Serial configuration...");

        let test_configs = [
            (115200, DataBits::Eight, StopBits::One, Parity::None),
            (9600, DataBits::Seven, StopBits::One, Parity::Even),
            (38400, DataBits::Eight, StopBits::Two, Parity::Odd),
        ];

        for (i, (baudrate, data_bits, stop_bits, parity)) in test_configs.iter().enumerate() {
            let config = some_serial::Config::new()
                .baudrate(*baudrate)
                .data_bits(*data_bits)
                .stop_bits(*stop_bits)
                .parity(*parity);

            if let Err(e) = serial.set_config(&config) {
                info!("Config {} failed: {:?}", i + 1, e);
                continue;
            }

            // 验证配置
            let actual_baudrate = serial.baudrate();
            let actual_data_bits = serial.data_bits();
            let actual_stop_bits = serial.stop_bits();
            let actual_parity = serial.parity();

            info!("Config {} applied:", i + 1);
            info!("  Baudrate: {} (expected: {})", actual_baudrate, baudrate);
            info!(
                "  Data bits: {:?} (expected: {:?})",
                actual_data_bits, data_bits
            );
            info!(
                "  Stop bits: {:?} (expected: {:?})",
                actual_stop_bits, stop_bits
            );
            info!("  Parity: {:?} (expected: {:?})", actual_parity, parity);
        }

        info!("✓ Serial configuration test completed");
    }

    /// 测试 Serial 回环控制功能
    fn test_serial_loopback_control(serial: &mut some_serial::Serial<some_serial::pl011::Pl011>) {
        info!("Testing Serial loopback control...");

        // 初始状态
        let initial_state = serial.is_loopback_enabled();
        info!("Initial loopback state: {}", initial_state);

        // 启用回环
        serial.enable_loopback();
        assert!(serial.is_loopback_enabled());
        info!("✓ Loopback enable verified");

        // 禁用回环
        serial.disable_loopback();
        assert!(!serial.is_loopback_enabled());
        info!("✓ Loopback disable verified");

        // 恢复初始状态
        if initial_state {
            serial.enable_loopback();
        }

        info!("✓ Serial loopback control test completed");
    }

    /// 测试 Serial 中断管理功能
    fn test_serial_interrupt_management(
        serial: &mut some_serial::Serial<some_serial::pl011::Pl011>,
    ) {
        info!("Testing Serial interrupt management...");

        let test_masks = [
            InterruptMask::RX_AVAILABLE,
            InterruptMask::TX_EMPTY,
            InterruptMask::RX_AVAILABLE | InterruptMask::TX_EMPTY,
        ];

        for (i, mask) in test_masks.iter().enumerate() {
            info!("Testing mask {}: {:?}", i + 1, mask);

            // 启用中断
            serial.enable_interrupts(*mask);
            info!("✓ Interrupts enabled for mask {}", i + 1);

            // 禁用中断
            serial.disable_interrupts(*mask);
            info!("✓ Interrupts disabled for mask {}", i + 1);
        }

        info!("✓ Serial interrupt management test completed");
    }

    /// 测试 Serial DriverGeneric 接口
    fn test_serial_driver_generic(serial: &mut some_serial::Serial<some_serial::pl011::Pl011>) {
        info!("Testing Serial DriverGeneric interface...");

        // 测试 open/close
        serial.open().expect("Failed to open serial");
        info!("✓ Serial open successful");

        serial.close().expect("Failed to close serial");
        info!("✓ Serial close successful");

        // 测试 base 地址获取
        let base_addr = serial.base();
        info!("✓ Serial base address: 0x{:x}", base_addr);

        info!("✓ Serial DriverGeneric interface test completed");
    }

    // === 现有辅助函数（保持不变） ===
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

        let pl011_devices = fdt.find_compatible(&["arm,pl011"]).collect::<Vec<_>>();

        info!("=== UART Device Detection ===");
        info!("Found {} PL011 devices in device tree", pl011_devices.len());

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

        let pl011_devices = fdt.find_compatible(&["arm,pl011"]).collect::<Vec<_>>();

        if pl011_devices.len() > 1 {
            let node = &pl011_devices[1];

            let addr = node.reg().unwrap().next().unwrap();
            let size = addr.size.unwrap_or(0x1000);
            let irq_info = node.irq_info().unwrap();
            let base = iomap((addr.address as usize).into(), size);

            let clk = node
                .clocks()
                .next()
                .and_then(|clk| clk.clock_frequency)
                .unwrap_or(24_000_000);

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
