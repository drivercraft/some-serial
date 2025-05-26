#![no_std]

extern crate alloc;

pub mod ns16550;
pub mod pl011;

#[repr(transparent)]
#[derive(Clone, Copy)]
struct Mmio(usize);

impl Mmio {
    fn read<T>(&self, offset: usize) -> T {
        unsafe { (self.0 as *const T).add(offset).read_volatile() }
    }

    fn write<T>(&self, offset: usize, val: T) {
        unsafe {
            (self.0 as *mut T).add(offset).write_volatile(val);
        }
    }
}
