//! The time trial stage results page (`screens::draw_stage_results`).

use crate::errors::{BadInput, Refused};
use crate::menu::event_flow::StageTable;
use crate::menu::owner::Menus;
use crate::menu::owner::Play;
use crate::owners::Owners;
use crate::records::stages_store::{Holder, RecordRef, RecordTable};
use crate::screens::game_screen::{Held, HeldText};
use crate::screens::text_buffers_store::text_buffers;
use crate::text::draw_text::Canvas;

/// The stage's record table and record (by `ChampView.switch_f8`, `CaptionTables.shown_group` and
/// `CaptionTables.shown_stage`), none: 0.
fn records(
    menus: &Menus,
    stages: &crate::records::stages_store::StageTables,
) -> Result<(RecordTable, RecordRef), BadInput> {
    let base = stages.group_event_table(
        menus.play.champ_view.switch_f8.get(),
        menus.tables.captions.shown_group.get(),
    )?;
    Ok((
        base,
        crate::records::stages_store::row(stages, base, menus.tables.captions.shown_stage.get())?,
    ))
}

/// The times table: `ChampView.switch_f8` (+ 2 when `ChampView.switch_fc` is 1) names which of
/// the four holds it (the stage, course, time trial and alternate best times; see
/// `saves::best_times`), or None. Refuses a list entry that names none of them.
fn times_table<'a>(
    play: &'a Play,
    stages: &crate::records::stages_store::StageTables,
) -> Result<Option<&'a StageTable>, BadInput> {
    let mut n = play.champ_view.switch_f8.get();
    if play.champ_view.switch_fc.get() == 1 {
        n += 2;
    }
    use crate::menu::champ_store::TimesVariable;
    use crate::saves::best_times as b;
    let table = match stages.result_table(n)? {
        TimesVariable::StageTimes => &b::STAGE_TIMES,
        TimesVariable::CourseTimes => &b::COURSE_TIMES,
        TimesVariable::TimeTrialTimes => &b::TIME_TRIAL_TIMES,
        TimesVariable::StageTimesAlt => &b::STAGE_TIMES_ALT,
    };
    Ok(table.is_taken().then_some(table))
}

/// Whether the end mark comes within the ten rows (n = 1 .. 10 entries on: the tenth is
/// the next stage's first, or the block's zeros past the table). Refuses one past the
/// block.
fn ends_within_ten(table: &StageTable, stage: usize) -> Result<bool, BadInput> {
    table.with(|t| {
        for n in 1..=10 {
            let first = t.first_at(stage * 10 + n).ok_or(BadInput(
                "an entry after the ten past the best times' block",
            ))?;
            if first == u32::MAX {
                return Ok(true);
            }
        }
        Ok(false)
    })?
}

/// The time trial stage's results: the current menu's items 1 to 3 (strings by the two
/// switches, the record's name), the record's value into `Buffers.text.stage_result_value`, the
/// stage line (string 0x481 .. in `Buffers.text.stage_heading`), then up to ten rows of name,
/// result and rank with the car's and flag's sprites: in seal mode
/// (`MenuState.menus.caption_choice` 1) the row's seal code (`strings::format_seal_code`) one
/// character at a time; else its time (fixed pitch), the first kept in
/// `MenuState.menus.first_row_time` and the others' gap to it (string 0x579 and the time); after
/// ten rows the fastest line (string 0x1A2; from the stage record found again by
/// `records::find_stage_record` when both switches are 0, else the record's own, its second
/// fastest with `ChampView.switch_fc` 1). The sprites' palette is unused. Refuses no current
/// menu, no stage records or times table, a stage line past its buffer, a times table whose end
/// (-1) comes within the ten rows and what the strings refuse.
pub fn draw_stage_results(o: Owners<'static>, canvas: &mut dyn Canvas) -> Result<(), Refused> {
    let menus = o.menus;
    let play = &menus.play;
    let menu = menus.screens.loop_flags.menu.get().ok();
    let stages = &o.system.records().stage_tables;
    let (base, record) = records(menus, stages)?;
    let table = times_table(play, stages)?;
    let (Some(table), Some(menu)) = (table, menu) else {
        return Err(BadInput("no current menu, stage records or times table").into());
    };
    let both_off =
        (menus.play.champ_view.switch_f8.get() ^ 1) & (menus.play.champ_view.switch_fc.get() ^ 1);
    let fastest = if both_off == 1 {
        crate::records::find_stage_record::find_stage_record(
            &o.system.records().stage_tables,
            stages.championship(),
            menus.tables.captions.shown_group.get(),
            menus.tables.captions.shown_stage.get(),
        )?
    } else {
        record
    };
    let holder = if both_off != 1 && menus.play.champ_view.switch_fc.get() == 1 {
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
        crate::strings::units::units(),
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
    let names = ten
        .iter()
        .map(|e| e.name_text().and_then(|t| Held::new(&t)))
        .collect::<Option<Vec<_>>>()
        .ok_or(BadInput("a best time's name without its end"))?;
    if ends_within_ten(table, slot)? {
        return Err(BadInput(
            "a times table ending within the ten rows: the original returns through a wrong stack",
        )
        .into());
    }
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
        if menus.play.champ_view.switch_f8.get() == 1 {
            0x19C
        } else {
            0x19B
        },
    )?;
    set_item(
        2,
        if menus.play.champ_view.switch_fc.get() == 1 {
            0x19E
        } else {
            0x19D
        },
    )?;
    set_item(3, base.title(&o.system.records().stage_tables)?)?;
    text_buffers()
        .stage_result_value
        .set(record.record(&o.system.records().stage_tables)?.track());
    text_buffers()
        .stage_heading
        .write_at(0, &stage, "the stage line")?;
    // The logic is in `rc2k_core::screens::stage_results`.
    use rc2k_core::screens::stage_results::{Entry, Fastest, stage_results};
    let rows = std::array::from_fn(|n| {
        let e = &ten[n];
        Entry {
            name: HeldText::Held(names[n]),
            nation: e.head.nation,
            car: e.head.car,
            time: e.head.key,
            seal: (e.sealed[0], e.sealed[1]),
        }
    });
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
    let seal_fault = crate::menu::event_flow::Fault::default();
    let mut seal_code = |a: u32, b: u32| {
        seal_fault.ok(crate::strings::format_seal_code::format_seal_code(
            o.race, a, b,
        ));
        let text = crate::strings::format_seal_code::SEAL_TEXT.get().read();
        seal_fault
            .ok(crate::strings::text_length::nul_at(&text).map(|len| text[..len as usize].to_vec()))
    };
    let drawn = stage_results(
        &mut crate::screens::game_screen::GameScreen::<_, _, HeldText>::new(canvas, o),
        menus.screens.state.menus.caption_choice.get() == 1,
        crate::strings::append_time::show_hours(),
        &rows,
        &mut seal_code,
        &fastest,
    );
    seal_fault.check()?;
    drawn?;
    Ok(())
}
