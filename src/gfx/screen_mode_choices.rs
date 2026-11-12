//! `gfx::screen_mode_choices` (0x4045D6): the graphics options' screen mode choices, the
//! record `gfx::choices_store` (`made`: made, the video card (0 primary, 1
//! secondary), the resolution (a place in the list), the buffering (0 double, 1 triple),
//! mipmapping, the two cards present, the list made, the mipmapping's limit, the list's
//! length, the list: 16 bytes a resolution, its width, height, pixel kind's figure and
//! the mode). The card is the mode's bit `display.modes.kind_bits` (0x1000: the secondary).
//!
//! - Made and the choice to be tried: the mode chosen (0x300000 with `kind_bits[+4]`, the
//!   list's mode at +8 and `third_bits[+0xC]`) tried (`gfx::try_screen_mode`); if it works and
//!   +0x10 is not above the texture figure (`cards.mips`) it is kept (`gfx::keep_race_screen_mode`); either
//!   way the menus' mode set again (`gfx::set_menu_screen_mode`); carry if it did not work.
//! - Not made: the card from `g_raceScreenMode` (0x1000 or not), the cards present, then
//!   as made without a choice to try, from `g_raceScreenMode`.
//! - Made and no choice to try (or not made): the list made again from the mode table's modes of the
//!   pixel kind (`system::find_entry_with_bits`, `system::next_entry`), each with its size
//!   (`g_screenSizes`) and pixel kind's figure (`g_pixelKinds`) by `system::find_entry`
//!   (a key not there: the place past the table, as the original reads it); +8 the mode's
//!   place in the list by its low 12 bits (else the first whose width is not below the
//!   mode's: the original's search keeps 0, its bound never moves); then +8 and +0xC from
//!   `g_raceScreenMode` (its low 12 bits in the list, else both 0; +0xC from
//!   `modes.kinds` by its fourth nibble, less 1, not below 0); +0x10 masked by +0x20;
//!   made.
//!
//! Callers: `gfx::choose_screen_modes` and the graphics options' actions
//! (`menu::apply_screen_mode`). Refuses (before anything is written) a list longer than
//! the record's room (16 modes: the original would write over the constants after the record) and a size's
//! place of 0x1000 or more when made (the original reads that far past the list).

use crate::d3d::owner::{D3d, d3d};
use crate::errors::{BadInput, Refused};
use crate::framework::CarryZeroFlags;
use crate::gfx::modes::Modes;
use crate::gfx::owner::{Display, Gfx};
use core::mem::offset_of;
use rc2k_core::gfx::mode_choices as mc;

// ---- game data (IDA addresses) ----

/// `system::find_entry` as the original uses it (the mode tables are walked as bytes, a
/// run-on past their last entry as the original): the offset in `Modes` of the entry whose
/// first dword is `key`, or of the place past the table's `count` entries.
fn entry_or_past(modes: &Modes, key: u32, table: usize, count: u32) -> Result<usize, BadInput> {
    for n in 0..count {
        if read(modes, table + 8 * n as usize)? == key {
            return Ok(table + 8 * n as usize);
        }
    }
    Ok(table + 8 * count as usize)
}

fn read(modes: &Modes, at: usize) -> Result<u32, BadInput> {
    crate::pointers::load_value(modes, at)
        .inspect_err(|_| crate::framework::log("a screen mode table's dword: past the block"))
}

/// The list's mode at `place`, which may lie past the list's end (below `PLACE_LIMIT`).
fn list_mode(display: &Display, place: u32) -> Result<u32, BadInput> {
    display
        .choices
        .list
        .dword(place, 3, "a screen mode table's dword")
}

fn read_word(modes: &Modes, at: usize) -> Result<u32, BadInput> {
    Ok(crate::pointers::load_value::<u16>(modes, at)
        .inspect_err(|_| crate::framework::log("a screen mode table's word: past the block"))?
        as u32)
}

/// `system::find_entry_with_bits`: the offset in `Modes` of the first of `count` entries from
/// `table` whose first dword has all of `bits`, and the entries left with it.
fn first_with_bits(modes: &Modes, bits: u32, table: usize, count: u32) -> Option<(usize, u32)> {
    (0..count).find_map(|n| {
        let at = table + 8 * n as usize;
        let key = read(modes, at).unwrap_or_else(|e| crate::system::checked_reads::refuse(e));
        (key & bits == bits).then_some((at, count - n))
    })
}

/// The modes the list would hold (the mode table's of the pixel kind's bits), or None
/// when the original stops (the kind's choice past 1, no mode of the kind).
fn modes_of(display: &Display, kind: u32) -> Option<Vec<usize>> {
    if kind > 1 {
        return None;
    }
    let modes = &display.modes;
    let bits = modes.kind_bits[kind as usize].get();
    let (mut at, mut left) =
        first_with_bits(modes, bits, offset_of!(Modes, table), modes.count.get())?;
    let mut found = Vec::new();
    loop {
        found.push(at);
        // `system::next_entry`: on by one entry, the carry set when none are left.
        if left <= 1 {
            break;
        }
        // The mode table walked as bytes, its entries 8 bytes each (`Modes::table`).
        match first_with_bits(modes, bits, at + 8, left - 1) {
            Some(next) => (at, left) = next,
            None => break,
        }
    }
    Some(found)
}

/// With `try_choice` (true) the choice tried, else the list made
/// again; the flags it leaves (see the header).
pub fn screen_mode_choices(gfx: &Gfx, try_choice: bool) -> Result<CarryZeroFlags, Refused> {
    screen_mode_choices_in(gfx, d3d(), try_choice)
}

/// `screen_mode_choices` for a caller that holds the owners.
pub fn screen_mode_choices_in(
    gfx: &Gfx,
    d3d: &D3d,
    try_choice: bool,
) -> Result<CarryZeroFlags, Refused> {
    let display = &gfx.display;
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
        // The list's mode at the place chosen (read before the choice flag is looked at).
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
            let tried = crate::gfx::try_screen_mode::try_screen_mode(gfx, mode)?;
            let worked =
                !tried.carry && mc::kept(display.choices.mipmapping.get(), d3d.cards.mips.get());
            if worked {
                crate::gfx::keep_race_screen_mode::keep_race_screen_mode(gfx, mode)?;
            }
            crate::platform::set_menu_mode_now(gfx);
            if !worked {
                return failed();
            }
            // The mode the three calls leave (they keep it).
            return Ok(CarryZeroFlags {
                carry: false,
                zero: mode == 0,
            });
        }
        return rebuild(display, d3d, current, modes);
    }
    display.choices.card.set(kind);
    display.choices.has_primary.set(display.cards.primary.get());
    display
        .choices
        .has_secondary
        .set(display.cards.secondary.get());
    rebuild(display, d3d, display.mode.race_mode.get(), modes)
}

/// The list made again, `current` the mode kept.
fn rebuild(
    display: &Display,
    d3d: &D3d,
    current: u32,
    modes: Option<Vec<usize>>,
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
        let mode = read(&display.modes, at)?;
        let (size_key, kind_key) = mc::keys(mode);
        let size = entry_or_past(&display.modes, size_key, offset_of!(Modes, sizes), 8)?;
        let kind = entry_or_past(&display.modes, kind_key, offset_of!(Modes, pixel_kinds), 2)?;
        // The entries' fields by offset: the place may be the one past the table, as the original.
        let l = mc::Listed {
            width: read_word(&display.modes, size + 4)?,
            height: read_word(&display.modes, size + 6)?,
            figure: read(&display.modes, kind + 4)?,
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
    display.choices.mip_limit.set(d3d.cards.mips.get());
    display.choices.listed.set(1);
    display
        .choices
        .resolution
        .set(mc::in_use_place(&listed, current));
    let race = display.mode.race_mode.get();
    let failed = std::cell::Cell::new(None);
    let (place, third) = mc::race_place(&listed, race, || {
        entry_or_past(
            &display.modes,
            mc::third_key(race),
            offset_of!(Modes, kinds),
            3,
        )
        // (a third's entry, the place past the table possible, as above)
        .and_then(|e| read(&display.modes, e + 4))
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
