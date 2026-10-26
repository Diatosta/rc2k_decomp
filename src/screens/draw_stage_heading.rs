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

use crate::errors::Refused;
use crate::records::trakpak_entry::record_entry;
use crate::screens::stage_panel::PanelScreen;
use crate::strings::append_string_by_id::push_string;
use crate::text::draw_text::Canvas;
use rc2k_core::screens::stage_panel::stage_line;

// ---- game data ----

fn dword(at: u32) -> Result<u32, crate::errors::BadInput> {
    crate::pointers::GamePtr::new(at).read::<u32>("a stage record")
}

// ---- Rust version ----

/// The same line starting with another label string (the stage panels build one with 0x482).
/// Converted: `rc2k_core::screens::stage_panel::stage_line`.
pub fn heading_text(record: u32, label: u32) -> Result<Vec<u8>, Refused> {
    let fault = crate::menu::event_flow::Fault::default();
    let name = || -> Result<u32, Refused> { Ok(dword(record_entry(record)?)?) };
    let number = dword(record + 0x40)?;
    let line = stage_line(&mut |id| fault.ok(string_bytes(id)), label, number, &name);
    fault.check()?;
    let mut line = line?;
    line.push(0);
    Ok(line)
}

/// String `id`'s bytes.
pub fn string_bytes(id: u32) -> Result<Vec<u8>, crate::errors::BadInput> {
    let mut out = Vec::new();
    push_string(&mut out, id)?;
    Ok(out)
}

/// Builds the line in the game's line buffer and draws it. Returns the carry flag.
/// Converted: `rc2k_core::screens::stage_panel::heading`.
pub fn draw_stage_heading(record: u32, canvas: &mut dyn Canvas) -> Result<bool, Refused> {
    let name = || -> Result<u32, Refused> { Ok(dword(record_entry(record)?)?) };
    rc2k_core::screens::stage_panel::heading(
        &mut PanelScreen::new(canvas),
        dword(record + 0x40)?,
        &name,
    )
}

//
