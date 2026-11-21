//! The graphics options' screen mode choices, in the record `gfx::choices_store` (`made`: made, the
//! video card (0 primary, 1 secondary), the resolution (a place in the list), the buffering (0
//! double, 1 triple), mipmapping, the two cards present, the list made, the mipmapping's limit, the
//! list's length, the list: 16 bytes a resolution, its width, height, pixel kind's figure and the
//! mode). The card is the mode's bit `display.modes.kind_bits` (0x1000: the secondary).
//!
//! - Made and the choice to be tried: the mode chosen (0x300000 with `kind_bits[kind]`, the list's
//!   mode at the chosen resolution and `third_bits[buffering]`) tried (`gfx::try_screen_mode`); if
//!   it works and the mipmapping choice is not above the texture figure (`cards.mips`) it is kept
//!   (`gfx::keep_race_screen_mode`); either way the menus' mode set again
//!   (`gfx::set_menu_screen_mode`); failure if it did not work.
//! - Not made: the card from the race screen mode (0x1000 or not), the cards present, then as made
//!   without a choice to try, from the race screen mode.
//! - Made and no choice to try (or not made): the list made again from the mode table's modes of
//!   the pixel kind (`system::find_entry_with_bits`, `system::next_entry`), each with its size and
//!   pixel kind's figure by `system::find_entry` (a key not there: the place past the table, as the
//!   game reads it); the resolution is the mode's place in the list by its low 12 bits (else the
//!   first whose width is not below the mode's: the search keeps 0, its bound never moves); then
//!   the resolution and the buffering from the race screen mode (its low 12 bits in the list, else
//!   both 0; the buffering from `modes.kinds` by its fourth nibble, less 1, not below 0); the
//!   mipmapping choice masked by its limit; made.
//!
//! Refuses (before anything is written) a list longer than the record's room (64 modes: it would
//! write over the constants after the record) and a size's place of 0x1000 or more when made (it
//! would read that far past the list).

use crate::d3d::owner::D3d;
use crate::errors::{BadInput, Refused};
use crate::gfx::modes::Modes;
use crate::gfx::owner::{Display, Gfx};
use rc2k_core::gfx::mode_choices as mc;

/// The widest race a phone offers while its 3D is on the software rasteriser (a phone's 2868 x 1320
/// ran at 5 to 7 frames a second): the screen's ratio at this width, unless the player sets a size.
const SOFTWARE_PHONE_WIDTH: u32 = 1280;

/// `system::find_entry` as the game uses it (a run-on past a table's last entry goes on into the
/// field after it): the dword of `Modes` where the entry whose first dword is `key` starts, or the
/// place past the table's `count` entries.
fn entry_or_past(modes: &Modes, key: u32, table: usize, count: u32) -> Result<usize, BadInput> {
    for n in 0..count as usize {
        if modes.dword(table + 2 * n)? == key {
            return Ok(table + 2 * n);
        }
    }
    Ok(table + 2 * count as usize)
}

/// `entry_or_past` for a key that must be listed: the table of modes (`standalone_modes`, the only
/// writer, with `mark_3d_modes` setting only high bits) holds a size and a pixel kind taken from
/// `sizes` and `pixel_kinds`, which nothing writes, so a mode's size key and pixel kind key are
/// always among their entries.
fn entry_of(modes: &Modes, key: u32, table: usize, count: u32) -> Result<usize, BadInput> {
    let at = entry_or_past(modes, key, table, count)?;
    if at == table + 2 * count as usize {
        return Err(BadInput(
            "a listed mode's size or pixel kind key is not in its table",
        ));
    }
    Ok(at)
}

/// The list's mode at `place`, which may lie past the list's end (below `PLACE_LIMIT`).
fn list_mode(display: &Display, place: u32) -> Result<u32, BadInput> {
    display
        .choices
        .list
        .dword(place, 3, "a screen mode table's dword")
}

/// `system::find_entry_with_bits`: the dword in `Modes` of the first of `count` entries from
/// `table` whose first dword has all of `bits`, and the entries left with it.
fn first_with_bits(modes: &Modes, bits: u32, table: usize, count: u32) -> Option<(usize, u32)> {
    (0..count).find_map(|n| {
        let at = table + 2 * n as usize;
        let key = modes
            .dword(at)
            .unwrap_or_else(|e| crate::system::checked_reads::refuse(e));
        (key & bits == bits).then_some((at, count - n))
    })
}

/// The modes the list would hold (the mode table's of the pixel kind's bits), or None when it stops
/// (the kind's choice past 1, no mode of the kind).
fn modes_of(display: &Display, kind: u32) -> Option<Vec<usize>> {
    if kind > 1 {
        return None;
    }
    let modes = &display.modes;
    let bits = modes.kind_bits[kind as usize].get();
    let (mut at, mut left) = first_with_bits(modes, bits, Modes::TABLE_AT, modes.count.get())?;
    let mut found = Vec::new();
    loop {
        found.push(at);
        // `system::next_entry`: on by one entry, a failure when none are left.
        if left <= 1 {
            break;
        }
        // The mode table's entries are two dwords each (`Modes::table`).
        match first_with_bits(modes, bits, at + 2, left - 1) {
            Some(next) => (at, left) = next,
            None => break,
        }
    }
    Some(found)
}

/// With `try_choice` (true) the choice tried, else the list made again. True when the choice was taken
/// or the list made, false when the choice was not possible or did not work (the list is not
/// made without modes).
pub fn screen_mode_choices(
    gfx: &Gfx,
    d3d: &D3d,
    system: &crate::system::owner::System,
    try_choice: bool,
) -> Result<bool, Refused> {
    let display = &gfx.display;
    let failed = || Ok(false);
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
            let tried = crate::gfx::try_screen_mode::try_screen_mode(gfx, system, d3d, mode)?;
            let worked =
                tried && mc::kept(display.choices.mipmapping.get(), d3d.cards.get().mips.get());
            if worked {
                crate::gfx::keep_race_screen_mode::keep_race_screen_mode(gfx, system, d3d, mode)?;
                // A size chosen from the list is no longer the window's own.
                if crate::gfx::monitor_sizes::size_of_mode(mode)
                    != crate::gfx::monitor_sizes::window()
                {
                    crate::gfx::monitor_sizes::set_window(None);
                }
            }
            crate::platform::set_menu_mode_now(gfx, system);
            if !worked {
                return failed();
            }
            return Ok(true);
        }
        return rebuild(display, d3d, current, modes);
    }
    display.choices.card.set(kind);
    // The one card there is: a primary one that draws in 3D.
    display.choices.has_primary.set(1);
    display.choices.has_secondary.set(0);
    rebuild(display, d3d, display.mode.race_mode.get(), modes)
}

/// The list made again, `current` the mode kept.
fn rebuild(
    display: &Display,
    d3d: &D3d,
    current: u32,
    modes: Option<Vec<usize>>,
) -> Result<bool, Refused> {
    display.choices.listed.set(0);
    display.choices.resolutions.set(0);
    let Some(modes) = modes else {
        return Ok(false);
    };
    let mut all = Vec::new();
    for &at in &modes {
        let mode = display.modes.dword(at)?;
        let (size_key, kind_key) = mc::keys(mode);
        let size = entry_of(&display.modes, size_key, Modes::SIZES_AT, 8)?;
        let kind = entry_of(&display.modes, kind_key, Modes::PIXEL_KINDS_AT, 2)?;
        // The entries' second dwords: the size's two halves, the kind's figure.
        let size_halves = display.modes.dword(size + 1)?;
        all.push(mc::Listed {
            width: size_halves & 0xFFFF,
            height: size_halves >> 16,
            figure: display.modes.dword(kind + 1)?,
            mode,
        });
    }
    // The sizes beyond the game's table: the monitor's modes, the usual sizes that fit its
    // desktop and the size set by hand, each as the first mode's kind with a size of its own.
    if let Some(first) = all.first().copied() {
        let (modes, desktop) = crate::platform::monitor_modes();
        // A phone whose 3D is on the software rasteriser cannot carry the screen's full size: the
        // desktop is taken as the cut size (the size set by hand stays what the player chose).
        let cap = crate::render::gpu::software_phone()
            .then(|| rc2k_core::gfx::resolutions::capped_desktop(desktop, SOFTWARE_PHONE_WIDTH))
            .flatten();
        let desktop = cap.or(desktop);
        let monitor = rc2k_core::gfx::resolutions::modes_as_sizes(&modes);
        crate::gfx::monitor_sizes::set_monitor(monitor.clone(), desktop);
        let table: Vec<(u32, u32, u32)> = all
            .iter()
            .map(|l| (l.width, l.height, l.mode & 0xFF))
            .collect();
        let table_sizes: Vec<(u32, u32)> = all.iter().map(|l| (l.width, l.height)).collect();
        for (w, h) in rc2k_core::gfx::resolutions::extra_sizes(
            &table_sizes,
            &monitor,
            desktop,
            crate::gfx::monitor_sizes::custom().or(cap),
        ) {
            let Some(mode) = crate::gfx::monitor_sizes::mode_for(first.mode, &table, w, h) else {
                break;
            };
            all.push(mc::Listed {
                width: w,
                height: h,
                figure: first.figure,
                mode,
            });
        }
        // The window's own size, when the list has no entry of that size already.
        let window = crate::gfx::monitor_sizes::window();
        let window_listed =
            window.filter(|&(w, h)| !all.iter().any(|l| (l.width, l.height) == (w, h)));
        crate::gfx::monitor_sizes::set_window_listed(window_listed);
        if let Some((w, h)) = window_listed
            && let Some(mode) = crate::gfx::monitor_sizes::mode_for(first.mode, &table, w, h)
        {
            all.push(mc::Listed {
                width: w,
                height: h,
                figure: first.figure,
                mode,
            });
        }
        // The game's own table has sizes up to 1920 x 1280 whatever the screen: one larger than
        // the desktop could only be shown shrunk, with bars, so it is left out (the size set by
        // hand stays). A saved race size that is left out is replaced by the largest size listed.
        if let Some((dw, dh)) = desktop {
            let custom = crate::gfx::monitor_sizes::custom();
            let fits = |l: &mc::Listed| {
                (l.width <= dw && l.height <= dh)
                    || custom == Some((l.width, l.height))
                    || window_listed == Some((l.width, l.height))
            };
            if all.iter().any(fits) {
                let race = display.mode.race_mode.get();
                let saved = all
                    .iter()
                    .find(|l| (l.mode ^ race) & mc::SAME_SIZE == 0)
                    .copied();
                all.retain(fits);
                all.sort_by_key(|l| rc2k_core::gfx::resolutions::order(l.width, l.height));
                if let (Some(saved), Some(used)) = (
                    saved,
                    all.iter()
                        .rev()
                        .find(|l| l.width <= dw && l.height <= dh)
                        .copied(),
                ) && !fits(&saved)
                {
                    crate::program_log::log(&format!(
                        "screen: the saved race size {} x {} is larger than the desktop {dw} x {dh}: {} x {} used",
                        saved.width, saved.height, used.width, used.height
                    ));
                    display
                        .mode
                        .race_mode
                        .set((race & !mc::SAME_SIZE) | (used.mode & mc::SAME_SIZE));
                }
            }
        }
        all.sort_by_key(|l| rc2k_core::gfx::resolutions::order(l.width, l.height));
        all.truncate(mc::ROOM as usize);
    }
    let mut listed = Vec::new();
    for (n, l) in all.iter().enumerate() {
        // `n` is below the list's room: the table's modes were counted before anything was
        // written and the rest cut to the room.
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
    display.choices.mip_limit.set(d3d.cards.get().mips.get());
    display.choices.listed.set(1);
    display
        .choices
        .resolution
        .set(mc::in_use_place(&listed, current));
    let race = display.mode.race_mode.get();
    let failed = std::cell::Cell::new(None);
    let (place, third) = mc::race_place(&listed, race, || {
        entry_or_past(&display.modes, mc::third_key(race), Modes::KINDS_AT, 3)
            // (a third's entry: the race mode's key may be none of the three, then the place past
            // `kinds` is `other_tables`' first entry)
            .and_then(|e| display.modes.dword(e + 1))
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
    Ok(true)
}
