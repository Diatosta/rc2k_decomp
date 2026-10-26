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
