//! The bridge between the converted code (`rc2k_core`, plain Rust with no addresses) and
//! the game's memory, for as long as the hooked DLL checks the conversion and unconverted
//! modules still keep their state at RAL.EXE's addresses (`notes/conversion.md`). An
//! adapter loads a core structure from the game's variables, calls the core, and stores
//! the result back. This module goes with the hooks.

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

impl rc2k_core::strings::bytes::Bytes for Raw {
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

impl rc2k_core::audio::samples::SampleData for RawMut {
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
