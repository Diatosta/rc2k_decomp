//! `menu::keep_stage_best` (0x48E02D): when the current player's +0x94 is above (signed)
//! the chosen stage record's +0x2C, the record takes it: +0x2C that, +0x9C the player's
//! nationality (+4), +0xA0 the car (+8), and the 0x20 bytes of the player's name (at
//! +0x18) at +0x7C.
//!
//! In:  nothing
//! Out: every register preserved (`pusha` .. `popa`)
//!
//! Callers: the time trial's events, after the results. What +0x94 is is not read (a figure of the
//! stage's; the time trial's stage page may show it). Refuses (before anything) a current player
//! past the four or missing, a stage the tables do not give (`event_flow::stage_record`), or a name
//! pointer of 0 when the record is taken.

use crate::errors::{BadInput, Refused};
use crate::menu::event_flow::*;

pub fn keep_stage_best() -> Result<(), Refused> {
    use rc2k_core::menu::bests;
    let record = stage_record()?;
    let p = current_player()?;
    // Converted: `rc2k_core::menu::bests::{takes_record, take_record}`.
    let figure = dword(p + 0x94)?;
    if !bests::takes_record(figure, dword(record + 0x2C)?) {
        return Ok(());
    }
    let name = dword(p + 0x18)?;
    if name == 0 {
        return Err(BadInput("a player's name pointer of 0").into());
    }
    let holder = bests::Holder {
        figure,
        nationality: dword(p + 4)?,
        car: dword(p + 8)?,
        name: crate::menu::standings::bytes(name)?,
    };
    let mut r = crate::menu::standings::table(record, STAGE_RECORD, 1)?;
    bests::take_record(&mut r, 0, &bests::STAGE_HOLDER, &holder);
    Ok(())
}
