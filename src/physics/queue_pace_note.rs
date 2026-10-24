//! `physics::queue_pace_note` (0x4418B3): the pace notes group called at checkpoint edx
//! (see `race/pace_notes.rs`): with groups (`dword_615E14` not 0), the first group whose
//! checkpoint (`dword_619E18`) is edx; its text (`byte_61BE18` from `dword_61AE18[group]`)
//! code by code up to a NUL: each below 0x54 cued (`physics::cue_listed_car_sound_last`,
//! inside `pusha` / `popa`) and queued as a car event (`physics::queue_listed_event`,
//! ebx the code); one of 0x54 or more ends it with "!!! invalid pace data", "data", the
//! code, "ecx", the group, "pointer", its place logged (level 1, each in `pusha` /
//! `popa`).
//!
//! In:  edx the checkpoint
//! Out: no groups: every register kept; none at edx: ecx the groups' count; else ecx the
//!      group, esi the text's place where it stopped, ebx that code (0 or 0x54 ..); the
//!      rest kept; no flags read (the caller's `pusha` / `popa`)
//!
//! Caller: `physics::pace_notes_step` (inside `pusha` / `popa`). Its calls by place (this
//! file's `pace_part`): 0 `logfile::log_write_l1`, 1 `logfile::log_int_l1`, 2
//! `logfile::log_int_l1_nl`, 3 `physics::cue_listed_car_sound_last`, 4
//! `physics::queue_listed_event`, 5 this function, 6 `physics::expire_car_events`.
//! Refuses (before anything) more than 0x400 groups (the scan reads past `dword_619E18`)
//! and a text place past the 0x4000 bytes (read past the text; checked at each code).
//! Shadow mode runs the original alone (sound cues and car events must not be queued
//! twice). Self-test: a copy whose calls go to stand-ins that note them, against the Rust
//! version with the same, on groups of the test's in the game's tables (put back).
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::errors::Refused;
use crate::framework::{game_addr, game_read, rva};
use rc2k_core::physics::pace_calls::{Code, code, find_group, text_place};

const GROUPS: usize = 0x615E14; // dword_615E14
const GROUP_CHECKPOINTS: usize = 0x619E18; // dword_619E18
const GROUP_TEXTS: usize = 0x61AE18; // dword_61AE18
const TEXT: usize = 0x61BE18; // byte_61BE18
/// "!!! invalid pace data", "data", "ecx", "pointer".
const WORDS: [usize; 4] = [0x4DBB70, 0x4DBB86, 0x4DBB8B, 0x4DBB8F];

fn g(x: usize) -> u32 {
    unsafe { game_read(rva(x)) }
}

/// The Rust version: the pace notes of checkpoint `checkpoint` (edx), each cued
/// (`physics::cue_listed_car_sound_last`) and queued (`physics::queue_listed_event`). The
/// registers it leaves are read by none of its callers.
// Converted: `rc2k_core::physics::pace_calls::{find_group, code, text_place}`.
pub fn queue_pace_note(checkpoint: u32) -> Result<(), Refused> {
    use crate::logfile::{log_int::log_int, log_write::log_write};
    let groups = g(GROUPS);
    let found = find_group(
        groups,
        checkpoint,
        |n| g(GROUP_CHECKPOINTS + 4 * n as usize),
    )?;
    if groups == 0 {
        return Ok(());
    }
    let Some(group) = found else {
        return Ok(());
    };
    let mut at = g(GROUP_TEXTS + 4 * group as usize);
    loop {
        let byte = unsafe { game_read::<u8>(rva(TEXT) + text_place(at)? as usize) } as u32;
        match code(byte as u8) {
            Code::End => return Ok(()),
            Code::Bad => {
                let word = |n: usize| game_addr(rva(WORDS[n])) as *const u8;
                unsafe {
                    log_write(word(0), false, 1);
                    log_write(word(1), false, 1);
                    log_int(byte as i32, false, 1);
                    log_write(word(2), false, 1);
                    log_int(group as i32, true, 1);
                    log_write(word(3), false, 1);
                    log_int(at as i32, true, 1);
                }
                return Ok(());
            }
            Code::Call => {
                crate::physics::cue_listed_car_sound_last::cue_listed_car_sound_last(byte)?;
                unsafe { crate::physics::queue_listed_event::queue_listed_event(byte) }?;
                at = at.wrapping_add(1);
            }
        }
    }
}
