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

/// Game memory from a pointer, of an extent nobody here knows (a text up to its NUL, a
/// glyph table): every read answers, as the original's did.
pub struct Raw(*const u8);

impl Raw {
    /// # Safety
    /// Every byte the core will ask for must be readable.
    pub unsafe fn new(at: *const u8) -> Self {
        Raw(at)
    }
}

impl rc2k_core::bytes::Bytes for Raw {
    fn byte(&self, at: usize) -> Option<u8> {
        Some(unsafe { self.0.wrapping_add(at).read() })
    }
}

/// Writable game memory from a pointer, of an extent nobody here knows (a sample's data):
/// every read and write goes where the original's did.
pub struct RawMut(*mut u8);

impl RawMut {
    /// # Safety
    /// Every byte the core will read or write must be there and writable.
    pub unsafe fn new(at: *mut u8) -> Self {
        RawMut(at)
    }
}

impl rc2k_core::samples::SampleData for RawMut {
    fn get(&self, at: u32) -> Option<u8> {
        Some(unsafe { self.0.wrapping_add(at as usize).read() })
    }
    fn set(&mut self, at: u32, value: u8) -> bool {
        unsafe { self.0.wrapping_add(at as usize).write(value) };
        true
    }
}

/// A core result whose only failure is a read past the end, from `Raw` memory that never
/// has one.
pub fn read_through<T>(result: Result<T, rc2k_core::Invalid>) -> T {
    match result {
        Ok(value) => value,
        Err(e) => unreachable!("a read of raw game memory refused: {}", e.0),
    }
}
