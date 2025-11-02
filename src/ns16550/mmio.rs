//! NS16550 MMIO 版本实现
//!
//! 适用于嵌入式平台的内存映射 IO 版本

use super::{Kind, Ns16550};
use core::ptr::NonNull;

#[derive(Clone)]
pub struct Mmio {
    base: usize,
}

impl Kind for Mmio {
    fn read_reg(&self, reg: u8) -> u8 {
        unsafe {
            let addr = self.base + (reg as usize) * 4;
            core::ptr::read_volatile(addr as *const u8)
        }
    }

    fn write_reg(&self, reg: u8, val: u8) {
        unsafe {
            let addr = self.base + (reg as usize) * 4;
            core::ptr::write_volatile(addr as *mut u8, val);
        }
    }

    fn get_base(&self) -> usize {
        self.base
    }
}

impl Ns16550<Mmio> {
    pub fn new_mmio(base: NonNull<u8>, clock_freq: u32) -> Ns16550<Mmio> {
        Ns16550::new(
            Mmio {
                base: base.as_ptr() as usize,
            },
            clock_freq,
        )
    }
}
