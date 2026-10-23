//! `physics::value_notice` (0x45B357): notice 0xCE by `dword_71A24C` against 300 (the
//! sign of the difference): at 300 or more, with no notice shown (`dword_71A700` 0) and
//! 0xCE not the last (`dword_71A6FC`), `dword_71A6F4` = 0xCE and `dword_71A704` = 0;
//! below 300 with 0xCE the last, `dword_71A700` and `dword_71A6FC` 0. What the figure is
//! is not known.
//!
//! In:  nothing
//! Out: every register kept; flags not read (`sub_43B635` returns next, its callers
//!      go on with `inc`, calls)
//!
//! Caller: `sub_43B635` (0x43B9AE), its last call, every physics step.
//! Self-test: `physics/copy_hooks_test.rs`, 16 cases.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{Regs, game_addr, rva};
use crate::physics::cells::{get, put_changed};
use crate::screens::harness::Left;
use crate::service::harness::state_entry;
use rc2k_core::physics::notices::Notice;

/// The figure, shown, last, id, time (IDA addresses).
const VARIABLES: [usize; 5] = [0x71A24C, 0x71A700, 0x71A6FC, 0x71A6F4, 0x71A704];
// Converted: `rc2k_core::physics::notices::Notice::value_notice`.
unsafe fn core(v: &[u32]) {
    unsafe {
        let was = Notice {
            shown: get(v[1]),
            last: get(v[2]),
            id: get(v[3]),
            time: get(v[4]),
            ..Notice::default()
        };
        let mut n = was;
        n.value_notice(get(v[0]));
        put_changed(v[3], was.id, n.id);
        put_changed(v[4], was.time, n.time);
        put_changed(v[1], was.shown, n.shown);
        put_changed(v[2], was.last, n.last);
    }
}

fn game() -> Vec<u32> {
    VARIABLES
        .iter()
        .map(|&x| game_addr(rva(x)) as u32)
        .collect()
}

/// The Rust version.
pub unsafe fn value_notice() {
    unsafe { core(&game()) }
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { value_notice() };
    Ok(Left::nothing())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
