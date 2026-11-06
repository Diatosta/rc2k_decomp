//! `drive::choose_shown_cars` (0x4560EA): which car blocks the two views show
//! (`off_63DBB0`, `off_63DBB4` = 0x70F3EC + n 0xFE0 for `dword_63DBA8`, `dword_63DBAC`),
//! `dword_63DB98` (the views' number, from `dword_63DB94`, by `dword_63DB88` when
//! `dword_5C5CE0` is 2, also setting `dword_6238A4`), the replay's car when
//! `dword_52A910` is set (`dword_63DBA0`, then the second view 0), and the eight of
//! `dword_641680` .. -1 but the shown ones' 0 (the first, and the second when there are
//! two views). By the look of it the split screen's set-up (a reading).
//!
//! In:  nothing
//! Out: eax `dword_63DBA0` (with `dword_52A910` set) or the second view's block; the
//!      rest kept; no flags read (callers go on with `mov`, `lea`, calls)
//!
//! Callers (5): race init's loop (`sub_4608BB`), `sub_460835`, `sub_460B2F`, `sub_43A649`,
//! `sub_45429B`. Refuses (before anything) a replay car (`dword_63DBA0`) above 7 with
//! `dword_52A910` set (it would write past `dword_641680`'s eight).

use crate::errors::Refused;
use crate::race::vars_store::RACE_VARS;
use crate::race::view_store::RACE_VIEW;
use crate::session::Dword;
use rc2k_core::drive::shown_cars::{ShownCars, ShownSetup};

fn get(v: &[&Dword], k: usize) -> u32 {
    v[k].bits()
}

fn put(v: &[&Dword], k: usize, x: u32) {
    v[k].set_bits(x);
}

/// The Rust version on the variables at `v` (`game`'s order).
// Converted: `rc2k_core::drive::shown_cars::ShownCars::choose`.
/// The eax it leaves is read by no caller (liveness at its five call sites).
pub fn choose_at(v: &[&Dword]) -> Result<(), Refused> {
    let setup = ShownSetup {
        players: get(v, 2),
        screen_mode: get(v, 4),
        layout: get(v, 5),
        replay: get(v, 7),
        replay_car: get(v, 8),
    };
    let split = get(v, 6);
    let mut shown = ShownCars {
        split,
        ..ShownCars::default()
    };
    shown.choose(&setup, crate::race::blocks_store::block_addr(0))?;
    put(v, 0, shown.cars[0]);
    put(v, 1, shown.cars[1]);
    put(v, 3, shown.views);
    if shown.split != split {
        put(v, 6, shown.split);
    }
    // The views' blocks kept by handle (`blocks_store::block_ref`).
    put(v, 9, crate::race::blocks_store::handle_of(shown.blocks[0]));
    put(v, 10, crate::race::blocks_store::handle_of(shown.blocks[1]));
    for (k, &x) in shown.listed.iter().enumerate() {
        put(v, 11 + k, x);
    }
    Ok(())
}

/// The variables' addresses, in the order the core uses them.
fn game() -> Vec<&'static Dword> {
    let mut cells: Vec<&'static Dword> = vec![
        &RACE_VIEW.view_cars[0],
        &RACE_VIEW.view_cars[1],
        &RACE_VIEW.players,
        &RACE_VIEW.views,
        &crate::race::owner::race().course.numbers.replay_mode,
        &RACE_VIEW.view_switch,
        &RACE_VARS.split,
        &crate::net::store::NET.session, // dword_52A910: in the network's variables
        &RACE_VIEW.local_car,
        &RACE_VIEW.view_blocks[0],
        &RACE_VIEW.view_blocks[1],
    ];
    cells.extend(
        crate::race::owner::race()
            .frame
            .frame_work
            .listed_car_sounds
            .iter(),
    );
    cells
}

/// The Rust version.
pub fn choose_shown_cars() -> Result<(), Refused> {
    choose_at(&game())
}
