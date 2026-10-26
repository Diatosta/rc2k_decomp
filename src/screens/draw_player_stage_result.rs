//! `screens::draw_player_stage_result` (0x486299): the player's result on a championship
//! stage: the three labels (strings 0x206 .. 0x208 at x 0x1EC, font 0x51E6E4), the stage's
//! heading (`screens::draw_stage_times_heading`'s), the player's time (+0x88) and top speed
//! (+0x94, with its unit) at x 0x1FE, the car's sprite, the name (+0x18) at x 0x221 and the
//! flag's sprite after it (its width by `text::measure_text` in the font at 0x51E624); the
//! sprites' palette ebx as the original leaves it (the speed line past its NUL, then the
//! measure's line count).
//!
//! In:  nothing
//! Out: every register preserved (`pusha` .. `popa`)
//!
//! In a menu loop's draw list (0x758E7E). Refuses (before anything) no current player or name,
//! no stage records, and what `strings::append_speed` refuses. Hook removed 2026-10-22 (typed
//! tables): the menus and draw lists run its `entry` (`menu::table_calls`); the registers above
//! are the original's, for reference.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::screens::best_scores::{STAGE_TIMES, draw_times_heading};
use crate::screens::harness::{Left, draw_entry};
use crate::text::draw_text::Canvas;

/// The Rust version.
pub unsafe fn draw_player_stage_result(canvas: &mut dyn Canvas) -> Result<(), Refused> {
    unsafe {
        let player = crate::menu::player_settings::current()?;
        let p = player as u32;
        let f = |o: u32| ((p + o) as *const u32).read_unaligned();
        let name = crate::menu::player_name::name()?;
        let mut speed = crate::strings::append_speed::append_speed(
            f(0x94),
            true,
            crate::strings::units::units(),
        )?;
        speed.push(0);
        if (STAGE_TIMES.records)() == 0 {
            return Err(crate::errors::BadInput("no stage records").into());
        }
        // Converted: `rc2k_core::screens::player_result::stage_result`.
        use crate::screens::game_screen::PlayerResultScreen;
        speed.pop();
        let p = rc2k_core::screens::player_result::Player {
            name: name as u32,
            nation: f(4),
            car: f(8),
            time: f(0x88),
            speed,
        };
        rc2k_core::screens::player_result::stage_result(
            &mut PlayerResultScreen::new(canvas),
            &mut |s: &mut PlayerResultScreen| draw_times_heading(&STAGE_TIMES, s.canvas),
            &p,
            crate::strings::append_time::show_hours(),
        )?;
        Ok(())
    }
}

fn rust(_: &Regs, canvas: &mut dyn Canvas) -> Result<Left, Refused> {
    unsafe { draw_player_stage_result(canvas) }?;
    Ok(Left::nothing())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    draw_entry("screens::draw_player_stage_result", r, &rust)
}
