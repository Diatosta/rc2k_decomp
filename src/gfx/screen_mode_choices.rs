//! `gfx::screen_mode_choices` (0x4045D6): the graphics options' screen mode choices, a
//! record at `unk_4F50C0` (a reading of its fields: +0 made, +4 the pixel kind's choice 0
//! or 1, +8 the size's (a place in the list), +0xC the third choice 0 or 1, +0x10 a
//! texture figure, +0x14 / +0x18 the two 3D cards, +0x1C the list made, +0x20
//! `dword_5AB9A4` then, +0x24 the list's length, from +0x28 the list: 16 bytes a mode,
//! its width, height, pixel kind's figure and the mode).
//!
//! - Made and eax not 0: the mode chosen (0x300000 with `dword_4F4E30[+4]`, the list's
//!   mode at +8 and `dword_4F4E38[+0xC]`) tried (`gfx::try_screen_mode`); if it works and
//!   +0x10 is not above `dword_5AB9A4` it is kept (`gfx::keep_race_screen_mode`); either
//!   way the menus' mode set again (`gfx::set_menu_screen_mode`); carry if it did not work.
//! - Not made: +4 from `g_raceScreenMode`'s pixel kind (0x1000 or not), the cards, then
//!   as made with eax 0, from `g_raceScreenMode`.
//! - Made and eax 0 (or not made): the list made again from the mode table's modes of the
//!   pixel kind (`system::find_entry_with_bits`, `system::next_entry`), each with its size
//!   (`g_screenSizes`) and pixel kind's figure (`g_pixelKinds`) by `system::find_entry`
//!   (a key not there: the place past the table, as the original reads it); +8 the mode's
//!   place in the list by its low 12 bits (else the first whose width is not below the
//!   mode's: the original's search keeps 0, its bound never moves); then +8 and +0xC from
//!   `g_raceScreenMode` (its low 12 bits in the list, else both 0; +0xC from
//!   `dword_4F4E90` by its fourth nibble, less 1, not below 0); +0x10 masked by +0x20;
//!   made.
//!
//! In:  eax = 0 to make the list again, else to try the choice
//! Out: carry = refused by the original's own checks or the mode does not work; else carry
//!      clear and the zero flag by the eax left; every register kept (`pusha` .. `popa`)
//!
//! Callers: `gfx::choose_screen_modes` (0x4045CA) and the graphics options' actions
//! (`sub_470017`, `sub_4703C2`). Refuses (before anything is written) a list longer than
//! the record's room (16 modes: the original would write over 0x4F51F0's constants) and a
//! size's place of 0x1000 or more when made (the original reads that far past the list).
//! Its three calls through `part` (0 the trial, 1 the race's mode kept, 2 the menus' mode
//! set; in both programs the first two Rust versions, the third through its hook, the
//! platform's in the standalone). Shadow mode runs the original alone (the screen
//! changes). Self-test: a copy whose three calls go to stand-ins, against the Rust
//! version, on a mode table of the test's: made or not, eax 0 or not, the trial working
//! or not, the race's mode in the list or not.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, Refused};
use crate::framework::{Flags, game_addr, rva};
use crate::race::viewer::{get, put};
use crate::system::find_entry_with_bits::find_entry_with_bits;
use crate::system::next_entry::next_entry;
use rc2k_core::gfx::mode_choices as mc;

// ---- game data (IDA addresses) ----

const RECORD: usize = 0x4F50C0; // unk_4F50C0: the choices
const LIST: usize = RECORD + 0x28;
const KIND_BITS: usize = 0x4F4E30; // dword_4F4E30: two pixel kinds' bits
const THIRD_BITS: usize = 0x4F4E38; // dword_4F4E38: two
const THIRD_TABLE: usize = 0x4F4E90; // dword_4F4E90: three entries
const SIZES: usize = 0x4F4E50; // g_screenSizes: eight entries
const PIXEL_KINDS: usize = 0x4F4E40; // g_pixelKinds: two entries
const TEXTURE_FIGURE: usize = 0x5AB9A4; // dword_5AB9A4
const PRIMARY_CARD: usize = 0x5AB4AC; // g_primary3dCard
const SECONDARY_CARD: usize = 0x5AB4B0; // g_secondary3dCard
use crate::gfx::keep_race_screen_mode::RACE_MODE;
use crate::gfx::try_screen_mode::{MODE_COUNT, MODE_TABLE};

fn field(offset: usize) -> u32 {
    get(RECORD + offset)
}

fn set(offset: usize, v: u32) {
    put(RECORD + offset, v)
}

/// `system::find_entry` as the original uses it: the entry whose first dword is `key`, or
/// the place past the table's `count` entries.
fn entry_or_past(key: u32, table: usize, count: u32) -> u32 {
    let at = game_addr(rva(table)) as u32;
    (0..count)
        .map(|n| at + 8 * n)
        .find(|&e| unsafe { (e as *const u32).read_unaligned() } == key)
        .unwrap_or(at + 8 * count)
}

fn read(at: u32) -> u32 {
    unsafe { (at as *const u32).read_unaligned() }
}

fn read_word(at: u32) -> u32 {
    unsafe { (at as *const u16).read_unaligned() as u32 }
}

/// The modes the list would hold (the mode table's of the pixel kind's bits), or None
/// when the original stops (the kind's choice past 1, no mode of the kind).
fn modes_of(kind: u32) -> Option<Vec<u32>> {
    if kind > 1 {
        return None;
    }
    let bits = get(KIND_BITS + 4 * kind as usize);
    let (mut found, mut at, mut left) =
        find_entry_with_bits(bits, game_addr(rva(MODE_TABLE)) as u32, get(MODE_COUNT));
    if !found {
        return None;
    }
    let mut modes = Vec::new();
    while found {
        modes.push(at);
        let (carry, next, rest) = next_entry(at, left);
        if carry {
            break;
        }
        (found, at, left) = find_entry_with_bits(bits, next, rest);
    }
    Some(modes)
}

/// The Rust version: with `try_choice` (eax not 0) the choice tried, else the list made
/// again; the flags it leaves (see the header).
pub fn screen_mode_choices(try_choice: bool) -> Result<Flags, Refused> {
    // Converted: `rc2k_core::gfx::mode_choices`.
    let failed = || {
        Ok(Flags {
            carry: true,
            zero: false,
        })
    };
    let made = field(0) == 1;
    // The way taken and what decides it, before anything is written.
    let kind = if made {
        field(4)
    } else {
        mc::kind_of_race(get(RACE_MODE), get(KIND_BITS))
    };
    let rebuilding = !made || !try_choice;
    let modes = if rebuilding { modes_of(kind) } else { None };
    if let Some(modes) = &modes
        && modes.len() as u32 > mc::ROOM
    {
        return Err(BadInput("more screen modes than the choices' list holds").into());
    }
    if made {
        // ebp: the list's mode at the place chosen (read before eax is looked at).
        if field(8) >= mc::PLACE_LIMIT {
            return Err(BadInput("a size's place far past the choices' list").into());
        }
        let current = read(game_addr(rva(LIST)) as u32 + 16 * field(8) + 0xC);
        if try_choice {
            // The choice tried.
            let (third, place) = (field(0xC), field(8));
            if !mc::can_try(kind, place, field(0x24), third) {
                return failed();
            }
            let mode = mc::tried_mode(
                get(KIND_BITS + 4 * kind as usize),
                read(game_addr(rva(LIST)) as u32 + 16 * place + 0xC),
                get(THIRD_BITS + 4 * third as usize),
            );
            let tried = crate::gfx::try_screen_mode::try_screen_mode(mode)?;
            let worked = !tried.carry && mc::kept(field(0x10), get(TEXTURE_FIGURE));
            if worked {
                crate::gfx::keep_race_screen_mode::keep_race_screen_mode(mode)?;
            }
            crate::platform::set_menu_mode_now();
            if !worked {
                return failed();
            }
            // The eax the three calls leave: the mode (they keep it).
            return Ok(Flags {
                carry: false,
                zero: mode == 0,
            });
        }
        return rebuild(current, modes);
    }
    set(4, kind);
    set(0x14, get(PRIMARY_CARD));
    set(0x18, get(SECONDARY_CARD));
    rebuild(get(RACE_MODE), modes)
}

/// The list made again (`loc_40468A` on), `current` the mode kept in ebp.
fn rebuild(current: u32, modes: Option<Vec<u32>>) -> Result<Flags, Refused> {
    set(0x1C, 0);
    set(0x24, 0);
    let Some(modes) = modes else {
        return Ok(Flags {
            carry: true,
            zero: false,
        });
    };
    let mut listed = Vec::new();
    for (n, &at) in modes.iter().enumerate() {
        let mode = read(at);
        let (size_key, kind_key) = mc::keys(mode);
        let size = entry_or_past(size_key, SIZES, 8);
        let kind = entry_or_past(kind_key, PIXEL_KINDS, 2);
        let l = mc::Listed {
            width: read_word(size + 4),
            height: read_word(size + 6),
            figure: read(kind + 4),
            mode,
        };
        let slot = LIST + 16 * n;
        put(slot, l.width);
        put(slot + 4, l.height);
        put(slot + 8, l.figure);
        put(slot + 0xC, l.mode);
        set(0x24, field(0x24) + 1);
        listed.push(l.mode);
    }
    set(0x20, get(TEXTURE_FIGURE));
    set(0x1C, 1);
    set(8, mc::in_use_place(&listed, current));
    let race = get(RACE_MODE);
    let (place, third) = mc::race_place(&listed, race, || {
        read(entry_or_past(mc::third_key(race), THIRD_TABLE, 3) + 4)
    });
    set(8, place);
    set(0xC, third);
    let figure = mc::figure(field(0x10), field(0x20));
    set(0x10, figure);
    set(0, 1);
    Ok(Flags {
        carry: false,
        zero: figure == 0,
    })
}
