//! PL011 UART 回环功能示例
//!
//! 这个示例展示了如何使用 PL011 UART 的回环功能进行自我测试。

#![no_std]
#![no_main]

extern crate alloc;
extern crate bare_test;

#[bare_test::tests]
mod tests {
    use bare_test::tests;
    use core::ptr::NonNull;
    use log::info;
    use some_serial::pl011;
    use some_serial::{DataBits, Parity, SerialRegister, StopBits};

    #[test]
    fn example_loopback_demo() {
        info!("PL011 UART 回环功能演示");

        // 注意：在实际环境中，你需要从设备树或其他配置源获取这些值
        // 这里使用示例值
        let base_addr = 0x9000000 as *mut u8; // 示例基地址
        let clock_freq = 24_000_000; // 24MHz

        // 在实际环境中，请确保地址是有效的
        let base = unsafe {
            // 这里需要一个有效的内存映射地址
            // 实际使用时应该从平台特定的配置获取
            match NonNull::new(base_addr) {
                Some(ptr) => ptr,
                None => {
                    info!("无效的基地址");
                    return;
                }
            }
        };

        // 创建 PL011 实例
        let mut uart = pl011::Pl011::new_raw(base, clock_freq);

        // 配置 UART
        let config = some_serial::Config::new()
            .baudrate(115200)
            .data_bits(DataBits::Eight)
            .stop_bits(StopBits::One)
            .parity(Parity::None);

        if let Err(e) = uart.set_config(&config) {
            info!("UART 配置失败: {:?}", e);
            return;
        }

        info!("UART 配置完成");

        // 启用回环模式
        uart.enable_loopback();
        info!("回环模式已启用");

        // 发送测试数据
        let test_message = b"Hello, PL011 Loopback!";
        info!(
            "发送测试消息: {:?}",
            core::str::from_utf8(test_message).unwrap()
        );

        let bytes_written = uart.write_buf(test_message);
        info!("已发送 {} 字节", bytes_written);

        // 读取回环的数据
        let mut read_buffer = [0u8; 64];
        let mut total_read = 0;

        // 等待数据可读并读取
        for _ in 0..10000 {
            if uart.line_status().can_read() {
                match uart.read_buf(&mut read_buffer[total_read..]) {
                    Ok(bytes_read) => {
                        if bytes_read > 0 {
                            total_read += bytes_read;
                            if total_read >= bytes_written {
                                break;
                            }
                        }
                    }
                    Err(e) => {
                        info!("读取错误: {:?}", e);
                        break;
                    }
                }
            }
        }

        info!("已读取 {} 字节", total_read);

        // 比较发送和接收的数据
        if total_read == bytes_written {
            let received_data = &read_buffer[..total_read];
            if test_message == received_data {
                info!("✓ 回环测试成功！数据完全匹配");
                info!("  发送: {:?}", core::str::from_utf8(test_message).unwrap());
                info!("  接收: {:?}", core::str::from_utf8(received_data).unwrap());
            } else {
                info!("✗ 回环测试失败：数据不匹配");
                info!("  发送: {:?}", test_message);
                info!("  接收: {:?}", received_data);
            }
        } else {
            info!("✗ 回环测试失败：长度不匹配");
            info!("  发送长度: {}", bytes_written);
            info!("  接收长度: {}", total_read);
        }

        // 禁用回环模式
        uart.disable_loopback();
        info!("回环模式已禁用");

        info!("回环功能演示完成");
    }

    #[test]
    fn example_loopback_configuration_methods() {
        info!("PL011 回环配置方法演示");

        let base_addr = 0x9000000 as *mut u8;
        let clock_freq = 24_000_000;

        let base = unsafe {
            match NonNull::new(base_addr) {
                Some(ptr) => ptr,
                None => return,
            }
        };

        let mut uart = pl011::Pl011::new_raw(base, clock_freq);
        uart.open();

        // 演示回环控制方法
        info!("初始回环状态: {}", uart.is_loopback_enabled());

        // 启用回环
        uart.enable_loopback();
        info!("启用后回环状态: {}", uart.is_loopback_enabled());

        // 禁用回环
        uart.disable_loopback();
        info!("禁用后回环状态: {}", uart.is_loopback_enabled());

        info!("回环配置方法演示完成");
    }
}
