//! `gfx::screen_mode_choices` (0x4045D6): the graphics options' screen mode choices, the
//! record at `unk_4F50C0` (`gfx::choices_store`: made, the video card (0 primary, 1
//! secondary), the resolution (a place in the list), the buffering (0 double, 1 triple),
//! mipmapping, the two cards present, the list made, the mipmapping's limit, the list's
//! length, the list: 16 bytes a resolution, its width, height, pixel kind's figure and
//! the mode). The card is the mode's bit `MODES.kind_bits` (0x1000: the secondary).
//!
//! - Made and eax not 0: the mode chosen (0x300000 with `dword_4F4E30[+4]`, the list's
//!   mode at +8 and `dword_4F4E38[+0xC]`) tried (`gfx::try_screen_mode`); if it works and
//!   +0x10 is not above `dword_5AB9A4` it is kept (`gfx::keep_race_screen_mode`); either
//!   way the menus' mode set again (`gfx::set_menu_screen_mode`); carry if it did not work.
//! - Not made: the card from `g_raceScreenMode` (0x1000 or not), the cards present, then
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
//! (`sub_470017`, `sub_4703C2`). Refuses (before anything is written) a list longer than the
//! record's room (16 modes: the original would write over 0x4F51F0's constants) and a size's
//! place of 0x1000 or more when made (the original reads that far past the list).

use crate::errors::{BadInput, Refused};
use crate::framework::Flags;
use crate::gfx::cards::CARDS;
use crate::gfx::modes::{MODES, Modes};
use crate::pointers::GamePtr;
use crate::system::find_entry_with_bits::find_entry_with_bits;
use rc2k_core::gfx::mode_choices as mc;

// ---- game data (IDA addresses) ----

use crate::gfx::choices_store::SCREEN_CHOICES as CHOICES;
use crate::gfx::screen_state::SCREEN_MODE;

/// `system::find_entry` as the original uses it: the entry whose first dword is `key`, or
/// the place past the table's `count` entries.
fn entry_or_past(key: u32, table: GamePtr, count: u32) -> Result<GamePtr, BadInput> {
    for n in 0..count {
        if read(table.offset(8 * n))? == key {
            return Ok(table.offset(8 * n));
        }
    }
    Ok(table.offset(8 * count))
}

fn read(at: GamePtr) -> Result<u32, BadInput> {
    at.read("a screen mode table's dword")
}

/// The list's mode at `place`, which may lie past the list's end (below `PLACE_LIMIT`).
fn list_mode(place: u32) -> Result<u32, BadInput> {
    GamePtr::to(&CHOICES.list)
        .offset(16 * place + 0xC)
        .read("a screen mode table's dword")
}

fn read_word(at: GamePtr) -> Result<u32, BadInput> {
    Ok(at.read::<u16>("a screen mode table's word")? as u32)
}

/// The modes the list would hold (the mode table's of the pixel kind's bits), or None
/// when the original stops (the kind's choice past 1, no mode of the kind).
fn modes_of(kind: u32) -> Option<Vec<GamePtr>> {
    if kind > 1 {
        return None;
    }
    let bits = MODES.kind_bits[kind as usize].get();
    let (mut found, mut at, mut left) =
        find_entry_with_bits(bits, Modes::mode_table_ptr(), MODES.count.get());
    if !found {
        return None;
    }
    let mut modes = Vec::new();
    while found {
        modes.push(at);
        // `system::next_entry`: on by one entry, the carry set when none are left.
        if left <= 1 {
            break;
        }
        (found, at, left) = find_entry_with_bits(bits, at.offset(8), left - 1);
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
    let made = CHOICES.made.get() == 1;
    // The way taken and what decides it, before anything is written.
    let kind = if made {
        CHOICES.card.get()
    } else {
        mc::kind_of_race(SCREEN_MODE.race_mode.get(), MODES.kind_bits[0].get())
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
        if CHOICES.resolution.get() >= mc::PLACE_LIMIT {
            return Err(BadInput("a size's place far past the choices' list").into());
        }
        let current = list_mode(CHOICES.resolution.get())?;
        if try_choice {
            // The choice tried.
            let (third, place) = (CHOICES.buffering.get(), CHOICES.resolution.get());
            if !mc::can_try(kind, place, CHOICES.resolutions.get(), third) {
                return failed();
            }
            let mode = mc::tried_mode(
                MODES.kind_bits[kind as usize].get(),
                list_mode(place)?,
                MODES.third_bits[third as usize].get(),
            );
            let tried = crate::gfx::try_screen_mode::try_screen_mode(mode)?;
            let worked = !tried.carry
                && mc::kept(
                    CHOICES.mipmapping.get(),
                    crate::d3d::cards_store::CARDS_3D.mips.get(),
                );
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
    CHOICES.card.set(kind);
    CHOICES.has_primary.set(CARDS.primary.get());
    CHOICES.has_secondary.set(CARDS.secondary.get());
    rebuild(SCREEN_MODE.race_mode.get(), modes)
}

/// The list made again (`loc_40468A` on), `current` the mode kept in ebp.
fn rebuild(current: u32, modes: Option<Vec<GamePtr>>) -> Result<Flags, Refused> {
    CHOICES.listed.set(0);
    CHOICES.resolutions.set(0);
    let Some(modes) = modes else {
        return Ok(Flags {
            carry: true,
            zero: false,
        });
    };
    let mut listed = Vec::new();
    for (n, &at) in modes.iter().enumerate() {
        let mode = read(at)?;
        let (size_key, kind_key) = mc::keys(mode);
        let size = entry_or_past(size_key, Modes::sizes_ptr(), 8)?;
        let kind = entry_or_past(kind_key, Modes::pixel_kinds_ptr(), 2)?;
        let l = mc::Listed {
            width: read_word(size.offset(4))?,
            height: read_word(size.offset(6))?,
            figure: read(kind.offset(4))?,
            mode,
        };
        let slot = 16 * n;
        CHOICES.list.set_u32(slot, l.width);
        CHOICES.list.set_u32(slot + 4, l.height);
        CHOICES.list.set_u32(slot + 8, l.figure);
        CHOICES.list.set_u32(slot + 0xC, l.mode);
        CHOICES.resolutions.set(CHOICES.resolutions.get() + 1);
        listed.push(l.mode);
    }
    CHOICES
        .mip_limit
        .set(crate::d3d::cards_store::CARDS_3D.mips.get());
    CHOICES.listed.set(1);
    CHOICES.resolution.set(mc::in_use_place(&listed, current));
    let race = SCREEN_MODE.race_mode.get();
    let failed = std::cell::Cell::new(None);
    let (place, third) = mc::race_place(&listed, race, || {
        entry_or_past(mc::third_key(race), Modes::kinds_ptr(), 3)
            .and_then(|e| read(e.offset(4)))
            .unwrap_or_else(|e| {
                failed.set(Some(e));
                0
            })
    });
    if let Some(e) = failed.take() {
        return Err(e.into());
    }
    CHOICES.resolution.set(place);
    CHOICES.buffering.set(third);
    let figure = mc::figure(CHOICES.mipmapping.get(), CHOICES.mip_limit.get());
    CHOICES.mipmapping.set(figure);
    CHOICES.made.set(1);
    Ok(Flags {
        carry: false,
        zero: figure == 0,
    })
}
