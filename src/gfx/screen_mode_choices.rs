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

use crate::errors::{BadInput, Refused};
use crate::framework::{
    CARRY_FLAG, RawHook, Regs, SelfTest, ZERO_FLAG, call_game, game_addr, hook_asm, rva,
};
use crate::race::viewer::{
    Part, Saved, answer, dispatch_parts, dword, get, put, region, run_both, test_part,
};
use crate::system::find_entry_with_bits::find_entry_with_bits;
use crate::system::next_entry::next_entry;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("gfx::screen_mode_choices", rva(0x4045D6), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data (IDA addresses) ----

const RECORD: usize = 0x4F50C0; // unk_4F50C0: the choices
const LIST: usize = RECORD + 0x28;
const MOST: u32 = 16; // the list's room, up to 0x4F51F0
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

/// The calls by place, as IDA shows them.
const CALLEES: [usize; 3] = [
    0x404AED, // 0 gfx::try_screen_mode
    0x404805, // 1 gfx::keep_race_screen_mode
    0x404823, // 2 gfx::set_menu_screen_mode
];

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

/// The Rust version: the registers and flags it leaves.
pub fn screen_mode_choices(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut out = *entry;
    let failed = |mut out: Regs| {
        out.set_carry(true);
        Ok(out)
    };
    let made = field(0) == 1;
    // The way taken and what decides it, before anything is written.
    let kind = if made {
        field(4)
    } else {
        ((get(RACE_MODE) & 0xF000) != get(KIND_BITS)) as u32
    };
    let rebuilding = !made || entry.eax == 0;
    let modes = if rebuilding { modes_of(kind) } else { None };
    if let Some(modes) = &modes
        && modes.len() as u32 > MOST
    {
        return Err(BadInput("more screen modes than the choices' list holds").into());
    }
    if made {
        // ebp: the list's mode at the place chosen (read before eax is looked at).
        if field(8) >= 0x1000 {
            return Err(BadInput("a size's place far past the choices' list").into());
        }
        let current = read(game_addr(rva(LIST)) as u32 + 16 * field(8) + 0xC);
        if entry.eax != 0 {
            // The choice tried.
            let (third, place) = (field(0xC), field(8));
            if kind > 1 || place >= field(0x24) || third > 1 {
                return failed(out);
            }
            let mode = 0x30_0000
                | get(KIND_BITS + 4 * kind as usize)
                | read(game_addr(rva(LIST)) as u32 + 16 * place + 0xC)
                | get(THIRD_BITS + 4 * third as usize);
            let mut r = Regs {
                eax: mode,
                ebx: third,
                esi: game_addr(rva(RECORD)) as u32,
                ebp: current,
                ..*entry
            };
            part(0, &mut r)?;
            let mut worked = !r.carry();
            if worked {
                r.ebx = field(0x10);
                worked = r.ebx <= get(TEXTURE_FIGURE);
            }
            if worked {
                part(1, &mut r)?;
            }
            part(2, &mut r)?;
            if !worked {
                return failed(out);
            }
            out.set_carry(false);
            out.set_zero(r.eax == 0);
            return Ok(out);
        }
        return rebuild(entry, current, modes);
    }
    set(4, kind);
    set(0x14, get(PRIMARY_CARD));
    set(0x18, get(SECONDARY_CARD));
    rebuild(entry, get(RACE_MODE), modes)
}

/// The list made again (`loc_40468A` on), `current` the mode kept in ebp.
fn rebuild(entry: &Regs, current: u32, modes: Option<Vec<u32>>) -> Result<Regs, Refused> {
    let mut out = *entry;
    set(0x1C, 0);
    set(0x24, 0);
    let Some(modes) = modes else {
        out.set_carry(true);
        return Ok(out);
    };
    for (n, &at) in modes.iter().enumerate() {
        let mode = read(at);
        let size = entry_or_past(mode & 0xFF, SIZES, 8);
        let kind = entry_or_past(mode & 0xF00, PIXEL_KINDS, 2);
        let slot = LIST + 16 * n;
        put(slot, read_word(size + 4));
        put(slot + 4, read_word(size + 6));
        put(slot + 8, read(kind + 4));
        put(slot + 0xC, mode);
        set(0x24, field(0x24) + 1);
    }
    set(0x20, get(TEXTURE_FIGURE));
    set(0x1C, 1);
    let count = field(0x24);
    let list = |n: u32| read(game_addr(rva(LIST)) as u32 + 16 * n + 0xC);
    // The current mode's place (its low 12 bits); else the first place (the original's
    // search by width keeps 0: its bound, ecx, stays 0).
    let place = (0..count)
        .find(|&n| (list(n) ^ current) & 0xFFF == 0)
        .unwrap_or(0);
    set(8, place);
    let race = get(RACE_MODE);
    match (0..count).find(|&n| list(n) & 0xFFF == race & 0xFFF) {
        Some(n) => {
            set(8, n);
            let third = entry_or_past(race & 0xF_0000, THIRD_TABLE, 3);
            set(0xC, (read(third + 4) as i32).wrapping_sub(1).max(0) as u32);
        }
        None => {
            set(8, 0);
            set(0xC, 0);
        }
    }
    let figure = field(0x10) & field(0x20);
    set(0x10, figure);
    set(0, 1);
    out.set_carry(false);
    out.set_zero(figure == 0);
    Ok(out)
}

/// In both programs: the trial and the race's mode by their Rust versions, the menus' mode
/// through its hook.
pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    use crate::gfx::{keep_race_screen_mode as keep, try_screen_mode as trial};
    match place {
        0 => *r = trial::try_screen_mode(r, &mut trial::game_part)?,
        1 => *r = keep::keep_race_screen_mode(r, &mut keep::game_part)?,
        _ => unsafe { call_game(r, game_addr(rva(CALLEES[2]))) },
    }
    Ok(())
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_parts(&HOOK, r, call_original, &|r| {
        screen_mode_choices(r, &mut game_part)
    });
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let places: &'static [(usize, u32)] = &[(0x404AED, 0), (0x404805, 1), (0x404823, 2)];
    // pusha; lea esi, unk_4F50C0
    let copy = match crate::race::viewer::copy(
        0x4045D6,
        0x22F,
        &[0x60, 0x8D, 0x35, 0xC0, 0x50, 0x4F, 0x00],
        &[
            0x5A, 0x6C, 0x71, 0x7B, 0xE1, 0x105, 0x12A, 0x142, 0x149, 0x18C, 0x203,
        ],
        places,
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let regions = [
        region(RECORD, 0x130),
        region(MODE_TABLE, 0x40),
        dword(MODE_COUNT),
        dword(RACE_MODE),
        dword(TEXTURE_FIGURE),
        dword(PRIMARY_CARD),
        dword(SECONDARY_CARD),
    ];
    let saved = Saved::new(&regions);
    // Eight modes of the test's: sizes 1 .. 0x80, pixel kinds 0x100 / 0x800, bits of both
    // kinds (0x1000, 0x2000), thirds 0x10000 .. 0x40000.
    let table: [u32; 8] = [
        0x31_1101, 0x32_1102, 0x31_2802, 0x32_1104, 0x34_2808, 0x31_1110, 0x34_2820, 0x32_1180,
    ];
    let mut case = 0u32;
    for made in [false, true] {
        for eax in [0u32, 1] {
            for race in [0x32_1102u32, 0x34_2808, 0x31_1140] {
                for works in [false, true] {
                    for figure in [0u32, 7] {
                        case += 1;
                        run_both(
                            &mut t,
                            &format!(
                                "made {made}, eax {eax}, race mode {race:#x}, the trial works {works}, figure {figure}"
                            ),
                            copy.addr(),
                            Regs {
                                eax,
                                ..Regs::SENTINEL
                            },
                            &regions,
                            CARRY_FLAG | ZERO_FLAG,
                            &|| {
                                for n in 0..0x130 / 4 {
                                    put(RECORD + 4 * n, 0xAAAA_AAAA);
                                }
                                for (n, &m) in table.iter().enumerate() {
                                    put(MODE_TABLE + 8 * n, m);
                                    put(MODE_TABLE + 8 * n + 4, 0);
                                }
                                put(MODE_COUNT, 8);
                                put(RACE_MODE, race);
                                put(TEXTURE_FIGURE, 5);
                                put(PRIMARY_CARD, 0x11);
                                put(SECONDARY_CARD, 0x22);
                                if made {
                                    set(0, 1);
                                    set(4, case & 1);
                                    set(8, case % 3);
                                    set(0xC, case % 2);
                                    set(0x10, figure);
                                    set(0x24, 3);
                                    for (n, &m) in table.iter().take(3).enumerate() {
                                        put(LIST + 16 * n + 0xC, m);
                                    }
                                }
                                answer(Box::new(move |place, r| {
                                    if place == 0 {
                                        r.set_carry(!works);
                                    }
                                    r.eax = 0x10 * place;
                                }));
                            },
                            &|r| screen_mode_choices(r, &mut test_part),
                        );
                    }
                }
            }
        }
    }
    crate::race::viewer::quiet();
    drop(saved);
    t.finish()
}
