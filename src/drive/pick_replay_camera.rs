//! For a replay's camera wish (0x23, 0x24, 0x25: kinds 0, 1, 2), a camera mode is drawn
//! (`math::next_random` x 0x34, high dword) until its record is of that kind (0x23: until it is
//! NOT of kind 0, a slip kept as is); modes 2 and 3 become 4 when a car's kind is
//! past 0x16 (the second car's only with two cars). Other wishes: nothing.
//!
//! Refuses (before anything) a wish no mode stops (it would draw for ever).

use crate::errors::{BadInput, Refused};
use crate::race::owner::Race;
use crate::race::view_vars_store::ViewVars;
use rc2k_core::drive::replay_views::{MODES as MODE_COUNT, draw, fits, kept_mode, wish_kind};

fn kind_of(race: &Race, reading: &crate::scene::reading::Reading, mode: u32) -> u32 {
    // `view_records`
    reading.keep(
        ViewVars::record_of(&race.frame.view_and_records.borrow(), mode as usize)
            .map(|record| record.kind),
    )
}

/// The camera wish `wish`; the mode kept, or `None` for a wish that draws
/// nothing (its one caller draws only for the wishes that
/// draw).
pub fn pick_replay_camera(race: &Race, wish: u32) -> Result<Option<u32>, Refused> {
    let Some(kind) = wish_kind(wish) else {
        return Ok(None);
    };
    let reading = crate::scene::reading::Reading::new();
    let fits = |mode: u32| fits(kind, kind_of(race, &reading, mode));
    let any = (0..MODE_COUNT).any(fits);
    reading.finish()?;
    if !any {
        return Err(BadInput("no camera mode the wish stops at").into());
    }
    let mode = loop {
        let (_, mode) = draw(race.shared.random.next_random());
        if fits(mode) {
            break mode;
        }
    };
    let mode = kept_mode(
        mode,
        || race.car_blocks().cars[0].car_kind.get(),
        || race.shared.view.get().players.get() != 1,
        || race.car_blocks().cars[1].car_kind.get(),
    );
    Ok(Some(mode))
}
