//! `screens::draw_duel_result` (0x48647A): the two-player stage result: the heads (strings
//! 0x213 / 0x214, 0x20E, 0x217, 0x208, 0x210, 0x211, 0x212), the faster of players 1 and 2
//! first (`dword_758720` / `dword_758724` the two's byte offsets in `g_players`, 0 / 4 or
//! swapped when player 2's time +0x88 is below player 1's; strings 0x215 / 0x216 by it), the
//! stage's name (+0x38 of its section's first record) and stage line (string 0x482 ..) at
//! x 0x2EB, then per row the time (string 0x4FA for none, 0x36EE80; else fixed pitch at
//! 0x2AD), the car and flag sprites, the name (+0x18); by player 1 and 2 (not by order) the
//! top speeds (+0x94), the times `dword_758758` / `dword_75875C`, the strings
//! `dword_7588D2[dword_758768 - 1]` / `[dword_75876C - 1]`, and two columns of six numbers
//! (`dword_758778` .. at x 0x1C4, `dword_75877C` .. at 0x2C8).
//!
//! In:  nothing
//! Out: every register preserved (`pusha` .. `popa`)
//!
//! In a menu loop's draw list (0x758E86). The sprites' palettes are ebx as the original leaves it.
//! Refuses (before anything) players 1 or 2 missing or without a name, no stage records, a stage
//! line past the line buffer, and a string index past `dword_7588D2`'s 21 (the original reads the
//! -1 ends or past them). The menus and draw lists run its `entry` (`menu::table_calls`); the
//! registers above are the original's, for reference.

use crate::errors::{BadInput, Refused};
use crate::records::stages_store::{RecordRef, RecordTable};
use crate::text::draw_text::Canvas;

// dword_758720: byte offset in g_players
// dword_758724

/// The stage's section's first record and the stage's record (`off_735512[dword_7558C8]
/// [dword_7558D0]`, + 0xA4 x `dword_7558CC`); none: 0.
fn records() -> Result<(RecordTable, RecordRef), BadInput> {
    let base = crate::records::stages_store::STAGE_TABLES.group_event_table(
        crate::menu::owner::menus().screens.event.stage_group.get(),
        crate::menu::owner::menus().screens.event.stage_sub.get(),
    )?;
    Ok((
        base,
        crate::records::stages_store::row(
            base,
            crate::menu::owner::menus().screens.event.stage_number.get(),
        )?,
    ))
}

/// `dword_7588D2[v - 1]`, refused outside its 21 entries.
fn label(v: u32) -> Result<u32, Refused> {
    if !(1..=21).contains(&v) {
        return Err(BadInput("a string index past dword_7588D2's 21").into());
    }
    Ok(crate::menu::owner::menus().tables.captions.duel_labels[v as usize - 1].get())
}

pub fn draw_duel_result(canvas: &mut dyn Canvas) -> Result<(), Refused> {
    let (p1, p2) = (
        crate::menu::standings::player(0),
        crate::menu::standings::player(1),
    );
    if p1.get().name_player().is_none() || p2.get().name_player().is_none() {
        return Err(BadInput("player 1 or 2 without a name").into());
    }
    let (base, record) = records()?;
    let stage = crate::screens::page_settings::stage_line_bytes(0x482, record)?;
    if stage.len() > 0x100 {
        return Err(BadInput("a stage line past the line buffer").into());
    }
    let (label1, label2) = (
        label(crate::menu::owner::menus().play.results.positions[0].get())?,
        label(crate::menu::owner::menus().play.results.positions[1].get())?,
    );
    // Converted: `rc2k_core::screens::duel_result`.
    use rc2k_core::screens::player_result::Player;
    let units = crate::strings::units::units();
    let player = |p: crate::menu::standings::PlayerCell| -> Result<Player<crate::screens::game_screen::HeldText>, Refused> {
        let p = p.get();
        Ok(Player {
            name: crate::screens::game_screen::player_name_of(crate::menu::player_names::name_of(&p)?),
            nation: p.nationality,
            car: p.car_number,
            time: p.time,
            speed: crate::strings::append_speed::append_speed(p.figure, true, units)?,
        })
    };
    let players = [player(p1)?, player(p2)?];
    let figures = rc2k_core::screens::duel_result::Figures {
        labels: [label1, label2],
        best: [
            crate::menu::owner::menus().play.results.time_bonus[0].get(),
            crate::menu::owner::menus().play.results.time_bonus[1].get(),
        ],
        columns: [0, 1].map(|p| {
            [
                crate::menu::owner::menus().play.results.points[p].get(),
                crate::menu::owner::menus().play.results.time_bonus[p].get(),
                crate::menu::owner::menus().play.results.extra[p].get(),
                crate::menu::owner::menus().play.results.sums[p].get(),
                crate::menu::owner::menus().play.results.before[p].get(),
                crate::menu::owner::menus().play.results.totals[p].get(),
            ]
        }),
    };
    rc2k_core::screens::duel_result::duel_result(
        &mut crate::screens::game_screen::GameScreen::<_, _, crate::screens::game_screen::HeldText>::new(canvas),
        &players,
        base.title()?,
        &stage,
        &figures,
        crate::strings::append_time::show_hours(),
    )?;
    Ok(())
}
