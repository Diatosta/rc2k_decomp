//! What the six marker functions of the service area's setup bars share. There are four bars,
//! 204 pixels long: an upper pair at y = 409 and a lower pair at y = 465, the left ones at
//! x = 129 and the right ones at x = 464. They belong to menu items 0x10 (upper left), 0x11
//! (upper right), 0x12 (lower left) and 0x13 (lower right). Each function sets up the bar
//! globals and calls `gfx::blit_bar_marker`, so the marker sits at a percentage of the bar.
//!
//! Screen: seen in a screenshot of the championship's service area (2026-10-10), setup section:
//! 0x10 is the steering sensitivity (slow .. fast), 0x11 the brake balance (front .. rear), 0x12
//! the ride height (low .. high), 0x13 the suspension stiffness (soft .. hard). The two lower bars
//! have two markers each. The other (0x76, yellow) is at the value being chosen (`dword_75ACA8`,
//! `dword_75ACB4`, which the keys step) and only appears while it differs or its change is
//! selected. The steering and brake values are changed in the player directly, so those bars have
//! one marker.

use crate::errors::Refused;
use crate::gfx::blit_bar::Bar;
use crate::gfx::blit_bar_marker::blit_bar_marker;
use crate::menu::state_store::MENU_STATE;
use crate::service::state::{blink_on, edit_mode, menu_item, repair_flag};
use crate::session::Dword;
use crate::text::draw_text::Canvas;

pub use rc2k_core::screens::setup_markers::{BAR_WIDTH, LEFT_X, RIGHT_X};

pub const LOWER_LEFT_CHOSEN: &Dword = &MENU_STATE.driver_stats.setups[2].chosen;
pub const LOWER_LEFT_INSTALLED: &Dword = &MENU_STATE.driver_stats.setups[2].installed;
pub const LOWER_RIGHT_CHOSEN: &Dword = &MENU_STATE.driver_stats.setups[3].chosen;
pub const LOWER_RIGHT_INSTALLED: &Dword = &MENU_STATE.driver_stats.setups[3].installed;

/// One of the six functions.
#[derive(Clone, Copy)]
pub enum Marker {
    /// Always drawn, at the percentage in a variable (glyph 0x77).
    Always { x: i32, percent: &'static Dword },
    /// Glyph 0x76 on a lower bar, at the chosen value. While its item is being edited it
    /// blinks. Otherwise it is drawn if the chosen value differs from the installed one, or
    /// if the item's flag in the repair flag table is 1.
    Lower {
        item: u32,
        x: i32,
        chosen: &'static Dword,
        installed: &'static Dword,
    },
    /// Glyph 0x76 on an upper bar, at the percentage in a field of the player in edi.
    /// While its item is being edited it blinks.
    Upper { item: u32, x: i32, car_offset: u32 },
}

/// Puts the bar in the bar globals and draws the marker.
fn place(
    canvas: &mut dyn Canvas,
    palette_at_start: u32,
    (x, y, glyph): (i32, i32, u32),
    percent: i32,
) -> Result<(), Refused> {
    Bar {
        x,
        y,
        width: BAR_WIDTH,
        fill_glyph: glyph,
        percent,
        ..Bar::read()
    }
    .write();
    { blit_bar_marker(palette_at_start, &mut |call| canvas.blit_glyph(call)) }?;
    Ok(())
}

/// `palette` (the original's ebx) goes to the blitter as the palette when the percentage
/// is 0 or less, and `player` (edi) is the player for the upper bars. What the original
/// leaves in eax and esi its one caller, `screens::draw_service_area`, does not read.
pub fn draw_marker(
    marker: Marker,
    palette: u32,
    player: crate::menu::standings::PlayerCell,
    canvas: &mut dyn Canvas,
) -> Result<(), Refused> {
    // Converted: `rc2k_core::screens::setup_markers::plan`; here the values read and the bar drawn.
    use rc2k_core::screens::setup_markers::{Marker as M, Menu, Plan};
    let menu = Menu {
        edit_mode: edit_mode(),
        item: menu_item(),
        blink_on: blink_on(),
    };
    let read = |cell: &Dword| -> Result<i32, Refused> { Ok(cell.get() as i32) };
    let (core, item): (M, u32) = match marker {
        Marker::Always { x, .. } => (M::Always { x }, 0),
        Marker::Lower { item, x, .. } => (M::Lower { item, x }, item),
        Marker::Upper { item, x, .. } => (M::Upper { item, x }, item),
    };
    let percent = || -> Result<i32, Refused> {
        match marker {
            Marker::Always { percent, .. } => read(percent),
            Marker::Lower { chosen, .. } => read(chosen),
            Marker::Upper { car_offset, .. } => Ok(player.get().word(car_offset as usize) as i32),
        }
    };
    let installed = || match marker {
        Marker::Lower { installed, .. } => installed.get() as i32,
        _ => 0,
    };
    let (plan, _) =
        rc2k_core::screens::setup_markers::plan(core, menu, &percent, &installed, &|| {
            repair_flag(item)
        })?;
    match plan {
        Plan::Hidden { .. } => Ok(()),
        Plan::Draw {
            x,
            y,
            glyph,
            percent,
        } => place(canvas, palette, (x, y, glyph), percent),
    }
}
