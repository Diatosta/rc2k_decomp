//! `drive::countdown_beeps` (0x440E6C): `drive::countdown_beep` for the two cars of a
//! two-car race (`g_currentCar` 0 and 1, the blocks `g_carBlockStore` + 0 and + 0xFE0, the
//! seconds into `unk_6D1482` and `unk_6D1492`).
//!
//! In:  nothing
//! Out: esi the second block, edi `unk_6D1492`, the rest as the beeps leave them; no flags
//!      read
//!
//! Caller: `drive::replay_step`. Refuses what `countdown_beep` refuses.

use crate::errors::Refused;
use crate::race::panel_store::PANEL;
use crate::race::work_store::RACE_WORK;

/// The Rust version.
pub fn countdown_beeps() -> Result<(), Refused> {
    for (car, to) in [(0u32, &PANEL.lights[0]), (1, &PANEL.lights[1])] {
        RACE_WORK.current_car.set(car);
        crate::drive::countdown_beep::countdown_beep(
            crate::race::blocks_store::block_addr(car as usize),
            &to.lit,
        )?;
    }
    Ok(())
}
