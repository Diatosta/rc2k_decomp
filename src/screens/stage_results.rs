//! The time trial stage results page (`sub_48E740`; stream A, part F1, 2026-10-18).

use crate::errors::{BadInput, Refused};
use crate::menu::event_flow::StageTable;
use crate::menu::owner::Menus;
use crate::menu::owner::Play;
use crate::menu::owner::menus;
use crate::pointers::GamePtr;
use crate::records::stages_store::{Holder, RecordRef, RecordTable};
use crate::screens::game_screen::{Held, HeldText};
use crate::session::Dword;
use crate::system::owner::system;
use crate::text::draw_text::Canvas;

/// The stage's record table and record (`off_735512[dword_7556F8][dword_75880E]`, + 0xA4 x
/// `dword_758812`), none: 0.
fn records(menus: &Menus) -> Result<(RecordTable, RecordRef), BadInput> {
    let base = system().records.stage_tables.group_event_table(
        menus.play.champ_view.switch_f8.get(),
        menus.tables.captions.shown_group.get(),
    )?;
    Ok((
        base,
        crate::records::stages_store::row(base, menus.tables.captions.shown_stage.get())?,
    ))
}

/// The times table: `off_7354FE[dword_7556F8 (+ 2 when dword_7556FC is 1)]` names the
/// variable that holds it (the list's four: `g_stageBestTimes`, `g_courseBestTimes`,
/// `dword_7556E8`, `g_stageBestTimesAlt`; their tables are typed now,
/// `saves::best_times`), or None. Refuses a list entry that names none of them.
fn times_table(play: &Play) -> Result<Option<&'_ StageTable>, BadInput> {
    let mut n = play.champ_view.switch_f8.get();
    if play.champ_view.switch_fc.get() == 1 {
        n += 2;
    }
    let p = system().records.stage_tables.result_table(n)?;
    use crate::saves::best_times as b;
    let v = &play.champ_view;
    let (_, table) = [
        (&v.stage_best_times, &b::STAGE_TIMES),
        (&v.course_best_times, &b::COURSE_TIMES),
        (&v.time_trial_best_times, &b::TIME_TRIAL_TIMES),
        (&v.stage_best_times_alt, &b::STAGE_TIMES_ALT),
    ]
    .into_iter()
    // The entry names one of the variables by its address.
    .find(|(var, _)| crate::pointers::GamePtr::to::<Dword>(var).addr() == p)
    .ok_or(BadInput(
        "a times table list entry that names no best times variable",
    ))?;
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

/// `screens::draw_stage_results` (0x48E740): the time trial stage's results: the current
/// menu's (`dword_75558D`) items 1 .. 3 (strings by the two switches, the record's +0x38), the
/// record's +4 into `dword_A8C154`, the stage line (string 0x481 .. in `byte_A8BD54`), then up
/// to ten rows of name, result and rank with the car's and flag's sprites: in seal mode
/// (`dword_75AB30` 1) the row's seal code (`strings::format_seal_code` of +0x54 / +0x58) one
/// character at a time from x 0x20A every 0xE; else its time (fixed pitch at 0x276), the first
/// kept in `dword_75AB24` and the others' gap to it (string 0x579 and the time at 0x2EE); after
/// ten rows the fastest line (string 0x1A2; from the stage record found again by
/// `records::find_stage_record` when both switches are 0, else the record's own, its second
/// fastest with `dword_7556FC` 1). The sprites' palettes are ebx as the original leaves it.
/// Every register kept. Refuses (before anything) no current menu, no stage records or times
/// table, a stage line past its buffer, a times table whose end (-1) comes within the ten rows
/// (the original's early exit leaves two dwords on the stack and returns through a `pusha`
/// slot), and what the strings refuse.
pub fn draw_stage_results(canvas: &mut dyn Canvas) -> Result<(), Refused> {
    let menus = menus();
    let play = &menus.play;
    let menu = menus.screens.loop_flags.menu.get().ok();
    let (base, record) = records(menus)?;
    let table = times_table(play)?;
    let (Some(table), Some(menu)) = (table, menu) else {
        return Err(BadInput("no current menu, stage records or times table").into());
    };
    let both_off =
        (menus.play.champ_view.switch_f8.get() ^ 1) & (menus.play.champ_view.switch_fc.get() ^ 1);
    let fastest = if both_off == 1 {
        crate::records::find_stage_record::find_stage_record(
            system().records.stage_tables.championship(),
            menus.tables.captions.shown_group.get(),
            menus.tables.captions.shown_stage.get(),
        )?
    } else {
        record
    };
    let holder = if both_off != 1 && menus.play.champ_view.switch_fc.get() == 1 {
        Holder::Second
    } else {
        Holder::Fastest
    };
    let mut speed = crate::strings::append_speed::append_speed(
        fastest.record()?.holder(holder).speed,
        true,
        crate::strings::units::units(),
    )?;
    speed.push(0);
    let stage = crate::screens::page_settings::stage_line_bytes(0x481, record)?;
    if stage.len() > 0x100 {
        return Err(BadInput("a stage line past its buffer").into());
    }
    let slot = record.record()?.event_slot() as usize;
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
        menu.items()
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
    set_item(3, base.title()?)?;
    crate::screens::text_buffers_store::TEXT_BUFFERS
        .stage_result_value
        .set(record.record()?.track());
    crate::pointers::GamePtr::to(&crate::screens::text_buffers_store::TEXT_BUFFERS.stage_heading)
        .write_bytes(&stage, "the stage line")?;
    // Converted: `rc2k_core::screens::stage_results`.
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
    let held = fastest.record()?.holder(holder);
    let fastest = Fastest {
        nation: held.nation,
        name: HeldText::Place(fastest.text(holder.name_at())),
        car: held.car,
        speed,
    };
    let seal_fault = crate::menu::event_flow::Fault::default();
    let mut seal_code = |a: u32, b: u32| {
        seal_fault.ok(crate::strings::format_seal_code::format_seal_code(a, b));
        seal_fault.ok(GamePtr::to(&crate::strings::format_seal_code::SEAL_TEXT)
            .bytes("the seal text")
            .and_then(|text| crate::strings::text_length::nul_at(&text))
            .and_then(|len| {
                GamePtr::to(&crate::strings::format_seal_code::SEAL_TEXT)
                    .read_bytes(len as usize, "the seal text")
            }))
    };
    let drawn = stage_results(
        &mut crate::screens::game_screen::GameScreen::<_, _, HeldText>::new(canvas),
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
