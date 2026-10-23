//! `race::car_body_matrix` (0x461EA1): the turns about x and z by 0 multiplied, then times
//! the body's turn, the car's turn and `g_viewMatrix` into `g_madeObjectMatrix` (see
//! `race/car_draw.rs`).
//!
//! In:  nothing
//! Out: edx 0; esi, edi, ebp the last product's matrices, the rest as
//!      `scene::multiply_matrices` leaves them (a `call` follows)
//!
//! Callers: `sub_4616AA`, 0x4295CE. Shadow mode: by state. Self-test: the game's matrices
//! and two sets of test floats in them; all put back.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::race::car_draw::*;
use crate::race::frame_draw::all_regs;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

fn rust(r: &Regs) -> Result<Left, Refused> {
    let mut out = *r;
    unsafe { car_body_matrix(&mut out) };
    Ok(all_regs(&out))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
