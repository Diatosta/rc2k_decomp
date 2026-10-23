//! `physics::count_down_notice` (0x440F08): while a notice is shown (`dword_71A700` set)
//! and its time `dword_71A704` is not 0, the time less 1; at 0 the notice is cleared
//! (`dword_71A700` 0). `physics::take_notice` sets them.
//!
//! In:  nothing
//! Out: eax the time before (when `dword_71A700` is set); the rest kept; no flags read
//!
//! Caller: `sub_43B635` (0x43B919), every physics step.
//! Self-test: a copy of the original on test variables, shown or not, times 0, 1, 2,
//! large.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{EAX, Regs, game_addr, rva};
use crate::physics::cells::{get, put_changed};
use crate::screens::harness::Left;
use crate::service::harness::state_entry;
use rc2k_core::physics::notices::Notice;

/// The Rust version on (shown, time) at `v`; `r` as the original leaves it.
// Converted: `rc2k_core::physics::notices::Notice::count_down`.
pub unsafe fn count_down_at(v: [u32; 2], r: &mut Regs) {
    unsafe {
        let was = Notice {
            shown: get(v[0]),
            time: get(v[1]),
            ..Notice::default()
        };
        let mut n = was;
        if let Some(eax) = n.count_down() {
            r.eax = eax;
        }
        put_changed(v[1], was.time, n.time);
        put_changed(v[0], was.shown, n.shown);
    }
}

fn game() -> [u32; 2] {
    [0x71A700usize, 0x71A704].map(|x| game_addr(rva(x)) as u32)
}

/// The Rust version.
pub unsafe fn count_down_notice(r: &mut Regs) {
    unsafe { count_down_at(game(), r) }
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    let mut g = *r;
    unsafe { count_down_notice(&mut g) };
    Ok(Left::nothing().with(EAX, g.eax))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
