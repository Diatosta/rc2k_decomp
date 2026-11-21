//! The two record rows of a stage: `best time: <flag> <name> <car> <time>` and `top speed:
//! <flag> <name> <car> <speed>` (strings 0x34B and 0x34C). Takes the best-time entry (nationality,
//! car number, time in hundredths of a second, name). The top-speed row comes from the record the
//! game has selected (speed in tenths of a mile per hour, name, nationality, car number). The
//! sprites take whatever the text left as their palette, which they do not use (kept). It has
//! three more call sites that have not been looked at.

use crate::errors::{BadInput, Refused};
use crate::menu::owner::Menus;
use crate::records::stages_store::{Holder as RecordHolder, RecordRef, RecordTable};
use crate::screens::game_screen::{Held, HeldText};
use crate::screens::stage_panel::PanelScreen;
use crate::strings::append_time::show_hours;
use crate::strings::units::units;
use crate::system::owner::System;
use crate::text::draw_text::Canvas;

// ---- game data ----

/// The game's selection: a group, a table in that group and a record in that table.
#[derive(Clone, Copy, Debug)]
pub struct Selection {
    /// EventState.stage.group
    pub group: u32,
    /// EventState.stage.sub
    pub sub: u32,
    /// `tables[group][sub]`: the address of the table's first record.
    pub table: RecordTable,
    /// `table + index * 0xA4`, with index = EventState.stage.number.
    pub record: RecordRef,
}

/// None while the tables are not there.
pub fn selection_in(menus: &Menus, system: &System) -> Result<Option<Selection>, BadInput> {
    let screens = &menus.screens;
    let group: u32 = screens.event.read().stage.group;
    let index: u32 = screens.event.read().stage.number;
    let sub: u32 = screens.event.read().stage.sub;
    let table = system
        .records()
        .stage_tables
        .group_event_table(group, sub)?;
    Ok(Some(Selection {
        group,
        sub,
        table,
        record: crate::records::stages_store::row(&system.records().stage_tables, table, index)?,
    }))
}

fn selected_record(menus: &Menus, system: &System) -> Result<Option<(RecordRef, u32)>, BadInput> {
    Ok(selection_in(menus, system)?.map(|s| (s.record, s.sub)))
}

/// Where the top-speed row's values are in the selected record: the stage panels'
/// (`draw_stage_records`) and the time trial's (`TIME_TRIAL_ROWS`).
pub struct Rows {
    pub holder: RecordHolder,
}

pub const STAGE_ROWS: Rows = Rows {
    holder: RecordHolder::Fastest,
};

pub(super) const TIME_TRIAL_ROWS: Rows = Rows {
    holder: RecordHolder::TimeTrial,
};

/// Returns whether the last text was too wide for its box. Refuses a missing record table and what
/// the speed formatter refuses; the fatal errors are the blitter's.
pub fn draw_stage_records(
    o: crate::owners::Owners<'static>,
    system: &System,
    entry: &rc2k_core::saves::entries::BestTime,
    canvas: &mut dyn Canvas,
) -> Result<bool, Refused> {
    draw_records_rows(o, system, entry, &STAGE_ROWS, canvas)
}

/// `draw_stage_records` with the top-speed row's places given. The logic is in
/// `rc2k_core::screens::stage_panel::records`.
pub(super) fn draw_records_rows(
    o: crate::owners::Owners<'static>,
    system: &System,
    entry: &rc2k_core::saves::entries::BestTime,
    rows: &Rows,
    canvas: &mut dyn Canvas,
) -> Result<bool, Refused> {
    let menus = o.menus;
    use rc2k_core::screens::stage_panel::Holder;
    {
        // Refused without a record table (the first row is drawn after this).
        let Some((record, _)) = selected_record(menus, system)? else {
            return Err(
                BadInput("no record table is selected: the original reads through null").into(),
            );
        };
        let name = entry
            .name_text()
            .and_then(|t| Held::new(&t))
            .ok_or(BadInput("a best time's name without its end"))?;
        let best = Holder {
            nation: entry.head.nation,
            name: HeldText::Held(name),
            car: entry.head.car,
            figure: entry.head.key,
        };
        // The stage record is the game's.
        let held = record
            .record(&system.records().stage_tables)?
            .holder(rows.holder);
        let top = Holder {
            nation: held.nation,
            name: HeldText::Held(record.holder_text(&system.records().stage_tables, rows.holder)?),
            car: held.car,
            figure: held.speed,
        };
        rc2k_core::screens::stage_panel::records(
            &mut PanelScreen::new(canvas, o),
            best,
            top,
            show_hours(o.strings),
            units(o.strings),
        )
    }
}
