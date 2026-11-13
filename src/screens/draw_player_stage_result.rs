//! The player's result on a championship stage: three labels, the stage's heading, the
//! player's time and top speed (with its unit), the car's sprite, the name and the flag's sprite
//! after it. Refuses no current player or name, no stage records and what `strings::append_speed`
//! refuses.

use crate::errors::Refused;
use crate::screens::best_scores::{STAGE_TIMES, draw_times_heading};
use crate::text::draw_text::Canvas;

pub fn draw_player_stage_result(
    o: crate::owners::Owners<'static>,
    system: &crate::system::owner::System,
    canvas: &mut dyn Canvas,
) -> Result<(), Refused> {
    let menus = o.menus;
    let player = crate::menu::player_settings::current(menus)?.get();
    let name = crate::screens::game_screen::player_name_of(crate::menu::player_name::name(menus)?);
    let top_speed = player.figure;
    let mut speed = crate::strings::append_speed::append_speed(
        crate::strings::append_string_by_id::StringOwners::of(o),
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
        &mut PlayerResultScreen::new(canvas, o),
        &mut |s: &mut PlayerResultScreen| draw_times_heading(o, system, &STAGE_TIMES, s.canvas),
        &p,
        crate::strings::append_time::show_hours(),
    )?;
    Ok(())
}
