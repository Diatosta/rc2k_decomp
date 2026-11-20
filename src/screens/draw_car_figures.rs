//! The car specification's figures for the page shown (`event.shown.car_page`):
//! - 0: the description (the seven strings of the car, or the one string for car 0x21).
//! - 1: engine: capacity, type, power and torque with their rpm, bore and stroke, compression.
//! - 2: transmission, drive, steering, brakes (front and rear) and suspension.
//! - 3: length, width, height, wheelbase, front and rear track, weight.
//!
//! Labels and figures use the two fonts of the page; a figure of 0 is string 0x578 ("n/d").
//! Refuses a page past the four, what the record's lookup refuses and a line longer than the line
//! buffer.

use crate::errors::{BadInput, Refused};
use crate::menu::car_description::{read, shown_description};
use crate::text::draw_text::Canvas;

// ---- game data ----

// `event.shown.car`
const PAGES: u32 = 4;

/// Returns whether the last text drawn was too wide for its box.
pub fn draw_car_figures(
    o: crate::owners::Owners<'static>,
    canvas: &mut dyn Canvas,
) -> Result<bool, Refused> {
    let menus = o.menus;
    let page: u32 = menus.screens.event.shown.car_page.get();
    if page >= PAGES {
        return Err(BadInput("a page past the four").into());
    }
    draw_car_page(o, page, shown_description(menus)?, canvas)
}

/// One page (0 to 3) of the record at `record`; returns whether the last text drawn was too wide
/// for its box. The logic is in `rc2k_core::screens::car_pages::page` (the record written back when
/// it changed it: the Clio's strings).
fn draw_car_page(
    o: crate::owners::Owners<'static>,
    page: u32,
    record: u32,
    canvas: &mut dyn Canvas,
) -> Result<bool, Refused> {
    use crate::screens::game_screen::GameScreen;
    use rc2k_core::screens::car_pages::{Font, Work};
    {
        let before = read(o.menus, record)?;
        let mut d = before;
        let notice = crate::strings::locale::notice_kind(o.strings);
        let too_wide = rc2k_core::screens::car_pages::page(
            &mut GameScreen::<Font, Work>::new(canvas, o),
            page,
            &mut d,
            notice,
            crate::strings::units::units(o.strings),
        );
        if d != before {
            crate::menu::cars_store::description(&o.menus.cars.table, record)?.set(d);
        }
        too_wide
    }
}
