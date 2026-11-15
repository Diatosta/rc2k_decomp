//! What the six marker functions of the service area's setup bars share. There are four bars,
//! 204 pixels long: an upper pair at y = 409 and a lower pair at y = 465, the left ones at x = 129
//! and the right ones at x = 464. They belong to menu items 0x10 (upper left), 0x11 (upper right),
//! 0x12 (lower left) and 0x13 (lower right). Each function sets up the bar globals and calls
//! `gfx::blit_bar_marker`, so the marker sits at a percentage of the bar.
//!
//! Seen in a screenshot of the championship's service area, setup section: 0x10 is the steering
//! sensitivity (slow to fast), 0x11 the brake balance (front to rear), 0x12 the ride height (low
//! to high), 0x13 the suspension stiffness (soft to hard). The two lower bars have two markers
//! each: the other (yellow) is at the value being chosen, which the keys step, and only appears
//! while it differs or its change is selected. The steering and brake values are changed in the
//! player directly, so those bars have one marker.

use crate::errors::Refused;
use crate::gfx::blit_bar::Bar;
use crate::gfx::blit_bar_marker::blit_bar_marker;
use crate::gfx::owner::Gfx;
use crate::menu::owner::Menus;
use crate::service::owner::Service;
use crate::service::state::{blink_on, edit_mode, repair_flag};
use crate::session::PlainDword;
use crate::text::draw_text::Canvas;

pub use rc2k_core::screens::setup_markers::{BAR_WIDTH, LEFT_X, RIGHT_X};

pub fn lower_left_chosen(service: &Service) -> &PlainDword {
    &service.setups[2].chosen
}
pub fn lower_left_installed(service: &Service) -> &PlainDword {
    &service.setups[2].installed
}
pub fn lower_right_chosen(service: &Service) -> &PlainDword {
    &service.setups[3].chosen
}
pub fn lower_right_installed(service: &Service) -> &PlainDword {
    &service.setups[3].installed
}

/// A setup value the markers read.
pub type Setting = for<'a> fn(&'a Service) -> &'a PlainDword;

/// One of the six functions.
#[derive(Clone, Copy)]
pub enum Marker {
    /// Always drawn, at the percentage in a variable (glyph 0x77).
    Always { x: i32, percent: Setting },
    /// Glyph 0x76 on a lower bar, at the chosen value. While its item is being edited it
    /// blinks. Otherwise it is drawn if the chosen value differs from the installed one, or
    /// if the item's flag in the repair flag table is 1.
    Lower {
        item: u32,
        x: i32,
        chosen: Setting,
        installed: Setting,
    },
    /// Glyph 0x76 on an upper bar, at the percentage in a field of the player. While its item is
    /// being edited it blinks.
    Upper {
        item: u32,
        x: i32,
        in_car: rc2k_core::menu::player::SetupField,
    },
}

/// Puts the bar in the bar globals and draws the marker.
fn place(
    gfx: &Gfx,
    menus: &Menus,
    canvas: &mut dyn Canvas,
    (x, y, glyph): (i32, i32, u32),
    percent: i32,
) -> Result<(), Refused> {
    Bar {
        x,
        y,
        width: BAR_WIDTH,
        fill_glyph: glyph,
        percent,
        ..Bar::read(gfx)
    }
    .write_in(gfx);
    blit_bar_marker(gfx, menus, &mut |call| canvas.blit_glyph(call))?;
    Ok(())
}

/// `player` is the player for the upper bars. Its one caller, `screens::draw_service_area`,
/// reads no result.
pub fn draw_marker(
    menus: &Menus,
    gfx: &Gfx,
    marker: Marker,
    player: crate::menu::standings::PlayerCell,
    canvas: &mut dyn Canvas,
) -> Result<(), Refused> {
    // The logic is in `rc2k_core::screens::setup_markers::plan`; here the values read and the bar
    // drawn.
    use rc2k_core::screens::setup_markers::{Marker as M, Menu, Plan};
    let menu = Menu {
        edit_mode: edit_mode(menus),
        item: menus.screens.loop_flags.item.get(),
        blink_on: blink_on(menus),
    };
    let read = |cell: Setting| -> Result<i32, Refused> { Ok(cell(&menus.service).get() as i32) };
    let (core, item): (M, u32) = match marker {
        Marker::Always { x, .. } => (M::Always { x }, 0),
        Marker::Lower { item, x, .. } => (M::Lower { item, x }, item),
        Marker::Upper { item, x, .. } => (M::Upper { item, x }, item),
    };
    let percent = || -> Result<i32, Refused> {
        match marker {
            Marker::Always { percent, .. } => read(percent),
            Marker::Lower { chosen, .. } => read(chosen),
            Marker::Upper { in_car, .. } => Ok(in_car.get(player.get()) as i32),
        }
    };
    let installed = || match marker {
        Marker::Lower { installed, .. } => installed(&menus.service).get() as i32,
        _ => 0,
    };
    let (plan, _) =
        rc2k_core::screens::setup_markers::plan(core, menu, &percent, &installed, &|| {
            repair_flag(&menus.service, item)
        })?;
    match plan {
        Plan::Hidden { .. } => Ok(()),
        Plan::Draw {
            x,
            y,
            glyph,
            percent,
        } => place(gfx, menus, canvas, (x, y, glyph), percent),
    }
}
