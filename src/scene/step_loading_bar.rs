//! `scene::step_loading_bar` (0x446786): moves the loading's progress on and shows it.
//!
//! Takes: how many of the system textures are done
//!
//! Unless there are no system textures (`dword_5DC974`): the progress `dword_623700` = the low
//! end `dword_623634` plus eax times (the high end `dword_623638` less the low) over the
//! textures' number, plus 1. Then the function `dword_623704` names is called (the front end's
//! set-up puts `race::loading_bar_frame` there, which draws). One caller,
//! `scene::reload_system_textures`. Refuses no function to call.

use crate::errors::{BadInput, Refused};
use crate::race::owner::Race;
use crate::race::owner_shared::RaceShared;
use crate::scene::owner::scene;

// ---- game data ----

/// `dword_623634` the low end, `dword_623638` the high end.
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

/// The Rust version: `done` of the system textures are done. The function
/// `dword_623704` names is one of the two the loading puts there: `race::loading_bar_frame`
/// (0x4467C7) or the empty function after it (0x4467C6); they read no register, so none of
/// the original's (ebp the span, eax / edx / ebx the progress, esi the function) is passed.
pub fn step_loading_bar(race: &Race, done: u32) -> Result<(), Refused> {
    let shared = &race.shared;
    let show: u32 = shared.vars.bar_function.bits();
    if show == 0 {
        return Err(BadInput("no function to show the loading with").into());
    }
    if let Some(progress) = progress(shared, done) {
        shared.vars.bar_share.set_bits(progress);
    }
    if show == BAR {
        crate::race::loading_bar_frame::loading_bar_frame(race)
    } else if show == NOTHING {
        Ok(())
    } else {
        Err(BadInput("a loading bar function that is neither of the loading's").into())
    }
}
