//! `menu::keep_stage_best`: when the current player's `figure` is above (signed)
//! the chosen stage record's `time_trial_speed`, the record takes it as that, with the player's
//! nationality as `time_trial_nation`, the car number as `time_trial_car` and the 0x20 bytes of
//! the player's name as `time_trial_name`.
//!
//! What `figure` is is not read (a figure of the stage's; the time trial's stage page may show it).
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
