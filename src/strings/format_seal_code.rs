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

use crate::moved::Place;

/// `g_sealCodeText`'s room (0x4E0544 .. the `MEMORYSTATUS` at 0x4E0558, whose first byte
/// was never written by the time the code is read: the text's NUL).
pub const SEAL_TEXT_LEN: usize = 0x14;
/// The code's text: out of the image since 2026-10-24 (phase 5). The two dwords stay in
/// the image, in `race::save_seal`'s cipher block (`value`, `dword_4E0540`).
pub static SEAL_TEXT: Place<[u8; SEAL_TEXT_LEN]> = Place::new([0; SEAL_TEXT_LEN]);

/// The 19 characters.
pub fn seal_code(a: u32, b: u32) -> Vec<u8> {
    rc2k_core::strings::format::seal_code(a, b)
}

/// The Rust version on the game's variables.
pub fn format_seal_code(a: u32, b: u32) -> Result<(), crate::errors::BadInput> {
    crate::race::seal_store::SEAL_WORK.sealed[0].set(a);
    crate::race::seal_store::SEAL_WORK.sealed[1].set(b);
    let code = seal_code(a, b);
    crate::pointers::GamePtr::new(SEAL_TEXT.addr() as u32).write_bytes(&code, "the seal text")
}
