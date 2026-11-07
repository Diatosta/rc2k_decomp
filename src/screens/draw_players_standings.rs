//! `screens::draw_players_standings` (0x483FDE): the players' standings: the heads (strings
//! 0x218, 0x219, 0x21B at y 0xE7), then four rows (three when `dword_755984` is 3: two
//! players still get four, as the original) in the order `dword_758720` .. (byte offsets into
//! `g_players`): the names at x 0x132, the cars' sprites at 0x114, the flags' at 0x1DA, the
//! places (strings 0x4FE ..) at 0xEA and the points (`off_7589A6[..]`'s first dword, kept in
//! `dword_7559B8`) at 0x217; rows 0x17 apart from y 0xFE; the sprites' palette ebx after the
//! last name.
//!
//! In:  nothing
//! Out: every register preserved (`pusha` .. `popa`)
//!
//! In a menu loop's draw list (0x758ECE). Refuses (before anything) a row's player without its
//! record or name, or without its points. The menus and draw lists run its `entry`
//! (`menu::table_calls`); the registers above are the original's, for reference.

use crate::errors::{BadInput, Refused};
use crate::menu::captions_store::CaptionTables;
use crate::menu::owner::Menus;
use crate::text::draw_text::Canvas;

/// The Rust version.
pub fn draw_players_standings(menus: &Menus, canvas: &mut dyn Canvas) -> Result<(), Refused> {
    let rows = if menus.screens.event.player_count.get() == 3 {
        3
    } else {
        4
    };
    let mut totals = [0u32; 4];
    for (row, total) in totals.iter_mut().enumerate().take(rows) {
        if menus.play.players.players[row]
            .get()
            .name_player()
            .is_none()
        {
            return Err(BadInput("a standings row without its name").into());
        }
        *total = CaptionTables::total_of(menus, row)?.get();
    }
    // Converted: `rc2k_core::screens::players_results::standings`.
    let player = |row: usize| menus.play.players.players[row].get();
    let rows: Vec<
        rc2k_core::screens::players_results::Standing<crate::screens::game_screen::HeldText>,
    > = (0..rows)
        .map(|row| {
            Ok(rc2k_core::screens::players_results::Standing {
                name: crate::screens::game_screen::player_name_of(
                    menus,
                    crate::menu::player_names::name_of(&player(row))?,
                ),
                nation: player(row).nationality,
                car: player(row).car_number,
                points: totals[row],
            })
        })
        .collect::<Result<_, BadInput>>()?;
    rc2k_core::screens::players_results::standings(
        &mut crate::screens::game_screen::GameScreen::<_, _, crate::screens::game_screen::HeldText>::new(canvas),
        &rows,
    )?;
    Ok(())
}
