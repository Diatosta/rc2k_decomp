//! The two-player stage result: the headings, the faster of players 1 and 2 first, the stage's
//! name and line, per row the time, the car and flag sprites and the name; then per player the top
//! speeds, the time bonuses, their labels and two columns of six figures from the results.
//!
//! Refused: players 1 or 2 missing or without a name, no stage records, a stage line past the line
//! buffer and a string index past the duel labels' 21.

use crate::errors::{BadInput, Refused};
use crate::menu::owner::Screens;
use crate::menu::owner::Tables;
use crate::menu::results_store::Results;
use crate::records::stages_store::{RecordRef, RecordTable};
use crate::text::draw_text::Canvas;

/// The stage's section's first record and the stage's record (the event's stage table for
/// the chosen group and sub-group, + 0xA4 x the stage number); none: 0.
fn records(
    screens: &Screens,
    stages: &crate::records::stages_store::StageTables,
) -> Result<(RecordTable, RecordRef), BadInput> {
    let base = stages.group_event_table(
        screens.event.stage_group.get(),
        screens.event.stage_sub.get(),
    )?;
    Ok((
        base,
        crate::records::stages_store::row(stages, base, screens.event.stage_number.get())?,
    ))
}

/// `captions.duel_labels[v - 1]`, refused outside its 21 entries.
fn label(tables: &Tables, v: u32) -> Result<u32, Refused> {
    if !(1..=21).contains(&v) {
        return Err(BadInput("a string index past the duel labels' 21").into());
    }
    Ok(tables.captions.duel_labels[v as usize - 1].get())
}

pub fn draw_duel_result(
    o: crate::owners::Owners<'static>,
    system: &crate::system::owner::System,
    results: &Results,
    canvas: &mut dyn Canvas,
) -> Result<(), Refused> {
    let menus = o.menus;
    let screens = &menus.screens;
    let tables = &menus.tables;
    let (p1, p2) = (
        crate::menu::standings::player(menus, 0),
        crate::menu::standings::player(menus, 1),
    );
    if p1.get().name_player().is_none() || p2.get().name_player().is_none() {
        return Err(BadInput("player 1 or 2 without a name").into());
    }
    let (base, record) = records(screens, &system.records().stage_tables)?;
    let stage = crate::screens::page_settings::stage_line_bytes(
        crate::strings::append_string_by_id::StringOwners::of(o),
        0x482,
        record,
    )?;
    if stage.len() > 0x100 {
        return Err(BadInput("a stage line past the line buffer").into());
    }
    let (label1, label2) = (
        label(tables, results.players[0].position.get())?,
        label(tables, results.players[1].position.get())?,
    );
    // The logic is in `rc2k_core::screens::duel_result`.
    use rc2k_core::screens::player_result::Player;
    let units = crate::strings::units::units();
    let player = |p: crate::menu::standings::PlayerCell| -> Result<Player<crate::screens::game_screen::HeldText>, Refused> {
        let p = p.get();
        Ok(Player {
            name: crate::screens::game_screen::player_name_of(crate::menu::player_names::name_of(&p)?),
            nation: p.nationality,
            car: p.car_number,
            time: p.time,
            speed: crate::strings::append_speed::append_speed(crate::strings::append_string_by_id::StringOwners::of(o), p.figure, true, units)?,
        })
    };
    let players = [player(p1)?, player(p2)?];
    let figures = rc2k_core::screens::duel_result::Figures {
        labels: [label1, label2],
        best: [
            results.players[0].time_bonus.get(),
            results.players[1].time_bonus.get(),
        ],
        columns: [0, 1].map(|p| results.players[p].get().column()),
    };
    rc2k_core::screens::duel_result::duel_result(
        &mut crate::screens::game_screen::GameScreen::<_, _, crate::screens::game_screen::HeldText>::new(canvas, o),
        &players,
        base.title(&system.records().stage_tables)?,
        &stage,
        &figures,
        crate::strings::append_time::show_hours(),
    )?;
    Ok(())
}
