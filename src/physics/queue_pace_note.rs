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
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{Regs, game_addr, game_read, rva};
use rc2k_core::physics::pace_calls::{Code, code, find_group, text_place};

pub const LOG_WRITE: u32 = 0;
pub const LOG_INT: u32 = 1;
pub const LOG_INT_NL: u32 = 2;
pub const CUE: u32 = 3;
pub const QUEUE: u32 = 4;
pub const PACE: u32 = 5;
pub const EXPIRE: u32 = 6;
const GROUPS: usize = 0x615E14; // dword_615E14
const GROUP_CHECKPOINTS: usize = 0x619E18; // dword_619E18
const GROUP_TEXTS: usize = 0x61AE18; // dword_61AE18
const TEXT: usize = 0x61BE18; // byte_61BE18
/// "!!! invalid pace data", "data", "ecx", "pointer".
const WORDS: [usize; 4] = [0x4DBB70, 0x4DBB86, 0x4DBB8B, 0x4DBB8F];

fn g(x: usize) -> u32 {
    unsafe { game_read(rva(x)) }
}

/// The steps on `r`.
// Converted: `rc2k_core::physics::pace_calls::{find_group, code, text_place}`.
pub fn queue_pace_note_in(r: &mut Regs) -> Result<(), Refused> {
    let groups = g(GROUPS);
    let found = find_group(groups, r.edx, |n| g(GROUP_CHECKPOINTS + 4 * n as usize))?;
    if groups == 0 {
        return Ok(());
    }
    let Some(group) = found else {
        r.ecx = groups;
        return Ok(());
    };
    r.ecx = group;
    r.esi = g(GROUP_TEXTS + 4 * r.ecx as usize);
    loop {
        let byte = unsafe { game_read::<u8>(rva(TEXT) + text_place(r.esi)? as usize) };
        r.ebx = byte as u32;
        match code(byte) {
            Code::End => return Ok(()),
            Code::Bad => {
                let word = |n: usize| game_addr(rva(WORDS[n])) as u32;
                for (place, eax) in [
                    (LOG_WRITE, word(0)),
                    (LOG_WRITE, word(1)),
                    (LOG_INT, r.ebx),
                    (LOG_WRITE, word(2)),
                    (LOG_INT_NL, r.ecx),
                    (LOG_WRITE, word(3)),
                    (LOG_INT_NL, r.esi),
                ] {
                    let mut c = Regs { eax, ..*r };
                    crate::physics::queue_pace_note::pace_part(place, &mut c)?;
                }
                return Ok(());
            }
            Code::Call => {
                let mut c = *r;
                crate::physics::queue_pace_note::pace_part(CUE, &mut c)?;
                crate::physics::queue_pace_note::pace_part(QUEUE, r)?;
                r.esi = r.esi.wrapping_add(1);
            }
        }
    }
}

/// The calls in the game: the Rust versions.
pub fn pace_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    use crate::logfile::{log_int::log_int, log_write::log_write};
    unsafe {
        match place {
            LOG_WRITE => log_write(r.eax as *const u8, false, 1),
            LOG_INT | LOG_INT_NL => log_int(r.eax as i32, place == LOG_INT_NL, 1),
            CUE => crate::physics::cue_listed_car_sound_last::cue_listed_car_sound_last_in(r)?,
            QUEUE => crate::physics::queue_listed_event::queue_listed_event(r.ebx)?.apply(r),
            PACE => queue_pace_note_in(r)?,
            _ => crate::physics::expire_car_events::expire_car_events_at(
                &crate::physics::queue_car_event::QueuePlaces::game(),
            )?
            .apply(r),
        }
    }
    Ok(())
}

/// `dispatch_pace` without its hook (Rust callers; a refusal is the caller's).
pub fn pace_entry(r: &mut Regs, rust: fn(&mut Regs) -> Result<(), Refused>) -> Result<(), Refused> {
    let mut g = *r;
    rust(&mut g)?;
    *r = Regs {
        esp: r.esp,
        eflags: r.eflags,
        ..g
    };
    Ok(())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    pace_entry(r, queue_pace_note_in)
}

// ---- for the self-tests ----
