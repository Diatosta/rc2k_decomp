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
use crate::menu::captions_store::CAPTIONS;
use crate::menu::event_store::EVENT_STATE;
use crate::menu::players_store::PLAYER_TABLE;
use crate::text::draw_text::Canvas;

/// The Rust version.
pub unsafe fn draw_players_standings(canvas: &mut dyn Canvas) -> Result<(), Refused> {
    unsafe {
        let rows = if EVENT_STATE.player_count.get() == 3 {
            3
        } else {
            4
        };
        for row in 0..rows {
            let p = PLAYER_TABLE.pointers[row].get();
            if p == 0
                || ((p + 0x18) as *const u32).read_unaligned() == 0
                || CAPTIONS.total_pointers[row].get() == 0
            {
                return Err(BadInput("a standings row without its player, name or points").into());
            }
        }
        // Converted: `rc2k_core::screens::players_results::standings`.
        let field = |row: usize, o: u32| {
            ((PLAYER_TABLE.pointers[row].get() + o) as *const u32).read_unaligned()
        };
        let rows: Vec<rc2k_core::screens::players_results::Standing<u32>> = (0..rows)
            .map(|row| rc2k_core::screens::players_results::Standing {
                name: field(row, 0x18),
                nation: field(row, 4),
                car: field(row, 8),
                points: (CAPTIONS.total_pointers[row].get() as *const u32).read_unaligned(),
            })
            .collect();
        rc2k_core::screens::players_results::standings(
            &mut crate::screens::game_screen::GameScreen::new(canvas),
            &rows,
        )?;
        Ok(())
    }
}
