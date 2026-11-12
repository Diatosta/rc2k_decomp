//! `strings::format_seal_code` (0x4026C9): two dwords as "AABB-CCDD-EEFF-GGHH" in
//! the seal text buffer: each dword's bytes from the lowest, two hex digits each
//! (`strings::hex_append`), a '-' after every second byte; no NUL (the buffer keeps whatever
//! follows). The dwords are kept in the cipher block (`value`). (The stage results page shows a seal code with it.)
//!
//! Takes two dwords.
//!
//! values by lot.

/// `g_sealCodeText`'s room (0x4E0544 .. the `MEMORYSTATUS` at 0x4E0558, whose first byte
/// was never written by the time the code is read: the text's NUL).
use crate::race::owner::Race;
pub const SEAL_TEXT_LEN: usize = 0x14;
/// The code's text. The two dwords are in `race::save_seal`'s cipher block (`value`,
/// `dword_4E0540`).
pub static SEAL_TEXT: crate::session::Bytes<SEAL_TEXT_LEN> =
    crate::session::Bytes::new([0; SEAL_TEXT_LEN]);

/// The 19 characters.
pub fn seal_code(a: u32, b: u32) -> Vec<u8> {
    rc2k_core::strings::format::seal_code(a, b)
}

pub fn format_seal_code(race: &Race, a: u32, b: u32) -> Result<(), crate::errors::BadInput> {
    let replay = &race.replay;
    replay.seal_work.sealed[0].set(a);
    replay.seal_work.sealed[1].set(b);
    let code = seal_code(a, b);
    SEAL_TEXT.write_at(0, &code, "the seal text")
}
