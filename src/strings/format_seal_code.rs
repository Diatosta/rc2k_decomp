//! `strings::format_seal_code` (0x4026C9): two dwords as "AABB-CCDD-EEFF-GGHH" in
//! `byte_4E0544`: each dword's bytes from the lowest, two hex digits each
//! (`strings::hex_append`), a '-' after every second byte; no NUL (the buffer keeps whatever
//! follows). The dwords are kept in `value` / `dword_4E0540`. (Stream A, part F1: the stage
//! results page shows a seal code with it.)
//!
//! In:  eax, ebx = the two dwords
//! Out: every register preserved (`pusha` .. `popa`)
//!
//! values by lot.
//! Hook removed 2026-10-21 (conversion phase 4): Rust callers call this directly; the
//! register use above is the original's, for reference.

use crate::framework::{game_addr, game_write, rva};

/// The 19 characters.
pub fn seal_code(a: u32, b: u32) -> Vec<u8> {
    rc2k_core::strings::format::seal_code(a, b)
}

/// The Rust version on the game's variables.
pub unsafe fn format_seal_code(a: u32, b: u32) {
    unsafe {
        game_write(rva(0x4E053C), a);
        game_write(rva(0x4E0540), b);
        let code = seal_code(a, b);
        std::ptr::copy_nonoverlapping(
            code.as_ptr(),
            game_addr(rva(0x4E0544)) as *mut u8,
            code.len(),
        );
    }
}
