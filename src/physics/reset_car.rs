//! `physics::reset_car` (0x43EFEF): the work car put back on the line. Its checkpoint
//! (`dword_71A06C`, `g_car` +0x94) into `dword_6365E4`, `dword_6365EC` 0. With
//! `dword_62389C` 2 a free checkpoint is looked for: offsets d = 0, 1, .. 0x3F
//! (`dword_614C4C`), each tried as +d then -d (`dword_614C50` = the checkpoint + it), a
//! try failing when `physics::reset_point_blocked` gives CF for any of the blocks at
//! 0x7103CC, 0x7113AC, 0x71238C (tried in that order, the first CF ends it). The first
//! free one goes into `dword_6365E4`; with none, "cant reset car anywhere - waiting a sec"
//! is logged (`log_write_l1_nl`, inside `pusha` / `popa`), +0x1E8 (`dword_71A1C0`, the
//! stuck count) less 100, and it returns. Else `race::place_car_on_line`,
//! `physics::reset_car_on_line`, and +0xFAC (`dword_71AF84`) 10.
//!
//! In:  nothing (the registers are passed on to the calls)
//! Out: as the last call leaves them (eax the last try's checkpoint and esi the last
//!      block when none was free); no flags read (every caller returns next)
//!
//! Callers: `physics::reset_when_stuck`, `sub_43A0D3`, `sub_43BDD1`.
//! Refuses what the calls refuse. Shadow mode runs the original alone
//! (`physics/reset_parts.rs`).
//! Self-test: a copy whose calls go to the stand-ins (place 13's carry by a pattern),
//! against the Rust version with the same: modes 2 and others, patterns free at once,
//! after some tries, and never (the log line, a stand-in too); all put back.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{Regs, game_addr, game_read, game_write, rva};
use crate::physics::reset_parts::{BLOCKED, LOG, ON_LINE, reset_entry};
use rc2k_core::physics::line_places::{RESET_TRIES, reset_offsets};

const LINE_PLACE: usize = rva(0x6365E4); // dword_6365E4
const OFFSET: usize = rva(0x614C4C); // dword_614C4C
const CANDIDATE: usize = rva(0x614C50); // dword_614C50
const STUCK: usize = rva(0x71A1C0); // g_car + 0x1E8
const BLOCKS: [usize; 3] = [0x7103CC, 0x7113AC, 0x71238C];
const NOWHERE_TEXT: usize = rva(0x4DBB01); // "cant reset car anywhere - waiting a sec"

fn g(a: usize) -> u32 {
    unsafe { game_read(a) }
}

fn p(a: usize, v: u32) {
    unsafe { game_write(a, v) }
}

/// One try: the candidate set; true when a block says CF.
fn blocked(r: &mut Regs) -> Result<bool, Refused> {
    r.eax = g(LINE_PLACE).wrapping_add(g(OFFSET));
    p(CANDIDATE, r.eax);
    for b in BLOCKS {
        r.esi = game_addr(rva(b)) as u32;
        crate::physics::reset_parts::game_part(BLOCKED, r)?;
        if r.carry() {
            return Ok(true);
        }
    }
    Ok(false)
}

/// The Rust version, the calls through `part` (places 13, 14, 6, 16).
// Converted: `rc2k_core::physics::line_places::reset_offsets`.
pub unsafe fn reset_car(entry: &Regs) -> Result<Regs, Refused> {
    let mut r = *entry;
    r.eax = g(rva(0x71A06C));
    p(LINE_PLACE, r.eax);
    p(rva(0x6365EC), 0);
    if g(rva(0x62389C)) == 2 {
        let mut free = false;
        for offset in reset_offsets() {
            p(OFFSET, offset);
            if !blocked(&mut r)? {
                free = true;
                break;
            }
        }
        if !free {
            p(OFFSET, RESET_TRIES);
            let mut l = r;
            l.eax = game_addr(NOWHERE_TEXT) as u32;
            crate::physics::reset_parts::game_part(LOG, &mut l)?;
            p(STUCK, g(STUCK).wrapping_sub(100));
            return Ok(r);
        }
        r.eax = g(CANDIDATE);
        p(LINE_PLACE, r.eax);
    }
    crate::physics::reset_parts::game_part(6, &mut r)?;
    crate::physics::reset_parts::game_part(ON_LINE, &mut r)?;
    p(rva(0x71AF84), 10);
    Ok(r)
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    reset_entry(r, reset_car)
}
