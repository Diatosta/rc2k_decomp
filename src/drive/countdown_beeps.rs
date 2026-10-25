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
use crate::physics::flow::a;
use crate::race::work_store::RACE_WORK;

const B: usize = 0x70F3EC;

/// The Rust version.
pub unsafe fn countdown_beeps() -> Result<(), Refused> {
    for (car, block, to) in [(0u32, B, 0x6D1482usize), (1, B + 0xFE0, 0x6D1492)] {
        RACE_WORK.current_car.set(car);
        unsafe { crate::drive::countdown_beep::countdown_beep(a(block), a(to)) }?;
    }
    Ok(())
}
