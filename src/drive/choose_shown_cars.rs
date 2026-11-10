//! `drive::choose_shown_cars` (0x4560EA): which car blocks the two views show (`off_63DBB0`,
//! `off_63DBB4` = 0x70F3EC + n 0xFE0 for `dword_63DBA8`, `dword_63DBAC`), `dword_63DB98` (the
//! views' number, from `dword_63DB94`, by `dword_63DB88` when `dword_5C5CE0` is 2, also setting
//! `dword_6238A4`), the replay's car when `dword_52A910` is set (`dword_63DBA0`, then the second
//! view 0), and the eight of `dword_641680` .. -1 but the shown ones' 0 (the first, and the second
//! when there are two views). By the look of it the split screen's set-up (a reading).
//!
//! Callers (5): race init's loop (`drive::race_cars_step`), `race::step_replay_cars`,
//! `drive::replay_step`, `race::race_keys`,
//! `race::draw_race_scene`. Refuses (before anything) a replay car (`dword_63DBA0`) above 7 with
//! `dword_52A910` set (it would write past `dword_641680`'s eight).

use crate::errors::Refused;
use crate::net::owner::net;
use crate::race::owner::Race;
use crate::session::Dword;
use rc2k_core::drive::shown_cars::{ShownCars, ShownSetup};

fn get(v: &[&Dword], k: usize) -> u32 {
    v[k].bits()
}

fn put(v: &[&Dword], k: usize, x: u32) {
    v[k].set_bits(x);
}

/// On the variables at `v` (`game`'s order).
// Converted: `rc2k_core::drive::shown_cars::ShownCars::choose`.
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
fn game(race: &Race) -> Vec<&Dword> {
    let shared = &race.shared;
    let mut cells: Vec<&Dword> = vec![
        &shared.view.view_cars[0],
        &shared.view.view_cars[1],
        &shared.view.players,
        &shared.view.views,
        &race.course.numbers.replay_mode,
        &shared.view.view_switch,
        &shared.vars.split,
        &net().vars.session, // dword_52A910: in the network's variables
        &shared.view.local_car,
        &shared.view.view_blocks[0],
        &shared.view.view_blocks[1],
    ];
    cells.extend(race.frame.frame_work.listed_car_sounds.iter());
    cells
}

pub fn choose_shown_cars(race: &Race) -> Result<(), Refused> {
    choose_at(&game(race))
}
