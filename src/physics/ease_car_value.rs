//! `physics::ease_car_value` (0x4554CA): unless `dword_6235E8` is set, `dword_71AF94`
//! follows `dword_71AF90` << 8: set to it when `dword_71AF88` is set, else stepped by 4
//! towards it (up when the difference's sign is clear, `jns`). What the values are is
//! not known.
//!
//! In:  nothing
//! Out: eax `dword_71AF90` << 8 (when `dword_6235E8` is 0); the rest kept; no flags read
//!
//! Caller: `sub_43B635` (0x43B705), every physics step.
//! Self-test: a copy of the original on test variables, the switches and values above,
//! at and below, near the sign's turn.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{EAX, Regs, game_addr, rva};
use crate::physics::cells::{get, put_changed};
use crate::screens::harness::Left;
use crate::service::harness::state_entry;
use rc2k_core::physics::easing::ease_value;

/// The variables (IDA addresses): the block, the jump switch, the target, the value.
const VARIABLES: [usize; 4] = [0x6235E8, 0x71AF88, 0x71AF90, 0x71AF94];

/// The Rust version on the variables at `v`; `r` as the original leaves it.
// Converted: `rc2k_core::physics::easing::ease_value`.
pub unsafe fn ease_at(v: &[u32; 4], r: &mut Regs) {
    unsafe {
        let was = get(v[3]);
        let mut value = was;
        if let Some(eax) = ease_value(get(v[0]), get(v[1]), get(v[2]), &mut value) {
            r.eax = eax;
        }
        put_changed(v[3], was, value);
    }
}

fn game() -> [u32; 4] {
    VARIABLES.map(|x| game_addr(rva(x)) as u32)
}

/// The Rust version.
pub unsafe fn ease_car_value(r: &mut Regs) {
    unsafe { ease_at(&game(), r) }
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    let mut g = *r;
    unsafe { ease_car_value(&mut g) };
    Ok(Left::nothing().with(EAX, g.eax))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
