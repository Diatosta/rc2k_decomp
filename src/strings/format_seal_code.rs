//! Two dwords as "AABB-CCDD-EEFF-GGHH" in the seal text buffer: each dword's bytes from the
//! lowest, two hex digits each, a '-' after every second byte; no NUL. The dwords are kept in
//! `race::save_seal`'s cipher block (`value`). The stage results page shows a seal code with it.

/// The seal text buffer's room; the byte after the 19 characters is the text's NUL.
use crate::race::owner::Race;
pub const SEAL_TEXT_LEN: usize = 0x14;
/// The code's text.
pub static SEAL_TEXT: crate::session::GameThread<crate::session::PlainBytes<SEAL_TEXT_LEN>> =
    crate::session::GameThread::new(crate::session::PlainBytes::new([0; SEAL_TEXT_LEN]));

/// The 19 characters.
pub fn seal_code(a: u32, b: u32) -> Vec<u8> {
    rc2k_core::strings::format::seal_code(a, b)
}

pub fn format_seal_code(race: &Race, a: u32, b: u32) -> Result<(), crate::errors::BadInput> {
    let replay = &race.replay;
    replay.seal_work().sealed[0].set(a);
    replay.seal_work().sealed[1].set(b);
    let code = seal_code(a, b);
    SEAL_TEXT.get().write_at(0, &code, "the seal text")
}
