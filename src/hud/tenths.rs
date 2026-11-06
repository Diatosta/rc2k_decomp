//! `hud::tenths` (0x4B1D2B): six times of the HUD's record in tenths (signed `idiv` by
//! 10): +0x24, +0x2C, +0x20, +0x54, +0x58, +0x50 into `dword_75C074` .. `dword_75C088`.
//!
//! In:  ebp the HUD's record
//! Out: eax the last quotient, edx its remainder, ebx 10; the rest kept
//!
//! Caller: `hud::draw_hud`.

use crate::errors::Refused;
use crate::hud::owner::Hud;
use crate::pointers::GamePtr;

const FIELDS: [u32; 6] = [0x24, 0x2C, 0x20, 0x54, 0x58, 0x50];

/// The Rust version (see the header): the record's six times in tenths.
pub fn tenths(hud: &Hud, record: GamePtr) -> Result<(), Refused> {
    // Converted: `rc2k_core::hud::tenths::tenths`.
    let times: [i32; 6] = crate::effects::try_from_fn(|n| {
        Ok(record
            .offset(FIELDS[n])
            .read::<i32>("a hud record's field")?)
    })?;
    let (tenths, _) = rc2k_core::hud::tenths::tenths(times);
    for (cell, v) in hud.block.times.iter().zip(tenths) {
        cell.set(v as u32);
    }
    Ok(())
}
