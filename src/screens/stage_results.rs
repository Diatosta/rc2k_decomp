//! The time trial stage results page (`sub_48E740`; stream A, part F1, 2026-10-18).

use crate::errors::{BadInput, Refused};
use crate::menu::captions_store::CAPTIONS;
use crate::menu::champ_store::CHAMP_VIEW;
use crate::menu::event_flow::StageTable;
use crate::menu::loop_store::LOOP_FLAGS;
use crate::menu::state_store::MENU_STATE;
use crate::pointers::GamePtr;
use crate::screens::game_screen::{Held, HeldText};
use crate::session::Dword;
use crate::text::draw_text::Canvas;

fn get(a: u32) -> Result<u32, crate::errors::BadInput> {
    crate::pointers::GamePtr::new(a).read::<u32>("a game value")
}

/// The stage's record table and record (`off_735512[dword_7556F8][dword_75880E]`, + 0xA4 x
/// `dword_758812`), none: 0.
fn records() -> Result<(u32, u32), BadInput> {
    let table = crate::records::stages_store::STAGE_TABLES.group_list(CHAMP_VIEW.switch_f8.get());
    if table == 0 {
        return Ok((0, 0));
    }
    let base = get(table + 4 * CAPTIONS.shown_group.get())?;
    Ok((
        base,
        base.wrapping_add(CAPTIONS.shown_stage.get().wrapping_mul(0xA4)),
    ))
}

/// The times table: `off_7354FE[dword_7556F8 (+ 2 when dword_7556FC is 1)]` names the
/// variable that holds it (the list's four: `g_stageBestTimes`, `g_courseBestTimes`,
/// `dword_7556E8`, `g_stageBestTimesAlt`; their tables are typed now,
/// `saves::best_times`), or None. Refuses a list entry that names none of them.
fn times_table() -> Result<Option<&'static StageTable>, BadInput> {
    let mut at = crate::records::stages_store::STAGE_TABLES.result_tables_addr()
        + 4 * CHAMP_VIEW.switch_f8.get();
    if CHAMP_VIEW.switch_fc.get() == 1 {
        at += 8;
    }
    let p = get(at)?;
    if p == 0 {
        return Ok(None);
    }
    use crate::saves::best_times as b;
    let v = &CHAMP_VIEW;
    let (_, table) = [
        (&v.stage_best_times, &b::STAGE_TIMES),
        (&v.course_best_times, &b::COURSE_TIMES),
        (&v.time_trial_best_times, &b::TIME_TRIAL_TIMES),
        (&v.stage_best_times_alt, &b::STAGE_TIMES_ALT),
    ]
    .into_iter()
    .find(|(var, _)| crate::pointers::game_address(*var as *const Dword) == p)
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
    let menu = LOOP_FLAGS.menu.get().ok();
    let (base, record) = records()?;
    let table = times_table()?;
    let (Some(table), Some(menu)) = (table.filter(|_| base != 0), menu) else {
        return Err(BadInput("no current menu, stage records or times table").into());
    };
    let both_off = (CHAMP_VIEW.switch_f8.get() ^ 1) & (CHAMP_VIEW.switch_fc.get() ^ 1);
    let fastest = if both_off == 1 {
        crate::records::find_stage_record::find_stage_record(
            crate::records::stages_store::STAGE_TABLES.championship(),
            CAPTIONS.shown_group.get(),
            CAPTIONS.shown_stage.get(),
        )?
    } else {
        record
    };
    let second = both_off != 1 && CHAMP_VIEW.switch_fc.get() == 1;
    let speed_at = if second { 0x2C } else { 0x30 };
    let mut speed = crate::strings::append_speed::append_speed(
        get(fastest + speed_at)?,
        true,
        crate::strings::units::units(),
    )?;
    speed.push(0);
    let stage = crate::screens::page_settings::stage_line_bytes(0x481, record)?;
    if stage.len() > 0x100 {
        return Err(BadInput("a stage line past its buffer").into());
    }
    let slot = get(record + 0x3C)? as usize;
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
        if CHAMP_VIEW.switch_f8.get() == 1 {
            0x19C
        } else {
            0x19B
        },
    )?;
    set_item(
        2,
        if CHAMP_VIEW.switch_fc.get() == 1 {
            0x19E
        } else {
            0x19D
        },
    )?;
    set_item(3, get(base + 0x38)?)?;
    crate::screens::text_buffers_store::TEXT_BUFFERS
        .stage_result_value
        .set(get(record + 4)?);
    crate::pointers::GamePtr::new(
        crate::screens::text_buffers_store::TEXT_BUFFERS
            .stage_heading
            .addr(),
    )
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
    let (nation, name, car) = if second {
        (0x9C, 0x7C, 0xA0)
    } else {
        (0x74, 0x54, 0x78)
    };
    speed.pop();
    let fastest = Fastest {
        nation: get(fastest + nation)?,
        name: HeldText::Game(fastest + name),
        car: get(fastest + car)?,
        palette: if both_off == 1 {
            CAPTIONS.shown_group.get()
        } else {
            both_off
        },
        speed,
    };
    let seal_fault = crate::menu::event_flow::Fault::default();
    let mut seal_code = |a: u32, b: u32| {
        seal_fault.ok(crate::strings::format_seal_code::format_seal_code(a, b));
        seal_fault.ok(crate::strings::text_length::nul_at(
            crate::strings::format_seal_code::SEAL_TEXT.addr(),
            "the seal text",
        )
        .and_then(|len| {
            GamePtr::new(crate::strings::format_seal_code::SEAL_TEXT.addr())
                .read_bytes(len as usize, "the seal text")
        }))
    };
    let drawn = stage_results(
        &mut crate::screens::game_screen::GameScreen::<_, _, HeldText>::new(canvas),
        MENU_STATE.menus.caption_choice.get() == 1,
        crate::strings::append_time::show_hours(),
        &rows,
        &mut seal_code,
        &fastest,
    );
    seal_fault.check()?;
    drawn?;
    Ok(())
}
