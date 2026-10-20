//! The bridge between the converted code (`rc2k_core`, plain Rust with no addresses) and
//! the game's memory, for as long as the hooked DLL checks the conversion and unconverted
//! modules still keep their state at RAL.EXE's addresses (`notes/conversion.md`). An
//! adapter loads a core structure from the game's variables, calls the core, and stores
//! the result back. This module goes with the hooks.

use crate::framework::{game_addr, rva};
use std::marker::PhantomData;

/// Types any bit pattern is a value of, so reading the game's bytes as one is sound.
///
/// # Safety
/// Implement only for types with no invalid bit patterns and no padding (integers, floats,
/// arrays of them, `repr(C)` structures of them laid out without gaps).
pub unsafe trait Plain: Copy + 'static {}

macro_rules! plain {
    ($($t:ty),*) => { $(unsafe impl Plain for $t {})* };
}
plain!(u8, i8, u16, i16, u32, i32, u64, i64, f32, f64);
unsafe impl<T: Plain, const N: usize> Plain for [T; N] {}

/// One of the game's variables of type `T`, at its IDA address.
pub struct Global<T: Plain> {
    rva: usize,
    _type: PhantomData<T>,
}

impl<T: Plain> Global<T> {
    /// The variable at `address` (as IDA shows it).
    pub const fn at(address: usize) -> Self {
        Self {
            rva: rva(address),
            _type: PhantomData,
        }
    }

    /// Its address in the image now.
    pub fn addr(&self) -> usize {
        game_addr(self.rva)
    }

    /// Its value now. Read on every use: the game's code changes it too.
    pub fn get(&self) -> T {
        let p = self.addr() as *const T;
        // SAFETY: the image is there while hooks or the standalone run (the only callers);
        // `T: Plain` makes any bytes a value; volatile as the game's code writes it.
        unsafe {
            if p.is_aligned() {
                p.read_volatile()
            } else {
                p.read_unaligned()
            }
        }
    }

    /// Sets it.
    pub fn set(&self, value: T) {
        let p = self.addr() as *mut T;
        // SAFETY: as `get`; the image is writable where the game keeps its variables.
        unsafe {
            if p.is_aligned() {
                p.write_volatile(value)
            } else {
                p.write_unaligned(value)
            }
        }
    }
}
