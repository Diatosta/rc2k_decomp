//! Draws sixteen values of the service area as clocks, in two columns of eight: the repair
//! times, left column first in each row.
//!
//! Seen in a screenshot of the championship's service area: the time under each of the fourteen
//! parts of the repair section (for example 10:19 under the gearbox, 1:35 under the tyres), and in
//! the last row the 1:20 under "ride height" and "suspension stiffness".

use crate::errors::Refused;
use crate::service::owner::service;
use crate::session::read_all;
use crate::text::draw_text::Canvas;

// ---- game data ----

const COUNT: usize = 16;

pub fn draw_repair_times(
    o: crate::owners::Owners<'static>,
    canvas: &mut dyn Canvas,
) -> Result<(), Refused> {
    // The logic is in `rc2k_core::screens::service::repair_times`.
    let times: [u32; COUNT] = read_all(&service().repair_times);
    rc2k_core::screens::service::repair_times(
        &mut crate::screens::game_screen::ServiceScreen::new(canvas, o),
        &times,
    )?;
    Ok(())
}
