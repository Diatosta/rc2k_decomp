//! `menu::player_entered` (0x4A95EB): eax kept when player ecx's +0xD0 is 1 (a reading: the
//! player takes part), else eax 0.
//!
//! In:  ecx the player, eax a value
//! Out: edi the player, eax as above; the rest kept
//!
//! Caller: `sub_473126` (four times). Refuses a player past 3 or missing, before anything.

use crate::errors::{BadInput, Refused};
use crate::menu::standings::*;

/// Player `n`'s (ecx) record (edi) and `value` (eax) if the player takes part, else 0.
pub fn player_entered(n: u32, value: u32) -> Result<(u32, u32), Refused> {
    if n >= 4 || player(n) == 0 {
        return Err(BadInput("a player past 3 or missing").into());
    }
    let record = player(n);
    // Converted: `rc2k_core::menu::event::if_entered`.
    Ok((
        record,
        rc2k_core::menu::event::if_entered(get(record + 0xD0)?, value),
    ))
}
