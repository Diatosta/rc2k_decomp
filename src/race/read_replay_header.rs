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

use crate::errors::Refused;
use crate::framework::Regs;
use crate::physics::flow::a;
use crate::race::race_end::{HEADER, setting_place};
use crate::race::viewer::{Part, get, put, regs_entry};
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

/// The Rust version: the registers it leaves (the entry's).
pub fn read_replay_header(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let call = |part: Part, place: u32, eax: u32| -> Result<(), Refused> {
        let mut r = Regs { eax, ..*entry };
        part(place, &mut r)
    };
    // Converted: `rc2k_core::race::replay::{HEADER_SETTINGS, read_logged}`.
    call(part, 1, a(TITLE))?;
    let from = |s: Setting| HEADER + s.offset() as usize;
    for s in HEADER_SETTINGS {
        for to in targets(s) {
            put(to, get(from(s)));
        }
    }
    for (&text, &s) in NAMES.iter().zip(read_logged()) {
        call(part, 2, a(text))?;
        call(part, 3, get(from(s)))?;
    }
    Ok(*entry)
}

/// In the game: the logs' Rust versions.
pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    unsafe {
        match place {
            1 | 2 => crate::logfile::log_write::log_write(r.eax as *const u8, place == 1, 1),
            _ => crate::logfile::log_int::log_int(r.eax as i32, true, 1),
        }
    }
    Ok(())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    regs_entry(r, &|r| read_replay_header(r, &mut game_part))
}
