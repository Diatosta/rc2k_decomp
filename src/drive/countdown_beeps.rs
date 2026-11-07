//! `drive::countdown_beeps` (0x440E6C): `drive::countdown_beep` for the two cars of a
//! two-car race (`g_currentCar` 0 and 1, the blocks `g_carBlockStore` + 0 and + 0xFE0, the
//! seconds into `unk_6D1482` and `unk_6D1492`).
//!
//! Out: esi the second block, edi `unk_6D1492`, the rest as the beeps leave them; no flags
//!      read
//!
//! Caller: `drive::replay_step`. Refuses what `countdown_beep` refuses.

use crate::errors::Refused;
use crate::race::owner::Race;

/// The Rust version.
pub fn countdown_beeps(race: &Race) -> Result<(), Refused> {
    for (car, to) in [
        (0u32, &race.frame.panel_and_overlay.panel.lights[0]),
        (1, &race.frame.panel_and_overlay.panel.lights[1]),
    ] {
        race.shared.work.current_car.set(car);
        crate::drive::countdown_beep::countdown_beep(
            race,
            &crate::race::blocks_store::CAR_BLOCKS.cars[car as usize],
            &to.lit,
        )?;
    }
    Ok(())
}
