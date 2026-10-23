//! `race::panel_neighbours` (0x45AD36): in mode 2 (`dword_62389C`), the first view's place
//! and the cars before and behind it into the panel (see `race/frame_draw.rs`): the place
//! (+0x654), +0x8C x 10, the car behind's +0x24 (-1 at the last place `dword_6051B4`) and
//! its label (`race::car_label` at 0x6D1838 of `dword_605034[place]`), +0x634, the car
//! ahead's +0x24 (-1 at place 1) and its label (0x6D1844, `dword_605030[place]`),
//! -(+0x630). The cars by place are `dword_604EC8[]` (place 0 first).
//!
//! In:  nothing
//! Out: in mode 2: eax -(+0x630), ebx the place, ecx the car ahead's value, esi the block,
//!      edi 0x6D1844, ebp the car ahead's block; else all kept (a `call` follows)
//!
//! Caller: `sub_45AB36` (0x45AB3B), each frame drawn. Refuses (before anything) a
//! neighbour's car number past 7 or a null block. Shadow mode: by state (the panel and the
//! two labels). Self-test: the game's blocks, the place tables set (places 1, 3, the last),
//! mode 2 and not; all put back (skipped with a null block); a car 8 refused.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::race::frame_draw::*;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

// dword_62389C
// dword_6051B4
// dword_604EC8[]
// dword_605030[]

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { panel_neighbours() }
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
