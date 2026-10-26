//! `drive::countdown_beeps` (0x440E6C): `drive::countdown_beep` for the two cars of a
//! two-car race (`g_currentCar` 0 and 1, the blocks `g_carBlockStore` + 0 and + 0xFE0, the
//! seconds into `unk_6D1482` and `unk_6D1492`).
//!
//! In:  nothing
//! Out: esi the second block, edi `unk_6D1492`, the rest as the beeps leave them; no flags
//!      read
//!
//! Caller: `drive::replay_step`.
//! Refuses what `countdown_beep` refuses.
//! Self-test: the original and the Rust version on the two blocks' figures (saved and put
//! back), neither car with a sound slot.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{EDI, ESI, Regs};
use crate::physics::flow::{a, p};
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

const B: usize = 0x70F3EC;

/// The Rust version.
pub unsafe fn countdown_beeps(entry: &Regs) -> Result<Left, Refused> {
    let mut r = *entry;
    for (car, block, to) in [(0u32, B, 0x6D1482usize), (1, B + 0xFE0, 0x6D1492)] {
        p(0x71B3F8, car);
        r.esi = a(block);
        r.edi = a(to);
        unsafe { crate::drive::countdown_beep::countdown_beep(r.esi, r.edi) }?.apply(&mut r);
    }
    Ok(crate::race::stage_prefs::left_of(&r)
        .with(ESI, r.esi)
        .with(EDI, r.edi))
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { countdown_beeps(r) }
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
