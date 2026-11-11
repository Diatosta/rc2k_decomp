//! `screens::draw_repair_times` (0x4A70D7): draws sixteen values of the service area as
//! clocks, in two columns of eight: the repair times, left column first in each row.
//!
//! Rows are at y = 110 .. 284 in steps of 29, and a last one at 452; columns at x = 106 and
//! 441.
//!
//! Screen: seen in a screenshot of the championship's service area (2026-10-10): the time under each of the
//! fourteen parts of the repair section (for example 10:19 under the gearbox, 1:35 under the
//! tyres), and in the last row the 1:20 under "ride height" and "suspension stiffness".

use crate::errors::Refused;
use crate::menu::owner::Screens;
use crate::session::read_all;
use crate::text::draw_text::Canvas;

// ---- game data ----

const COUNT: usize = 16;

pub fn draw_repair_times(screens: &Screens, canvas: &mut dyn Canvas) -> Result<(), Refused> {
    // The logic is in `rc2k_core::screens::service::repair_times`.
    let times: [u32; COUNT] = read_all(&screens.state.driver_stats.repair_times);
    rc2k_core::screens::service::repair_times(
        &mut crate::screens::game_screen::ServiceScreen::new(canvas),
        &times,
    )?;
    Ok(())
}
