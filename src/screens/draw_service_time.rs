//! The two time readouts at the top of the service area, from the time budget and the time of
//! the repairs chosen. While `budget - repairs` is at least 900: the time available
//! (`budget - 900`) on the left and the time left (`budget - 900 - repairs`) on the right. Below
//! that: the overrun (`900 - (budget - repairs)`) and the penalty (10 for every started 60 of
//! overrun). All four are printed with `append_clock`. It leaves the penalty in the money.
//!
//! Seen in a screenshot: "TEMPO DISPONÍVEL: 40:00" at the top left and "TEMPO RESTANTE: 40:00" at
//! the top right. The overrun pair of labels has not been seen.

use crate::errors::Refused;
use crate::text::draw_text::Canvas;

// ---- game data ----

// the money

// (label string, font) for the four readouts.
/// Returns whether the last text drawn was too wide for its box.
pub fn draw_service_time(
    o: crate::owners::Owners<'static>,
    canvas: &mut dyn Canvas,
) -> Result<bool, Refused> {
    let screens = &o.menus.screens;
    // The logic is in `rc2k_core::screens::service::service_time`.

    rc2k_core::screens::service::service_time(
        &mut crate::screens::game_screen::ServiceScreen::new(canvas, o),
        screens.event.service.minutes.get() as i32,
        o.menus.service.repair_time.get() as i32,
    )
}
