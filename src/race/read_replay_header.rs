//! `race::read_replay_header`: logs "READREPLAYHEADER", then copies the replay's header (from
//! `header_stage` on) into the race's variables: the arcade's noise seed, start place and cars, the
//! stage, the game mode, the players, the cars and their kinds (into the car blocks too), the two
//! game-over flags (`targets`); then twelve of them are logged, each as its name and its value
//! (`NAMES`).

use crate::race::owner::Race;
use crate::race::race_end::{header_word, setting_set};
use rc2k_core::race::replay::{HEADER_SETTINGS, Setting, read_logged};
use rc2k_core::race::replay_v2::MAX_CARS;

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
const TITLE: &std::ffi::CStr = c"READREPLAYHEADER";

/// Logs a text, with a line break for the title.
fn log_text(text: &std::ffi::CStr, newline: bool) {
    crate::logfile::log_write::log_nul(text.to_bytes(), newline, 1);
}

pub(super) fn read_replay_header(race: &Race) {
    // Logic in `rc2k_core::race::replay::{HEADER_SETTINGS, read_logged}`.
    log_text(TITLE, true);
    for s in HEADER_SETTINGS {
        let word = header_word(race, s);
        setting_set(race, s, word);
        // A kind goes into its block and into `race_car_kinds`.
        if let Setting::Kind(n) = s {
            race.shared.vars.get().race_car_kinds[n as usize].set(word);
        }
    }
    // The header names four cars' models; a replay of the own format names every car's in its
    // extension, and the cars after the fourth are built from those.
    if let Some(ext) = race.replay.v2().lock().ext.as_ref() {
        for (n, car) in ext.cars.iter().enumerate().take(MAX_CARS) {
            race.shared.vars.get().race_car_kinds[n].set(car.kind);
            race.car_blocks_mut().cars[n].car_kind = car.kind;
        }
    }
    for (&text, &s) in NAMES.iter().zip(read_logged()) {
        log_text(text, false);
        crate::logfile::log_int::log_int(header_word(race, s) as i32, true, 1);
    }
}
