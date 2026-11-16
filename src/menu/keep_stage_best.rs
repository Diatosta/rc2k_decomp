//! `menu::keep_stage_best`: when the current player's +0x94 is above (signed)
//! the chosen stage record's +0x2C, the record takes it: +0x2C that, +0x9C the player's
//! nationality (+4), +0xA0 the car (+8), and the 0x20 bytes of the player's name (at
//! +0x18) at +0x7C.
//!
//! What +0x94 is is not read (a figure of the stage's; the time trial's stage page may show it).
//! Refuses (before anything) a current player past the four or missing, a stage the tables do not
//! give (`event_flow::stage_record`), or a name pointer of 0 when the record is taken.

use crate::errors::Refused;
use crate::menu::event_flow::*;
use crate::menu::owner::Menus;
use crate::menu::standings::name_player;
use crate::records::stages_store::Holder;

pub fn keep_stage_best(
    menus: &Menus,
    system: &crate::system::owner::System,
) -> Result<(), Refused> {
    use rc2k_core::menu::bests;
    let record = stage_record(menus, system)?;
    let p = current_player(menus)?;
    let figure = p.get().figure;
    if !bests::takes_record(
        figure,
        record
            .record(&system.records().stage_tables)?
            .holder(Holder::TimeTrial)
            .speed,
    ) {
        return Ok(());
    }
    name_player(&p.get())?;
    let (row, name) = crate::menu::standings::as_holder(&menus.play, &p.get())?;
    record
        .record(&system.records().stage_tables)?
        .set_holder(Holder::TimeTrial, &row, &name);
    Ok(())
}
