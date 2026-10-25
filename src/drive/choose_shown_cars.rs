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
use crate::race::frame_store::FRAME_WORK;
use crate::race::numbers_store::NUMBERS;
use crate::race::vars_store::RACE_VARS;
use crate::race::view_store::RACE_VIEW;
use rc2k_core::drive::shown_cars::{ShownCars, ShownSetup};

unsafe fn get(v: &[u32], k: usize) -> u32 {
    unsafe { (v[k] as *const u32).read_unaligned() }
}

unsafe fn put(v: &[u32], k: usize, x: u32) {
    unsafe { (v[k] as *mut u32).write_unaligned(x) }
}

/// The Rust version on the variables at `v` (`game`'s order).
// Converted: `rc2k_core::drive::shown_cars::ShownCars::choose`.
/// The eax it leaves is read by no caller (liveness at its five call sites).
pub unsafe fn choose_at(v: &[u32]) -> Result<(), Refused> {
    unsafe {
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
        shown.choose(&setup, crate::race::blocks_store::block_addr(0) as u32)?;
        put(v, 0, shown.cars[0]);
        put(v, 1, shown.cars[1]);
        put(v, 3, shown.views);
        if shown.split != split {
            put(v, 6, shown.split);
        }
        put(v, 9, shown.blocks[0]);
        put(v, 10, shown.blocks[1]);
        for (k, &x) in shown.listed.iter().enumerate() {
            put(v, 11 + k, x);
        }
        Ok(())
    }
}

/// The variables' addresses, in the order the core uses them.
fn game() -> Vec<u32> {
    let mut cells = vec![
        RACE_VIEW.view_cars[0].addr(),
        RACE_VIEW.view_cars[1].addr(),
        RACE_VIEW.players.addr(),
        RACE_VIEW.views.addr(),
        NUMBERS.replay_mode.addr(),
        RACE_VIEW.view_switch.addr(),
        RACE_VARS.split.addr(),
        crate::net::store::NET.session.addr(), // dword_52A910: in the network's variables
        RACE_VIEW.local_car.addr(),
        RACE_VIEW.view_blocks[0].addr(),
        RACE_VIEW.view_blocks[1].addr(),
    ];
    cells.extend(FRAME_WORK.listed_car_sounds.iter().map(|c| c.addr()));
    cells.into_iter().map(|a| a as u32).collect()
}

/// The Rust version.
pub unsafe fn choose_shown_cars() -> Result<(), Refused> {
    unsafe { choose_at(&game()) }
}
