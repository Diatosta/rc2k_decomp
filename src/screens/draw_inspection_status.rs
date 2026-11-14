//! Works out the inspection status, a percentage, from the current car's thirteen damage
//! values and draws it as a bar at the bottom of the service area, with a sprite to its left and a
//! label at each end. Takes the palette of the sprite at the bar's left.
//!
//! The percentage is 100 minus the weights of the parts that are damaged (at least the repair
//! limit) and not chosen for repair, over the fourteen parts without the turbo (item 6). The
//! result is left in the inspection score.
//!
//! Seen in a screenshot of the championship's service area: the "ESTADO DE INSPECÇÃO" label (drawn
//! by `draw_service_area`) and a red to green bar between "REPROV." and "PASSAR".

use crate::errors::{BadInput, Refused};
use crate::menu::owner::Screens;
use crate::service::state::{current_player, repair_flag};
use crate::text::draw_text::Canvas;

// ---- game data ----

// the inspection score

/// The percentage and the item of the last damaged part looked at. The logic is in
/// `rc2k_core::screens::inspection::status`.
pub fn inspection_status(
    screens: &Screens,
    player: crate::menu::standings::PlayerCell,
) -> Result<(i32, Option<u32>), crate::errors::BadInput> {
    use rc2k_core::screens::inspection::{PARTS, Part, status};
    let parts: [Part; 13] = std::array::from_fn(|n| {
        let item = PARTS[n];
        Part {
            damage: player.get().damage[item as usize - 2] as i32,
            threshold: screens.state.driver_stats.limits[item as usize - 2].get() as i32,
            weight: screens.state.driver_stats.penalties[item as usize - 2].get() as i32,
            repairing: repair_flag(item) != 0,
        }
    });
    let (condition, last) = status(&parts);
    Ok((condition, last.map(|n| PARTS[n])))
}

/// Refuses a missing player. Returns whether the text was too wide for its box, which the one
/// caller, `screens::draw_service_area`, reads.
pub fn draw_inspection_status(
    o: crate::owners::Owners<'static>,
    screens: &Screens,
    canvas: &mut dyn Canvas,
) -> Result<bool, Refused> {
    let menus = o.menus;
    let Some(player) = current_player(menus) else {
        return Err(BadInput("no current player object: the original reads through null").into());
    };
    let (condition, _) = inspection_status(screens, player)?;
    // The logic is in `rc2k_core::screens::inspection::inspection_status`.
    let out = rc2k_core::screens::inspection::inspection_status(
        &mut crate::screens::game_screen::GameScreen::<
            rc2k_core::screens::inspection::Font,
            rc2k_core::screens::inspection::Work,
        >::new(canvas, o),
        condition,
    )?;
    Ok(out.too_wide)
}
