//! `screens::draw_player_stage_result` (0x486299): the player's result on a championship
//! stage: the three labels (strings 0x206 .. 0x208 at x 0x1EC, font 0x51E6E4), the stage's
//! heading (`screens::draw_stage_times_heading`'s), the player's time (+0x88) and top speed
//! (+0x94, with its unit) at x 0x1FE, the car's sprite, the name (+0x18) at x 0x221 and the
//! flag's sprite after it (its width by `text::measure_text` in the font at 0x51E624); the
//! sprites' palette ebx as the original leaves it (the speed line past its NUL, then the
//! measure's line count).
//!
//!
//! In a menu loop's draw list (0x758E7E). Refuses (before anything) no current player or name, no
//! stage records, and what `strings::append_speed` refuses. The menus and draw lists run its
//! `entry` (`menu::table_calls`).

use crate::errors::Refused;
use crate::menu::owner::Menus;
use crate::screens::best_scores::{STAGE_TIMES, draw_times_heading};
use crate::text::draw_text::Canvas;

pub fn draw_player_stage_result(
    menus: &Menus,
    system: &crate::system::owner::System,
    canvas: &mut dyn Canvas,
) -> Result<(), Refused> {
    let player = crate::menu::player_settings::current(menus)?.get();
    let name = crate::screens::game_screen::player_name_of(crate::menu::player_name::name(menus)?);
    let top_speed = player.figure;
    let mut speed = crate::strings::append_speed::append_speed(
        top_speed,
        true,
        crate::strings::units::units(),
    )?;
    speed.push(0);
    (STAGE_TIMES.records)(menus, system)?;
    // The logic is in `rc2k_core::screens::player_result::stage_result`.
    use crate::screens::game_screen::PlayerResultScreen;
    speed.pop();
    let p = rc2k_core::screens::player_result::Player {
        name,
        nation: player.nationality,
        car: player.car_number,
        time: player.time,
        speed,
    };
    rc2k_core::screens::player_result::stage_result(
        &mut PlayerResultScreen::new(canvas),
        &mut |s: &mut PlayerResultScreen| draw_times_heading(menus, system, &STAGE_TIMES, s.canvas),
        &p,
        crate::strings::append_time::show_hours(),
    )?;
    Ok(())
}
