//! `screens::draw_stage_records` (0x4742BF): draws the two record rows of a stage, at y = 508
//! and y = 531: "best time: <flag> <name> <car> <time>" and
//! "top speed: <flag> <name> <car> <speed>".
//!
//! In:  edi = best-time entry: +0x08 nationality (flag sprite 0x1C + it), +0x10 car number
//! (see `car_sprite`), +0x14 time in hundredths of a second, +0x20 name
//! Out: carry as the last `draw_text` leaves it; every register preserved
//! The top-speed row comes from the record the game has selected (see `selection`):
//! +0x30 speed in tenths of a mile per hour, +0x54 name, +0x74 nationality, +0x78 car number.
//! The labels are strings 0x34B ("melhor tempo" in the Portuguese build) and 0x34C
//! ("velocidade máxima").
//!
//! It has three more call sites that have not been looked at.
//!
//! The sprites are drawn with whatever is in ebx as the palette: a pointer into text, or an
//! index. That only works because those sprites do not use a palette; it is reproduced as is.

use crate::errors::{BadInput, Refused};
use crate::menu::event_store::EVENT_STATE;
use crate::screens::game_screen::{Held, HeldText};
use crate::screens::stage_panel::PanelScreen;
use crate::strings::append_time::show_hours;
use crate::strings::units::units;
use crate::text::draw_text::Canvas;

// ---- game data ----

const ROW_1_Y: i32 = 0x1FC;
const ROW_2_Y: i32 = 0x213;

/// The game's selection: a group, a table in that group and a record in that table.
#[derive(Clone, Copy, Debug)]
pub struct Selection {
    /// dword_7558C8
    pub group: u32,
    /// dword_7558D0
    pub sub: u32,
    /// `tables[group][sub]`: the address of the table's first record.
    pub table: u32,
    /// `table + index * 0xA4`, with index = dword_7558CC.
    pub record: u32,
}

/// None while the tables are not there; the original then reads through a null pointer.
pub fn selection() -> Result<Option<Selection>, BadInput> {
    let group: u32 = EVENT_STATE.stage_group.get();
    let index: u32 = EVENT_STATE.stage_number.get();
    let sub: u32 = EVENT_STATE.stage_sub.get();
    let table = crate::records::stages_store::STAGE_TABLES.group_event_table(group, sub)?;
    Ok(Some(Selection {
        group,
        sub,
        table,
        record: crate::records::stages_store::row(table, index)?,
    }))
}

fn selected_record() -> Result<Option<(u32, u32)>, BadInput> {
    Ok(selection()?.map(|s| (s.record, s.sub)))
}

// ---- Rust version ----

/// Where the top-speed row's values are in the selected record, and the y its car's sprite
/// is drawn at: the stage panels' (`draw_stage_records`) and the time trial's
/// (`screens::draw_time_trial_records` 0x48DE45, whose car sprite sits on the first row's
/// y, sic).
pub struct Rows {
    pub nationality: u32,
    pub name: u32,
    pub car: u32,
    pub speed: u32,
    pub car_y: i32,
}

pub const STAGE_ROWS: Rows = Rows {
    nationality: 0x74,
    name: 0x54,
    car: 0x78,
    speed: 0x30,
    car_y: ROW_2_Y,
};

pub const TIME_TRIAL_ROWS: Rows = Rows {
    nationality: 0x9C,
    name: 0x7C,
    car: 0xA0,
    speed: 0x2C,
    car_y: ROW_1_Y,
};

/// Returns the carry flag of the last `draw_text`. Refuses a missing record table and what
/// the speed formatter refuses; the fatal errors are the blitter's.
pub fn draw_stage_records(
    entry: &rc2k_core::saves::entries::BestTime,
    canvas: &mut dyn Canvas,
) -> Result<bool, Refused> {
    draw_records_rows(entry, &STAGE_ROWS, canvas)
}

/// `draw_stage_records` with the top-speed row's places given. Converted:
/// `rc2k_core::screens::stage_panel::records`.
pub fn draw_records_rows(
    entry: &rc2k_core::saves::entries::BestTime,
    rows: &Rows,
    canvas: &mut dyn Canvas,
) -> Result<bool, Refused> {
    use rc2k_core::screens::stage_panel::{CarRow, Holder};
    {
        // Without it the original reads through a null pointer after drawing the first row.
        let Some((record, _)) = selected_record()? else {
            return Err(
                BadInput("no record table is selected: the original reads through null").into(),
            );
        };
        let f = crate::menu::event_flow::Fault::default();
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
        let top = Holder {
            nation: f.get(record + rows.nationality),
            name: HeldText::Place(
                crate::records::stages_store::STAGE_TABLES.record_text(record, rows.name),
            ),
            car: f.get(record + rows.car),
            figure: f.get(record + rows.speed),
        };
        f.check()?;
        let car_row = if rows.car_y == ROW_1_Y {
            CarRow::First
        } else {
            CarRow::Second
        };
        rc2k_core::screens::stage_panel::records(
            &mut PanelScreen::new(canvas),
            best,
            top,
            car_row,
            show_hours(),
            units(),
        )
    }
}
