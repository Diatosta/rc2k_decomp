//! `screens::draw_stage_heading` (0x466DF3): draws one heading line for a record:
//! string 0x481, a space, "<record +0x40>/6", ": " and the name of the record's track pack
//! entry, at (747, 145) with the font at 0x51E9A4.
//!
//! In:  edi = record
//! Out: carry as `draw_text` leaves it; every register preserved
//! The line reads like "es 1/6: Clocaenog centro" in the Portuguese build: string 0x481 is the
//! short stage label.
//!
//! Screen: it is part of the panel `draw_stage_info` draws, which the owner confirmed on
//! 2026-10-10 as the stage details screen shown before each stage of a championship. Its other
//! caller, `sub_480064`, has not been seen in the game.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{FatalError, Refused};
use crate::framework::Regs;
use crate::records::trakpak_entry::record_entry;
use crate::screens::blanked;
use crate::screens::stage_panel::PanelScreen;
use crate::strings::append_string_by_id::push_string;
use crate::text::draw_text::{Canvas, GameCanvas};
use rc2k_core::screens::stage_panel::stage_line;

// ---- game data ----

const HEADING_STRING: u32 = 0x481;

unsafe fn dword(at: *const u8) -> u32 {
    unsafe { at.cast::<u32>().read_unaligned() }
}

// ---- Rust version ----

/// The line with its NUL. The error is the track pack lookup's.
pub unsafe fn stage_heading_text(record: *const u8) -> Result<Vec<u8>, FatalError> {
    unsafe { heading_text(record, HEADING_STRING) }
}

/// The same line starting with another label string (the stage panels build one with 0x482).
/// Converted: `rc2k_core::screens::stage_panel::stage_line`.
pub unsafe fn heading_text(record: *const u8, label: u32) -> Result<Vec<u8>, FatalError> {
    unsafe {
        let name = || -> Result<u32, FatalError> { Ok(dword(record_entry(record)? as *const u8)) };
        let mut line = stage_line(&mut string_bytes, label, dword(record.add(0x40)), &name)?;
        line.push(0);
        Ok(line)
    }
}

/// String `id`'s bytes.
pub fn string_bytes(id: u32) -> Vec<u8> {
    let mut out = Vec::new();
    unsafe { push_string(&mut out, id) };
    out
}

/// Builds the line in the game's line buffer and draws it. Returns the carry flag.
/// Converted: `rc2k_core::screens::stage_panel::heading`.
pub unsafe fn draw_stage_heading(
    record: *const u8,
    canvas: &mut dyn Canvas,
) -> Result<bool, Refused> {
    unsafe {
        let name = || -> Result<u32, Refused> { Ok(dword(record_entry(record)? as *const u8)) };
        rc2k_core::screens::stage_panel::heading(
            &mut PanelScreen::new(canvas),
            dword(record.add(0x40)),
            &name,
        )
    }
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    if blanked("screens::draw_stage_heading") {
        return Ok(());
    }
    let too_wide = unsafe { draw_stage_heading(r.edi as *const u8, &mut GameCanvas) }?;
    r.set_carry(too_wide);
    r.set_zero(!too_wide);
    Ok(())
}

//
// Uses the game's own font, strings and loaded track pack, so it only runs when those are
// there. The blitter's hook swallows the calls; the line buffer and the kerning table are
// restored.
