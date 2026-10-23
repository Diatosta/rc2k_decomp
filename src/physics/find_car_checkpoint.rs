//! `physics::find_car_checkpoint` (0x44139D): the work car's checkpoint looked for near
//! the last one. The car's x and z into `dword_639824` / `dword_63982C`; with no
//! checkpoints eax 0. Else `race::nearest_checkpoint_in` over `g_car` +0x98 - 1 ..
//! +0x98 + 1 (ecx, edx); its distance (edi) through `dword_62400C`, / 65536 into
//! `g_nearestCheckpointDistance`; above 0xA0000 (unsigned) `race::nearest_checkpoint`
//! over them all (eax), else eax the nearest (ebp).
//!
//! In:  nothing (the registers are passed on)
//! Out: eax the checkpoint; the rest as the calls leave them; no flags read (the caller
//!      stores eax)
//!
//! Caller: `sub_44157B`.
//! Refuses what the two calls refuse. Shadow mode runs the original alone
//! (`physics/reset_parts.rs`).
//! Self-test: a copy whose calls go to the stand-ins (places 17 and 8), against the Rust
//! version with the same: no checkpoints, distances below and above 0xA0000.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{Regs, game_read, game_write, rva};
use crate::math::x87_run::{ext_div2, int_80};
use crate::physics::reset_parts::NEAREST_IN;
use rc2k_core::physics::progress;

fn g(x: usize) -> u32 {
    unsafe { game_read(rva(x)) }
}

fn p(x: usize, v: u32) {
    unsafe { game_write(rva(x), v) }
}

const COUNT: usize = 0x623640; // g_checkpointCount
const DISTANCE: usize = 0x614C68; // g_nearestCheckpointDistance

/// The Rust version, the calls through `part` (places 17 and 8).
// Converted: `rc2k_core::physics::progress::{near_window, search_all}`.
pub unsafe fn find_car_checkpoint(entry: &Regs) -> Result<Regs, Refused> {
    let mut r = *entry;
    p(0x639824, g(0x71A10C));
    r.eax = g(0x71A114);
    p(0x63982C, r.eax);
    if g(COUNT) == 0 {
        r.eax = 0;
        return Ok(r);
    }
    (r.ecx, r.edx) = progress::near_window(g(0x71A070));
    crate::physics::reset_parts::game_part(NEAREST_IN, &mut r)?;
    p(0x62400C, r.edi);
    p(
        DISTANCE,
        ext_div2(int_80(r.edi as i32, false), g(0x5C5D98), 0x3F80_0000),
    );
    if progress::search_all(r.edi) {
        crate::physics::reset_parts::game_part(8, &mut r)?;
    } else {
        r.eax = r.ebp;
    }
    Ok(r)
}
