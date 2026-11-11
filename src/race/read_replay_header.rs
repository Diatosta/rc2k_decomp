//! `race::read_replay_header` (0x460428): "READREPLAYHEADER" logged; the replay's header
//! (`header_stage` ..) into the race's variables: the arcade's noise seed, start place and cars,
//! the stage, the game mode, the players, the cars and their kinds (into the car blocks too), the
//! two game-over flags (`targets`); then twelve of them logged, each as its name and its value
//! (`NAMES`).
//!
//! Callers: `race::viewer_load_stage`, 0x44401B (the replay run of `sub_443FxF`).

use crate::race::owner_shared::RaceShared;
use crate::race::race_end::{header_cell, setting_cell};
use crate::session::Dword;
use rc2k_core::race::replay::{HEADER_SETTINGS, Setting, read_logged};

/// The race's places a header setting goes back to, in the original's order (a kind
/// into its block and into `g_raceCarKinds`).
fn targets(shared: &RaceShared, s: Setting) -> Vec<&Dword> {
    match s {
        Setting::Kind(n) => vec![
            setting_cell(shared, s),
            &shared.vars.race_car_kinds[n as usize],
        ],
        _ => vec![setting_cell(shared, s)],
    }
}

/// "[replayheader.arcstartpos]" .. "[replayheader.gameoverflag2]", by `read_logged`.
const NAMES: [&std::ffi::CStr; 12] = [
    c"[replayheader.arcstartpos]",
    c"[replayheader.numarccars]",
    c"[replayheader.track]",
    c"[replayheader.gmode]",
    c"[replayheader.numpl]",
    c"[replayheader.numcars]",
    c"[replayheader.car1]",
    c"[replayheader.car2]",
    c"[replayheader.car3]",
    c"[replayheader.car4]",
    c"[replayheader.gameoverflag1]",
    c"[replayheader.gameoverflag2]",
];
const TITLE: &std::ffi::CStr = c"READREPLAYHEADER"; // "READREPLAYHEADER"

/// `log_write_l1` (with a line break for the title) of a text.
fn log_text(text: &std::ffi::CStr, newline: bool) {
    crate::logfile::log_write::log_nul(text.to_bytes(), newline, 1);
}

pub fn read_replay_header(shared: &RaceShared) {
    // Converted: `rc2k_core::race::replay::{HEADER_SETTINGS, read_logged}`.
    log_text(TITLE, true);
    for s in HEADER_SETTINGS {
        for to in targets(shared, s) {
            to.set(header_cell(s).get());
        }
    }
    for (&text, &s) in NAMES.iter().zip(read_logged()) {
        log_text(text, false);
        crate::logfile::log_int::log_int(header_cell(s).get() as i32, true, 1);
    }
}
