//! One heading line for a stage record: string 0x481, a space, `<record>/6`, ": " and the
//! name of the record's track pack entry; it reads like "es 1/6: Clocaenog centro" in the
//! Portuguese build. Takes the record. Part of the panel `draw_stage_info` draws (seen as the
//! stage details screen shown before each stage of a championship); its other caller,
//! `screens::draw_service_stage_info`, has not been seen in the game.

use crate::errors::Refused;
use crate::records::stages_store::RecordRef;
use crate::records::trakpak_entry::track_of;
use crate::screens::stage_panel::PanelScreen;
use crate::strings::append_string_by_id::{StringOwners, push_string};
use crate::text::draw_text::Canvas;
use rc2k_core::screens::stage_panel::stage_line;

// ---- game data ----

/// The same line starting with another label string (the stage panels build one with 0x482).
/// The logic is in `rc2k_core::screens::stage_panel::stage_line`.
pub(super) fn heading_text(
    o: StringOwners<'_>,
    record: RecordRef,
    label: u32,
) -> Result<Vec<u8>, Refused> {
    let fault = crate::errors::Fault::default();
    let stages = &o.system.records().stage_tables;
    let name = || -> Result<u32, Refused> { Ok(track_of(stages, record)?.name(stages)?) };
    let number = record.record(&o.system.records().stage_tables)?.number();
    let line = stage_line(
        &mut |id| fault.ok(string_bytes(o, id)),
        label,
        number,
        &name,
    );
    fault.check()?;
    let mut line = line?;
    line.push(0);
    Ok(line)
}

/// String `id`'s bytes.
pub fn string_bytes(o: StringOwners<'_>, id: u32) -> Result<Vec<u8>, crate::errors::BadInput> {
    let mut out = Vec::new();
    push_string(o, &mut out, id)?;
    Ok(out)
}

/// Builds the line in the game's line buffer and draws it. Returns whether it was too wide.
/// The logic is in `rc2k_core::screens::stage_panel::heading`.
pub(super) fn draw_stage_heading(
    o: crate::owners::Owners<'static>,
    record: RecordRef,
    canvas: &mut dyn Canvas,
) -> Result<bool, Refused> {
    let stages = &o.system.records().stage_tables;
    let name = || -> Result<u32, Refused> { Ok(track_of(stages, record)?.name(stages)?) };
    rc2k_core::screens::stage_panel::heading(
        &mut PanelScreen::new(canvas, o),
        record.record(&o.system.records().stage_tables)?.number(),
        &name,
    )
}

//
