//! `gfx::screen_mode_choices` (0x4045D6): the graphics options' screen mode choices, the
//! record at `unk_4F50C0` (`gfx::choices_store`: made, the video card (0 primary, 1
//! secondary), the resolution (a place in the list), the buffering (0 double, 1 triple),
//! mipmapping, the two cards present, the list made, the mipmapping's limit, the list's
//! length, the list: 16 bytes a resolution, its width, height, pixel kind's figure and
//! the mode). The card is the mode's bit `display.modes.kind_bits` (0x1000: the secondary).
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
//! Callers: `gfx::choose_screen_modes` (0x4045CA) and the graphics options' actions
//! (0x470017, `menu::apply_screen_mode`). Refuses (before anything is written) a list longer than
//! the record's room (16 modes: the original would write over 0x4F51F0's constants) and a size's
//! place of 0x1000 or more when made (the original reads that far past the list).

use crate::d3d::owner::d3d;
use crate::errors::{BadInput, Refused};
use crate::framework::CarryZeroFlags;
use crate::gfx::modes::Modes;
use crate::gfx::owner::Display;
use crate::pointers::GamePtr;
use crate::system::find_entry_with_bits::find_entry_with_bits;
use rc2k_core::gfx::mode_choices as mc;

// ---- game data (IDA addresses) ----

/// `system::find_entry` as the original uses it (the mode tables are walked as bytes, a
/// run-on past their last entry as the original): the entry whose first dword is `key`, or
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
fn list_mode(display: &Display, place: u32) -> Result<u32, BadInput> {
    display
        .choices
        .list
        .dword(place, 3, "a screen mode table's dword")
}

fn read_word(at: GamePtr) -> Result<u32, BadInput> {
    Ok(at.read::<u16>("a screen mode table's word")? as u32)
}

/// The modes the list would hold (the mode table's of the pixel kind's bits), or None
/// when the original stops (the kind's choice past 1, no mode of the kind).
fn modes_of(display: &Display, kind: u32) -> Option<Vec<GamePtr>> {
    if kind > 1 {
        return None;
    }
    let bits = display.modes.kind_bits[kind as usize].get();
    let (mut found, mut at, mut left) =
        find_entry_with_bits(bits, Modes::mode_table_ptr(), display.modes.count.get());
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
        // The mode table walked as bytes, its entries 8 bytes each (`Modes::table`).
        (found, at, left) = find_entry_with_bits(bits, at.offset(8), left - 1);
    }
    Some(modes)
}

/// With `try_choice` (eax not 0) the choice tried, else the list made
/// again; the flags it leaves (see the header).
pub fn screen_mode_choices(display: &Display, try_choice: bool) -> Result<CarryZeroFlags, Refused> {
    // Converted: `rc2k_core::gfx::mode_choices`.
    let failed = || {
        Ok(CarryZeroFlags {
            carry: true,
            zero: false,
        })
    };
    let made = display.choices.made.get() == 1;
    // The way taken and what decides it, before anything is written.
    let kind = if made {
        display.choices.card.get()
    } else {
        mc::kind_of_race(
            display.mode.race_mode.get(),
            display.modes.kind_bits[0].get(),
        )
    };
    let rebuilding = !made || !try_choice;
    let modes = if rebuilding {
        modes_of(display, kind)
    } else {
        None
    };
    if let Some(modes) = &modes
        && modes.len() as u32 > mc::ROOM
    {
        return Err(BadInput("more screen modes than the choices' list holds").into());
    }
    if made {
        // ebp: the list's mode at the place chosen (read before eax is looked at).
        if display.choices.resolution.get() >= mc::PLACE_LIMIT {
            return Err(BadInput("a size's place far past the choices' list").into());
        }
        let current = list_mode(display, display.choices.resolution.get())?;
        if try_choice {
            // The choice tried.
            let (third, place) = (
                display.choices.buffering.get(),
                display.choices.resolution.get(),
            );
            if !mc::can_try(kind, place, display.choices.resolutions.get(), third) {
                return failed();
            }
            let mode = mc::tried_mode(
                display.modes.kind_bits[kind as usize].get(),
                list_mode(display, place)?,
                display.modes.third_bits[third as usize].get(),
            );
            let tried = crate::gfx::try_screen_mode::try_screen_mode(display, mode)?;
            let worked =
                !tried.carry && mc::kept(display.choices.mipmapping.get(), d3d().cards.mips.get());
            if worked {
                crate::gfx::keep_race_screen_mode::keep_race_screen_mode(display, mode)?;
            }
            crate::platform::set_menu_mode_now();
            if !worked {
                return failed();
            }
            // The eax the three calls leave: the mode (they keep it).
            return Ok(CarryZeroFlags {
                carry: false,
                zero: mode == 0,
            });
        }
        return rebuild(display, current, modes);
    }
    display.choices.card.set(kind);
    display.choices.has_primary.set(display.cards.primary.get());
    display
        .choices
        .has_secondary
        .set(display.cards.secondary.get());
    rebuild(display, display.mode.race_mode.get(), modes)
}

/// The list made again (`loc_40468A` on), `current` the mode kept in ebp.
fn rebuild(
    display: &Display,
    current: u32,
    modes: Option<Vec<GamePtr>>,
) -> Result<CarryZeroFlags, Refused> {
    display.choices.listed.set(0);
    display.choices.resolutions.set(0);
    let Some(modes) = modes else {
        return Ok(CarryZeroFlags {
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
        // The entries' fields by offset: the place may be the one past the table, as the original.
        let l = mc::Listed {
            width: read_word(size.offset(4))?,
            height: read_word(size.offset(6))?,
            figure: read(kind.offset(4))?,
            mode,
        };
        // `n` is below the list's 16: the room was checked before anything was written.
        let slot = &display.choices.list.modes[n];
        slot.width.set(l.width);
        slot.height.set(l.height);
        slot.figure.set(l.figure);
        slot.mode.set(l.mode);
        display
            .choices
            .resolutions
            .set(display.choices.resolutions.get() + 1);
        listed.push(l.mode);
    }
    display.choices.mip_limit.set(d3d().cards.mips.get());
    display.choices.listed.set(1);
    display
        .choices
        .resolution
        .set(mc::in_use_place(&listed, current));
    let race = display.mode.race_mode.get();
    let failed = std::cell::Cell::new(None);
    let (place, third) = mc::race_place(&listed, race, || {
        entry_or_past(mc::third_key(race), Modes::kinds_ptr(), 3)
            // (a third's entry, the place past the table possible, as above)
            .and_then(|e| read(e.offset(4)))
            .unwrap_or_else(|e| {
                failed.set(Some(e));
                0
            })
    });
    if let Some(e) = failed.take() {
        return Err(e.into());
    }
    display.choices.resolution.set(place);
    display.choices.buffering.set(third);
    let figure = mc::figure(
        display.choices.mipmapping.get(),
        display.choices.mip_limit.get(),
    );
    display.choices.mipmapping.set(figure);
    display.choices.made.set(1);
    Ok(CarryZeroFlags {
        carry: false,
        zero: figure == 0,
    })
}
