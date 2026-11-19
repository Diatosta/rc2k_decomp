//! The time trial stage results page (`screens::draw_stage_results`).

use crate::errors::{BadInput, Refused};
use crate::menu::champ_store::ChampView;
use crate::menu::event_flow::StageTable;
use crate::owners::Owners;
use crate::records::stages_store::{Holder, RecordRef, RecordTable};
use crate::screens::game_screen::{Held, HeldText};
use crate::text::draw_text::Canvas;

/// The stage's record table and record (by `ChampView.switches.records_set`,
/// `ChampView.shown_stage.group` and `ChampView.shown_stage.number`), none: 0.
fn records(
    champ: &ChampView,
    stages: &crate::records::stages_store::StageTables,
) -> Result<(RecordTable, RecordRef), BadInput> {
    let base = stages.group_event_table(
        champ.switches.records_set.get(),
        champ.shown_stage.group.get(),
    )?;
    Ok((
        base,
        crate::records::stages_store::row(stages, base, champ.shown_stage.number.get())?,
    ))
}

/// The times table: `ChampView.switches.records_set` (+ 2 when `ChampView.switches.records_alt`
/// is 1) names which of the four holds it (the stage, course, time trial and alternate best
/// times; see
/// `saves::best_times::BestTables`), or None. Refuses a list entry that names none of them.
fn times_table<'t>(
    champ: &ChampView,
    stages: &crate::records::stages_store::StageTables,
    tables: &'t crate::saves::best_times::BestTables,
) -> Result<Option<&'t StageTable>, BadInput> {
    let mut n = champ.switches.records_set.get();
    if champ.switches.records_alt.get() == 1 {
        n += 2;
    }
    use crate::menu::champ_store::TimesVariable;
    let table = match stages.result_table(n)? {
        TimesVariable::StageTimes => &tables.stage_times,
        TimesVariable::CourseTimes => &tables.course_times,
        TimesVariable::TimeTrialTimes => &tables.time_trial_times,
        TimesVariable::StageTimesAlt => &tables.stage_times_alt,
    };
    Ok(table.is_taken().then_some(table))
}

/// How many rows the stage's list has: up to the first entry (1 .. 10 on: the tenth is the next
/// stage's first, or the block's zeros past the table) that is the end mark, at most ten.
/// Refuses one past the block.
fn rows_of(table: &StageTable, stage: usize) -> Result<usize, BadInput> {
    table.with(|t| {
        for n in 1..=10 {
            let first = t.first_at(stage * 10 + n).ok_or(BadInput(
                "an entry after the ten past the best times' block",
            ))?;
            if first == u32::MAX {
                return Ok(n);
            }
        }
        Ok(10)
    })?
}

/// The time trial stage's results: the current menu's items 1 to 3 (strings by the two
/// switches, the record's name), the record's value into `Buffers.text.stage_result_value`, the
/// stage line (string 0x481 .. in `Buffers.text.stage_heading`), then up to ten rows of name,
/// result and rank with the car's and flag's sprites: in seal mode
/// (`MenuState.menus.caption_choice` 1) the row's seal code (`strings::format_seal_code`) one
/// character at a time; else its time (fixed pitch), the first kept in
/// `MenuState.menus.first_row_time` and the others' gap to it (string 0x579 and the time); after
/// the rows (ten, or fewer when the list ends) the fastest line (string 0x1A2; from the stage record found again by
/// `records::find_stage_record` when both switches are 0, else the record's own, its second
/// fastest with `ChampView.switches.records_alt` 1). The sprites' palette is unused. Refuses no
/// current menu, no stage records or times table, a stage line past its buffer and what the
/// strings refuse.
pub fn draw_stage_results(o: Owners<'static>, canvas: &mut dyn Canvas) -> Result<(), Refused> {
    let menus = o.menus;
    let champ = &menus.play.champ_view;
    let menu = menus.screens.loop_flags.menu.get().ok();
    let stages = &o.system.records().stage_tables;
    let (base, record) = records(champ, stages)?;
    let table = times_table(champ, stages, &o.saves.best_tables)?;
    let (Some(table), Some(menu)) = (table, menu) else {
        return Err(BadInput("no current menu, stage records or times table").into());
    };
    let both_off = (champ.switches.records_set.get() ^ 1) & (champ.switches.records_alt.get() ^ 1);
    let fastest = if both_off == 1 {
        crate::records::find_stage_record::find_stage_record(
            &o.system.records().stage_tables,
            stages.championship(),
            champ.shown_stage.group.get(),
            champ.shown_stage.number.get(),
        )?
    } else {
        record
    };
    let holder = if both_off != 1 && champ.switches.records_alt.get() == 1 {
        Holder::TimeTrial
    } else {
        Holder::Fastest
    };
    let mut speed = crate::strings::append_speed::append_speed(
        crate::strings::append_string_by_id::StringOwners::of(o),
        fastest
            .record(&o.system.records().stage_tables)?
            .holder(holder)
            .speed,
        true,
        crate::strings::units::units(o.strings),
    )?;
    speed.push(0);
    let stage = crate::screens::page_settings::stage_line_bytes(
        crate::strings::append_string_by_id::StringOwners::of(o),
        0x481,
        record,
    )?;
    if stage.len() > 0x100 {
        return Err(BadInput("a stage line past its buffer").into());
    }
    let slot = record
        .record(&o.system.records().stage_tables)?
        .event_slot() as usize;
    let ten = table
        .with(|t| t.stages.get(slot).copied())?
        .ok_or(BadInput("a stage past its best times table"))?;
    let count = rows_of(table, slot)?;
    let names = ten[..count]
        .iter()
        .map(|e| e.name_text().and_then(|t| Held::new(&t)))
        .collect::<Option<Vec<_>>>()
        .ok_or(BadInput("a best time's name without its end"))?;
    let set_item = |n: u32, v: u32| -> Result<(), BadInput> {
        menu.items(&menus.tables.menus)
            .get(n as usize)
            .ok_or(BadInput("an item past its menu's end"))?
            .string
            .set(v);
        Ok(())
    };
    set_item(
        1,
        if champ.switches.records_set.get() == 1 {
            0x19C
        } else {
            0x19B
        },
    )?;
    set_item(
        2,
        if champ.switches.records_alt.get() == 1 {
            0x19E
        } else {
            0x19D
        },
    )?;
    set_item(3, base.title(&o.system.records().stage_tables)?)?;
    menus
        .text_buffers
        .stage_result_value
        .set(record.record(&o.system.records().stage_tables)?.track());
    menus
        .text_buffers
        .stage_heading
        .write_at(0, &stage, "the stage line")?;
    // The logic is in `rc2k_core::screens::stage_results`.
    use rc2k_core::screens::stage_results::{Entry, Fastest, stage_results};
    let rows: Vec<_> = (0..count)
        .map(|n| {
            let e = &ten[n];
            Entry {
                name: HeldText::Held(names[n]),
                nation: e.head.nation,
                car: e.head.car,
                time: e.head.key,
                seal: (e.sealed[0], e.sealed[1]),
            }
        })
        .collect();
    speed.pop();
    let held = fastest
        .record(&o.system.records().stage_tables)?
        .holder(holder);
    let fastest = Fastest {
        nation: held.nation,
        name: HeldText::Held(fastest.holder_text(&o.system.records().stage_tables, holder)?),
        car: held.car,
        speed,
    };
    let seal_fault = crate::errors::Fault::default();
    let mut seal_code = |a: u32, b: u32| {
        let seal_work = o.race.replay.seal_work();
        seal_work.sealed[0].set(a);
        seal_work.sealed[1].set(b);
        seal_fault.ok(crate::strings::format_seal_code::format_seal_code(
            o.strings, a, b,
        ));
        let text = o.strings.text.seal_text.read();
        seal_fault
            .ok(crate::strings::text_length::nul_at(&text).map(|len| text[..len as usize].to_vec()))
    };
    let drawn = stage_results(
        &mut crate::screens::game_screen::GameScreen::<_, _, HeldText>::new(canvas, o),
        menus.screens.state.menus.caption_choice.get() == 1,
        crate::strings::append_time::show_hours(o.strings),
        &rows,
        &mut seal_code,
        &fastest,
    );
    seal_fault.check()?;
    drawn?;
    Ok(())
}
