//! `screens::draw_player_rally_result` (0x4894AD): the player's result in a rally stage: the
//! labels (string 0x213 at (0xED, 0xE7), strings 0x20E .. 0x212 at x 0x155), the stage's
//! heading (`screens::draw_rally_times_heading`'s), the player's time (+0x88; string 0x513
//! for none, 0x980000) and top speed (+0x94), the rally's numbers and times
//! (`dword_758758` .. `dword_7587B8`), the car's sprite, the name and the flag's sprite (their
//! palettes ebx as the original leaves it: the last text past its NUL).
//!
//! In:  nothing
//! Out: every register preserved (`pusha` .. `popa`)
//!
//! In a menu loop's draw list (0x758E76). Refuses (before anything) no current player or name, no
//! rally stage records, and what `strings::append_speed` refuses. The menus and draw lists run its
//! `entry` (`menu::table_calls`); the registers above are the original's, for reference.

use crate::errors::Refused;
use crate::framework::{game_read, rva};
use crate::screens::best_scores::{RALLY_STAGE_TIMES, draw_times_heading};
use crate::text::draw_text::Canvas;

fn k(a: usize) -> u32 {
    unsafe { game_read(rva(a)) }
}

/// (id, x, y, font) of the labels.
/// The Rust version.
pub unsafe fn draw_player_rally_result(canvas: &mut dyn Canvas) -> Result<(), Refused> {
    unsafe {
        let p = crate::menu::player_settings::current()? as u32;
        let f = |o: u32| ((p + o) as *const u32).read_unaligned();
        let name = crate::menu::player_name::name()?;
        if (RALLY_STAGE_TIMES.records)() == 0 {
            return Err(crate::errors::BadInput("no rally stage records").into());
        }
        let speed = crate::strings::append_speed::append_speed(
            f(0x94),
            true,
            crate::strings::units::units(),
        )?;
        // Converted: `rc2k_core::screens::player_result::rally_result`.
        use crate::screens::game_screen::PlayerResultScreen;
        let p = rc2k_core::screens::player_result::Player {
            name: name as u32,
            nation: f(4),
            car: f(8),
            time: f(0x88),
            speed,
        };
        let figures = rc2k_core::screens::player_result::RallyFigures {
            position: k(0x758768),
            best: k(0x758758),
            column: [0x758778, 0x758758, 0x758788, 0x758798, 0x7587A8, 0x7587B8].map(k),
        };
        rc2k_core::screens::player_result::rally_result(
            &mut PlayerResultScreen::new(canvas),
            &mut |s: &mut PlayerResultScreen| draw_times_heading(&RALLY_STAGE_TIMES, s.canvas),
            &p,
            &figures,
            crate::strings::append_time::show_hours(),
        )?;
        Ok(())
    }
}
