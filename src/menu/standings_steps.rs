//! The event drivers' steps on the two standings tables (A: the rally's, B: the
//! championship's; see `menu::standings`): clearing a table's figures and numbering its
//! entries. Each refuses a table without its end entry (or a player past 4 or missing)
//! before changing anything. Originals: `clear_standings_a` 0x4A5A14, `_b` 0x4A5A2F;
//! `number_all_a` 0x4A2090, `_b` 0x4A5A98; `number_standings_a` 0x4A2055, `_b` 0x4A4A9D.

use crate::errors::Refused;
use crate::menu::owner::menus;
use crate::menu::standings::*;
use rc2k_core::menu::standings::Numbering;

/// Gives every entry of `table` a figure of 0 (table A, entry +0x44) or a position of 0
/// (table B, +0x70).
pub fn clear_standings(table: Which) -> Result<(), Refused> {
    match table {
        Which::A => {
            let n = walked_a()?;
            with_a(|t| t.entries[..n].iter_mut().for_each(|e| e.head.figure = 0))?;
        }
        Which::B => {
            let n = walked_b()?;
            with_b(|t| t.entries[..n].iter_mut().for_each(|e| e.position = 0))?;
        }
    }
    Ok(())
}

/// Numbers the entries of `table` 1, 2, .. in order (with the class mode 2 only those with
/// a group), each player's number also into its record. Table A's count goes into
/// `EVENT_STATE.count_a` (`dword_75597C`).
pub fn number_all(table: Which) -> Result<Numbered, Refused> {
    let screens = &menus().screens;
    let grouped = screens.state.menus.class_mode.get() == 2;
    let numbering = Numbering::All { grouped };
    let numbered = match table {
        Which::A => number_table(table, numbering, |p| &mut p.number_all_a)?,
        Which::B => number_table(table, numbering, |p| &mut p.number_all_b)?,
    };
    if table == Which::A {
        screens.event.count_a.set(numbered.last_number);
    }
    Ok(numbered)
}

/// Numbers the entries of group `class` of `table` 1, 2, .. in order, each player's number
/// also into its record.
pub fn number_standings(table: Which, class: u32) -> Result<Numbered, Refused> {
    let numbering = Numbering::Group(class);
    match table {
        Which::A => number_table(table, numbering, |p| &mut p.number_standings_a),
        Which::B => number_table(table, numbering, |p| &mut p.number_standings_b),
    }
}
