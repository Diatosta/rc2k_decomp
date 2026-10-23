//! `drive::two_car_order` (0x460CDA): the two cars' places in a two-car race (the car
//! blocks' +0xFA4, `g_carBlockStore`): -1 for both outside mode 0 (`dword_62389C`) or with
//! one car (`dword_63DB94` 1); left as they are while either has +0xA8 set; else 1 for the
//! one ahead (checkpoint +0x94, then +0x624, each by the sign of the `cmp`), 2 for the
//! other.
//!
//! In:  nothing
//! Out: ebx the last figure compared; the rest kept; no flags read
//!
//! Caller: `drive::replay_step`.
//! Self-test: the original and the Rust version on the two blocks' figures (saved and put
//! back): modes, ahead, behind, level.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{EBX, Regs};
use crate::physics::flow::{g, p};
use crate::screens::harness::Left;
use crate::service::harness::state_entry;
use rc2k_core::drive::places;

const B: usize = 0x70F3EC; // g_carBlockStore
const NEXT: usize = 0xFE0;

/// The Rust version.
// Converted: `rc2k_core::drive::places::two_car_order`.
pub fn two_car_order(entry: &Regs) -> Left {
    let figure = |car: usize, o: u32| g(B + car * NEXT + o as usize);
    let order = places::two_car_order(g(0x62389C), g(0x63DB94), figure, entry.ebx);
    if let Some([x, y]) = order.places {
        p(B + 0xFA4, x);
        p(B + NEXT + 0xFA4, y);
    }
    Left::nothing().with(EBX, order.last)
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(two_car_order(r))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
