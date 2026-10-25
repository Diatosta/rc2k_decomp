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
//! Caller: `sub_44157B`. Refuses what the two calls refuse.

use crate::errors::Refused;
use crate::framework::{game_write, rva};
use crate::math::x87_run::{ext_div2, int_80};
use crate::physics::car_store::CAR;
use crate::race::numbers_store::NUMBERS;
use crate::race::vars_store::RACE_VARS;
use crate::scene::draw_store::DRAW;
use rc2k_core::physics::progress;

fn p(x: usize, v: u32) {
    unsafe { game_write(rva(x), v) }
}
const DISTANCE: usize = 0x614C68; // g_nearestCheckpointDistance

/// The Rust version: the car's checkpoint (eax): the nearest within the window about the
/// one it had (`race::nearest_checkpoint_in`), or, when that is too far, of them all
/// (`race::nearest_checkpoint`). The other registers it leaves are read by none of its
/// callers.
// Converted: `rc2k_core::physics::progress::{near_window, search_all}`.
pub unsafe fn find_car_checkpoint() -> Result<u32, Refused> {
    p(0x639824, CAR.position[0].get());
    p(0x63982C, CAR.position[2].get());
    if RACE_VARS.checkpoint_count.get() == 0 {
        return Ok(0);
    }
    let (from, to) = progress::near_window(CAR.checkpoint.get());
    let near = unsafe { crate::race::start_place::nearest_checkpoint_in(from, to) }?;
    DRAW.scratch.set(near.distance);
    p(
        DISTANCE,
        ext_div2(
            int_80(near.distance as i32, false),
            NUMBERS.k65536.bits(),
            0x3F80_0000,
        ),
    );
    if progress::search_all(near.distance) {
        return unsafe { crate::race::start_place::nearest_checkpoint() };
    }
    Ok(near.checkpoint)
}
