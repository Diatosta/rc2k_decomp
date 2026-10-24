//! `drive::pick_replay_camera` (0x460AA6): for a replay's camera wish eax (0x23, 0x24,
//! 0x25: kinds 0, 1, 2), a camera mode drawn (`math::next_random` x 0x34, high dword) until
//! its record (`off_606320[mode]`) is of that kind (0x23: until it is NOT of kind 0, the
//! original's `jz`; notes/bugs.md); modes 2 and 3 become 4 when a car's
//! kind (+0x24, `g_carBlockStore`) is past 0x16 (the second car's only with two cars,
//! `dword_63DB94` not 1). Other wishes: nothing.
//!
//! In:  eax the wish
//! Out: edx the mode; eax the last draw's product, ebx its record; the rest kept; no flags
//!      read
//!
//! Caller: `drive::replay_cameras`.
//! Refuses (before anything) a wish no mode stops (the original draws for ever).
//! Self-test: the original and the Rust version on the game's records, the random state
//! saved and put back, the wishes and car kinds.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, Refused};
use crate::physics::flow::g;
use rc2k_core::drive::replay_views::{MODES as MODE_COUNT, draw, fits, kept_mode, wish_kind};

const MODES: usize = 0x606320; // off_606320
const B: usize = 0x70F3EC; // g_carBlockStore

fn kind_of(mode: u32) -> u32 {
    unsafe { (g(MODES + 4 * mode as usize) as *const u32).read_unaligned() }
}

/// The Rust version.
// Converted: `rc2k_core::drive::replay_views::{wish_kind, fits, draw, kept_mode}`.
/// The camera wish `wish` (eax); the mode kept (edx), or `None` for a wish that draws
/// nothing (the original leaves edx then; its one caller draws only for the wishes that
/// draw).
pub unsafe fn pick_replay_camera(wish: u32) -> Result<Option<u32>, Refused> {
    let Some(kind) = wish_kind(wish) else {
        return Ok(None);
    };
    let fits = |mode: u32| fits(kind, kind_of(mode));
    if !(0..MODE_COUNT).any(fits) {
        return Err(BadInput("no camera mode the wish stops at").into());
    }
    let edx = loop {
        let (_, edx) = draw(unsafe { crate::math::next_random::next_random() });
        if fits(edx) {
            break edx;
        }
    };
    let edx = kept_mode(
        edx,
        || g(B + 0x24),
        || g(0x63DB94) != 1,
        || g(B + 0xFE0 + 0x24),
    );
    Ok(Some(edx))
}
