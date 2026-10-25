//! `hud::tenths` (0x4B1D2B): six times of the HUD's record in tenths (signed `idiv` by
//! 10): +0x24, +0x2C, +0x20, +0x54, +0x58, +0x50 into `dword_75C074` .. `dword_75C088`.
//!
//! In:  ebp the HUD's record
//! Out: eax the last quotient, edx its remainder, ebx 10; the rest kept
//!
//! Caller: `hud::draw_hud`.

use crate::hud::store::HUD;

const FIELDS: [u32; 6] = [0x24, 0x2C, 0x20, 0x54, 0x58, 0x50];

/// The Rust version (see the header): the record's six times in tenths.
pub unsafe fn tenths(record: u32) {
    // Converted: `rc2k_core::hud::tenths::tenths`.
    let times = FIELDS.map(|f| unsafe { ((record + f) as *const i32).read_unaligned() });
    let (tenths, _) = rc2k_core::hud::tenths::tenths(times);
    for (cell, v) in HUD.times.iter().zip(tenths) {
        cell.set(v as u32);
    }
}
