//! `physics::lever_pushes` (0x43DDDF): `dword_6140E4` and `dword_6140E8` 0; then, only
//! with `dword_621884` 1 and `dword_71A22C` 0: the lever `flt_614B44`, `B48` 0 and
//! `flt_614C44` 0; a share A / (B + A) of A = |+0x210| + |+0x21C| and B = |+0x214| +
//! |+0x218| (`g_car`, `dword_71A1E8` .. `dword_71A1F4`; a reading: the wheels' loads)
//! into `flt_614C44`, held to `flt_5C5D90` (0) ..= `flt_5C5D38` (1). If B + A is 0 (or no
//! number) the share is A itself and it ends there. Else `flt_614B4C` = `flt_71ACD4` and
//! `flt_614C48` the share, `physics::lever_push`; `flt_614B4C` = `flt_71ACC8` and
//! `flt_614C48` 1 less it, `physics::lever_push`. It ends through the `retn` at 0x43DFB0.
//!
//! In:  nothing
//! Out: as the second `lever_push` leaves them when it ran, else all kept; no flags read
//!
//! Callers: `sub_43B635` (0x43B831, 0x43B8AA), every physics step.
//! Self-test: the original and the Rust version on the game's variables (saved and put
//! back): both switches, loads by lot and all 0, under three precisions.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{Regs, game_read, game_write, rva};
use crate::math::x87;
use crate::math::x87_run::{lever_share, one_less};
use crate::physics::lever_push::lever_push;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;
use std::cmp::Ordering;

fn g(x: usize) -> u32 {
    unsafe { game_read(rva(x)) }
}

fn p(x: usize, v: u32) {
    unsafe { game_write(rva(x), v) }
}

const SHARE: usize = 0x614C44; // flt_614C44
const ZERO: usize = 0x5C5D90; // flt_5C5D90
const ONE: usize = 0x5C5D38; // flt_5C5D38

/// `fld limit; fld share; fcompp` and its C0: share below limit, or unordered.
fn below(share: u32, limit: u32) -> bool {
    !matches!(
        f32::from_bits(share).partial_cmp(&f32::from_bits(limit)),
        Some(Ordering::Greater | Ordering::Equal)
    )
}

/// The Rust version.
pub unsafe fn lever_pushes() -> Left {
    p(0x6140E4, 0);
    p(0x6140E8, 0);
    if g(0x621884) != 1 || g(0x71A22C) != 0 {
        return Left::nothing();
    }
    p(0x614B44, 0);
    p(0x614B48, 0);
    p(SHARE, 0);
    let (share, zero) = lever_share(g(0x71A1E8), g(0x71A1F4), g(0x71A1EC), g(0x71A1F0), g(ZERO));
    p(SHARE, share);
    if zero {
        return Left::nothing();
    }
    if below(g(SHARE), g(ZERO)) {
        p(SHARE, x87::loaded(g(ZERO)));
    }
    if !below(g(SHARE), g(ONE)) {
        p(SHARE, x87::loaded(g(ONE)));
    }
    p(0x614B4C, x87::loaded(g(0x71ACD4)));
    p(0x614C48, x87::loaded(g(SHARE)));
    unsafe { lever_push() };
    p(0x614B4C, x87::loaded(g(0x71ACC8)));
    p(0x614C48, one_less(g(0x614C48)));
    unsafe { lever_push() }
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    Ok(unsafe { lever_pushes() })
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
