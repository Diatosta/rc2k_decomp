//! `screens::draw_service_time` (0x4A5DE7): draws the two time readouts at the top of the
//! service area, from the time budget and the time of the repairs chosen.
//!
//! While `budget - repairs` is at least 900: "<string 0x48D>:  <budget - 900>" on the left
//! and "<string 0x48E>:  <budget - 900 - repairs>" on the right. Below that:
//! "<string 0x490>:  <overrun>" and "<string 0x491>:  <penalty>", where the overrun is
//! `900 - (budget - repairs)` and the penalty is 10 for every started 60 of overrun.
//! All four values are printed with `append_clock`. It leaves the penalty in `dword_75ACC0`.
//!
//! Screen: seen in a screenshot of the championship's service area (2026-10-10): "TEMPO DISPONÍVEL: 40:00" at
//! the top left and "TEMPO RESTANTE: 40:00" at the top right (time available, time left).
//! The overrun pair of labels has not been seen.

use crate::errors::Refused;
use crate::menu::owner::Screens;
use crate::text::draw_text::Canvas;

// ---- game data ----

// dword_75ACC0
// seconds: a work variable
// dword_7559EC: a work variable

// (label string, font) for the four readouts.
/// Returns the carry flag of the last `draw_text`.
pub fn draw_service_time(screens: &Screens, canvas: &mut dyn Canvas) -> Result<bool, Refused> {
    // Converted: `rc2k_core::screens::service::service_time`.

    rc2k_core::screens::service::service_time(
        &mut crate::screens::game_screen::ServiceScreen::new(canvas),
        screens.event.service_minutes.get() as i32,
        screens.state.driver_stats.repair_time.get() as i32,
    )
}
