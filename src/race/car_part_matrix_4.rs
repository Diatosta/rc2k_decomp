//! `race::car_part_matrix_4` (0x461FE4): part 4's matrix (`flt_6418A0`, the car's +0x800) times
//! the body's turn, the car's turn and `g_viewMatrix` into `g_madeObjectMatrix` (see
//! `race/car_draw.rs`).
//!
//! In:  nothing
//! Out: esi, edi, ebp the last product's matrices, the rest as `scene::multiply_matrices`
//!      leaves them (a `call` follows)
//!
//! Caller: `sub_4616AA`. Shadow mode: by state. Self-test: the game's matrices and two
//! sets of test floats in them; all put back.
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
    unsafe { car_part_matrix(&mut out, PART_MATRICES[3]) };
    Ok(all_regs(&out))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
