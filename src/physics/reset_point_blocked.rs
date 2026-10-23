//! `physics::reset_point_blocked` (0x43F0DF): whether the checkpoint `dword_614C50` is no
//! place to put a reset car, as `sub_43EFEF` ("cant reset car anywhere - waiting a
//! second") asks it of the other cars' blocks (esi 0x7103CC, 0x7113AC, 0x71238C, ...).
//! Only when the block's +0x25C is 2 (by the look of it, a car in the stage; a reading):
//! CF when the checkpoint is within 8 of the start or the end (`g_checkpointCount` - 8),
//! or within -8 .. 7 of the block's +0x94 (its checkpoint). The comparisons take the sign
//! of the difference (`js`/`jns`), as here.
//!
//! In:  esi the block
//! Out: CF; eax and ebx as the path leaves them (see `reset_point_blocked_at`); the rest
//!      kept. ZF is not read (`jb`).
//!
//! Caller: `sub_43EFEF` (six calls), when a car is reset onto the line: not every frame.
//! Shadow mode: `physics/flag_result.rs`.
//! Self-test: a copy of the original on test variables, kinds 1 .. 3, checkpoints across
//! the window and the stage's ends, counts small and large.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{Regs, game_addr, rva};
use crate::physics::flag_result::{FlagResult, flags_entry};
use rc2k_core::physics::line_places::{self, Blocked};

const CANDIDATE_RVA: usize = rva(0x614C50); // dword_614C50: the checkpoint tried
const CHECKPOINT_COUNT_RVA: usize = rva(0x623640); // g_checkpointCount

unsafe fn at(a: u32) -> u32 {
    unsafe { (a as *const u32).read_unaligned() }
}

/// The Rust core: the candidate and count read at `candidate`, `count`; `r` the registers
/// at entry.
// Converted: `rc2k_core::physics::line_places::reset_point_blocked`.
pub unsafe fn reset_point_blocked_at(candidate: u32, count: u32, r: &Regs) -> FlagResult {
    let b = unsafe {
        line_places::reset_point_blocked(
            at(r.esi + 0x25C),
            at(candidate),
            || at(count),
            || at(r.esi + 0x94),
        )
    };
    let (eax, ebx) = match b {
        Blocked::Away => (r.eax, None),
        Blocked::Start => (unsafe { at(candidate) }, None),
        Blocked::End { end } => (unsafe { at(candidate) }, Some(end)),
        Blocked::Gap { end, gap, .. } => (gap, Some(end)),
    };
    FlagResult {
        eax,
        ecx: None,
        ebx,
        carry: b.carry(),
        zero: None,
    }
}

/// The Rust version.
pub unsafe fn reset_point_blocked(r: &Regs) -> FlagResult {
    unsafe {
        reset_point_blocked_at(
            game_addr(CANDIDATE_RVA) as u32,
            game_addr(CHECKPOINT_COUNT_RVA) as u32,
            r,
        )
    }
}

fn rust(r: &Regs) -> Result<FlagResult, Refused> {
    Ok(unsafe { reset_point_blocked(r) })
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    flags_entry(r, &rust)
}
