//! `scene::step_loading_bar` (0x446786): moves the loading's progress on and shows it.
//!
//! Takes: how many of the system textures are done
//!
//! Unless there are no system textures (the system textures total): the progress = the low
//! end plus the done count times (the high end less the low) over the
//! textures' number, plus 1. Then the loading bar's function is called (the front end's
//! set-up puts `race::loading_bar_frame` there, which draws). One caller,
//! `scene::reload_system_textures`. Refuses no function to call.

use crate::errors::{BadInput, Refused};
use crate::race::owner::Race;
use crate::race::owner_shared::RaceShared;
use crate::scene::owner::scene;

// ---- game data ----

/// `RaceVars.bar_span[0]` the low end, `RaceVars.bar_span[1]` the high end.
const BAR: u32 = 0x4467C7; // race::loading_bar_frame
const NOTHING: u32 = 0x4467C6; // nullsub_8

// ---- Rust version ----

fn ends(shared: &RaceShared) -> rc2k_core::scene::loading::BarEnds {
    let [low, high] = crate::session::read_bits(&shared.vars.bar_span);
    rc2k_core::scene::loading::BarEnds { low, high }
}

/// The progress, or None where it is left as it is. Converted:
/// `rc2k_core::scene::loading::BarEnds::progress`.
fn progress(shared: &RaceShared, done: u32) -> Option<u32> {
    let textures: u32 = scene().textures.places.totals[3].get();
    ends(shared).progress(done, textures)
}

/// `done` of the system textures are done. The loading bar's function is one of the two the
/// loading puts there: `race::loading_bar_frame` or the empty function after it.
pub fn step_loading_bar(race: &Race, done: u32) -> Result<(), Refused> {
    let shared = &race.shared;
    let show: u32 = shared.vars.bar_function.get();
    if show == 0 {
        return Err(BadInput("no function to show the loading with").into());
    }
    if let Some(progress) = progress(shared, done) {
        shared.vars.bar_share.set(progress);
    }
    if show == BAR {
        crate::race::loading_bar_frame::loading_bar_frame(race)
    } else if show == NOTHING {
        Ok(())
    } else {
        Err(BadInput("a loading bar function that is neither of the loading's").into())
    }
}
