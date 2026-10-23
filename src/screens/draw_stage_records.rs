//! `screens::draw_stage_records` (0x4742BF): draws the two record rows of a stage, at y = 508
//! and y = 531: "best time: <flag> <name> <car> <time>" and
//! "top speed: <flag> <name> <car> <speed>".
//!
//! In:  edi = best-time entry: +0x08 nationality (flag sprite 0x1C + it), +0x10 car number
//!      (see `car_sprite`), +0x14 time in hundredths of a second, +0x20 name
//! Out: carry as the last `draw_text` leaves it; every register preserved
//! The top-speed row comes from the record the game has selected (see `selection`):
//! +0x30 speed in tenths of a mile per hour, +0x54 name, +0x74 nationality, +0x78 car number.
//! The labels are strings 0x34B ("melhor tempo" in the Portuguese build) and 0x34C
//! ("velocidade máxima").
//!
//! Screen: confirmed on the stage selection screen ("Escolha Etapa"), simulation mode, by
//! blanking this hook (2026-10-09): the two rows at the bottom disappear and nothing else.
//! It has three more call sites that have not been looked at.
//!
//! The sprites are drawn with whatever is in ebx as the palette: a pointer into text, or an
//! index. That only works because those sprites do not use a palette; it is reproduced as is.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, Refused};
use crate::framework::{Regs, game_read, rva};
use crate::screens::blanked;
use crate::screens::stage_panel::PanelScreen;
use crate::strings::append_time::show_hours;
use crate::strings::units::units;
use crate::text::draw_text::{Canvas, GameCanvas};

// ---- game data ----

const RECORD_TABLES_RVA: usize = rva(0x735512); // off_735512: pointers to tables of tables
const SELECTED_GROUP_RVA: usize = rva(0x7558C8); // dword_7558C8
const SELECTED_INDEX_RVA: usize = rva(0x7558CC); // dword_7558CC
const SELECTED_SUB_RVA: usize = rva(0x7558D0); // dword_7558D0
const RECORD_SIZE: u32 = 0xA4;

const ROW_1_Y: i32 = 0x1FC;
const ROW_2_Y: i32 = 0x213;

unsafe fn dword(at: u32) -> u32 {
    unsafe { (at as *const u32).read_unaligned() }
}

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
pub unsafe fn selection() -> Option<Selection> {
    unsafe {
        let group: u32 = game_read(SELECTED_GROUP_RVA);
        let index: u32 = game_read(SELECTED_INDEX_RVA);
        let sub: u32 = game_read(SELECTED_SUB_RVA);
        let tables: u32 = game_read(RECORD_TABLES_RVA + group as usize * 4);
        if tables == 0 {
            return None;
        }
        let table = dword(tables.wrapping_add(sub.wrapping_mul(4)));
        if table == 0 {
            return None;
        }
        Some(Selection {
            group,
            sub,
            table,
            record: table.wrapping_add(index.wrapping_mul(RECORD_SIZE)),
        })
    }
}

unsafe fn selected_record() -> Option<(u32, u32)> {
    unsafe { selection() }.map(|s| (s.record, s.sub))
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
pub unsafe fn draw_stage_records(entry: u32, canvas: &mut dyn Canvas) -> Result<bool, Refused> {
    unsafe { draw_records_rows(entry, &STAGE_ROWS, canvas) }
}

/// `draw_stage_records` with the top-speed row's places given. Converted:
/// `rc2k_core::screens::stage_panel::records`.
pub unsafe fn draw_records_rows(
    entry: u32,
    rows: &Rows,
    canvas: &mut dyn Canvas,
) -> Result<bool, Refused> {
    use rc2k_core::screens::stage_panel::{CarRow, Holder};
    unsafe {
        // Without it the original reads through a null pointer after drawing the first row.
        let Some((record, sub)) = selected_record() else {
            return Err(
                BadInput("no record table is selected: the original reads through null").into(),
            );
        };
        let at = |base: u32, o: u32| dword(base.wrapping_add(o));
        let best = Holder {
            nation: at(entry, 8),
            name: entry.wrapping_add(0x20),
            car: at(entry, 0x10),
            figure: at(entry, 0x14),
        };
        let top = Holder {
            nation: at(record, rows.nationality),
            name: record.wrapping_add(rows.name),
            car: at(record, rows.car),
            figure: at(record, rows.speed),
        };
        let car_row = if rows.car_y == ROW_1_Y {
            CarRow::First
        } else {
            CarRow::Second
        };
        // The top speed's flag takes the selection's third value as its palette (ebx).
        rc2k_core::screens::stage_panel::records(
            &mut PanelScreen::new(canvas),
            best,
            top,
            sub,
            car_row,
            show_hours(),
            units(),
        )
    }
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    rows_entry("screens::draw_stage_records", r, &STAGE_ROWS)
}

/// `dispatch_rows` outside shadow mode, without its hook (typed tables, 2026-10-22: the entry the menus' and draw lists' enums run, with no hook).
pub fn rows_entry(name: &str, r: &mut Regs, rows: &Rows) -> Result<(), Refused> {
    if blanked(name) {
        return Ok(());
    }
    let too_wide = unsafe { draw_records_rows(r.edi, rows, &mut GameCanvas) }?;
    r.set_carry(too_wide);
    r.set_zero(!too_wide);
    Ok(())
}

//
// Uses the game's own fonts, strings and selected record, so it only runs when those are
// there. The blitter's hook swallows the calls; the line buffer and the kerning table are
// restored.
