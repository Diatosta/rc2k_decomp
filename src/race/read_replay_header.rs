//! `race::read_replay_header` (0x460428): "READREPLAYHEADER" logged; the replay's header
//! (`dword_719B14` ..) into the race's variables: the arcade's noise seed, start place
//! and cars, the stage, the game mode, the players, the cars and their kinds (into the
//! car blocks too), the two game-over flags (`targets`); then twelve of them logged, each
//! as its name and its value (`NAMES`).
//!
//! In:  nothing
//! Out: every register kept (the logs in `pusha` / `popa`, the copies by `push` / `pop`)
//!
//! Callers: `race::viewer_load_stage`, 0x44401B (the replay run of `sub_443FxF`). Its log
//! calls through `part`: 1 `logfile::log_write_l1_nl`, 2 `logfile::log_write_l1`, 3
//! `logfile::log_int_l1_nl` (eax the text or value); in the game their Rust versions.
//! Shadow mode runs the original alone (it logs). Self-test: a copy whose calls go to stand-ins, against the Rust version; the header filled
//! by the test, what it writes saved and put back.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::physics::flow::a;
use crate::race::race_end::{HEADER, setting_place};
use crate::race::viewer::{get, put};
use rc2k_core::race::replay::{HEADER_SETTINGS, Setting, read_logged};

/// The race's places a header setting goes back to, in the original's order (a kind
/// into its block and into `g_raceCarKinds`).
fn targets(s: Setting) -> Vec<usize> {
    match s {
        Setting::Kind(n) => vec![setting_place(s), 0x623858 + 4 * n as usize],
        _ => vec![setting_place(s)],
    }
}

/// "[replayheader.arcstartpos]" .. "[replayheader.gameoverflag2]", by `read_logged`.
const NAMES: [usize; 12] = [
    0x4DDDFB, 0x4DDE16, 0x4DDE30, 0x4DDE45, 0x4DDE5A, 0x4DDE6F, 0x4DDE86, 0x4DDE9A, 0x4DDEAE,
    0x4DDEC2, 0x4DDED6, 0x4DDEF3,
];
const TITLE: usize = 0x4DDDEA; // "READREPLAYHEADER"

/// `log_write_l1` (with a line break for the title) of a text.
fn log_text(text: u32, newline: bool) {
    unsafe { crate::logfile::log_write::log_write(text as *const u8, newline, 1) }
}

/// The Rust version. Reads no register and keeps every one (liveness), so its only
/// caller, `race::viewer_load_stage`, calls it plainly.
pub fn read_replay_header() {
    // Converted: `rc2k_core::race::replay::{HEADER_SETTINGS, read_logged}`.
    log_text(a(TITLE), true);
    let from = |s: Setting| HEADER + s.offset() as usize;
    for s in HEADER_SETTINGS {
        for to in targets(s) {
            put(to, get(from(s)));
        }
    }
    for (&text, &s) in NAMES.iter().zip(read_logged()) {
        log_text(a(text), false);
        crate::logfile::log_int::log_int(get(from(s)) as i32, true, 1);
    }
}
