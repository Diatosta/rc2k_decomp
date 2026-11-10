//! `drive::pick_replay_camera` (0x460AA6): for a replay's camera wish eax (0x23, 0x24, 0x25: kinds
//! 0, 1, 2), a camera mode drawn (`math::next_random` x 0x34, high dword) until its record
//! (`off_606320[mode]`) is of that kind (0x23: until it is NOT of kind 0, the original's `jz`;
//! notes/bugs.md); modes 2 and 3 become 4 when a car's kind (+0x24, `g_carBlockStore`) is past 0x16
//! (the second car's only with two cars, `dword_63DB94` not 1). Other wishes: nothing.
//!
//! Takes: the wish
//!
//! Caller: `drive::replay_cameras`. Refuses (before anything) a wish no mode stops (the
//! original draws for ever).

use crate::errors::{BadInput, Refused};
use crate::race::blocks_store::CAR_BLOCKS;
use crate::race::owner_shared::RaceShared;
use crate::race::view_vars_store::ViewVars;
use rc2k_core::drive::replay_views::{MODES as MODE_COUNT, draw, fits, kept_mode, wish_kind};

fn kind_of(reading: &crate::scene::reading::Reading, mode: u32) -> u32 {
    // off_606320
    reading.keep(ViewVars::record_of(mode as usize).map(|record| record.kind.get()))
}

// Converted: `rc2k_core::drive::replay_views::{wish_kind, fits, draw, kept_mode}`.
/// The camera wish `wish` (eax); the mode kept (edx), or `None` for a wish that draws
/// nothing (the original leaves edx then; its one caller draws only for the wishes that
/// draw).
pub fn pick_replay_camera(shared: &RaceShared, wish: u32) -> Result<Option<u32>, Refused> {
    let Some(kind) = wish_kind(wish) else {
        return Ok(None);
    };
    let reading = crate::scene::reading::Reading::new();
    let fits = |mode: u32| fits(kind, kind_of(&reading, mode));
    let any = (0..MODE_COUNT).any(fits);
    reading.finish()?;
    if !any {
        return Err(BadInput("no camera mode the wish stops at").into());
    }
    let edx = loop {
        let (_, edx) = draw(crate::math::next_random::next_random());
        if fits(edx) {
            break edx;
        }
    };
    let edx = kept_mode(
        edx,
        || CAR_BLOCKS.cars[0].car_kind.bits(),
        || shared.view.players.get() != 1,
        || CAR_BLOCKS.cars[1].car_kind.bits(),
    );
    Ok(Some(edx))
}
