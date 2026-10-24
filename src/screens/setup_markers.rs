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

use crate::errors::{BadInput, Refused};
use crate::framework::{game_read, rva};
use crate::gfx::blit_bar::Bar;
use crate::gfx::blit_bar_marker::blit_bar_marker;
use crate::service::state::{blink_on, edit_mode, menu_item, repair_flag};
use crate::text::draw_text::Canvas;

pub use rc2k_core::screens::setup_markers::{BAR_WIDTH, LEFT_X, RIGHT_X};

pub const LOWER_LEFT_CHOSEN_RVA: usize = rva(0x75ACA8); // dword_75ACA8
pub const LOWER_LEFT_INSTALLED_RVA: usize = rva(0x75ACAC); // dword_75ACAC
pub const LOWER_RIGHT_CHOSEN_RVA: usize = rva(0x75ACB4); // dword_75ACB4
pub const LOWER_RIGHT_INSTALLED_RVA: usize = rva(0x75ACB8); // dword_75ACB8

/// One of the six functions.
#[derive(Clone, Copy)]
pub enum Marker {
    /// Always drawn, at the percentage in a variable (glyph 0x77).
    Always { x: i32, percent_rva: usize },
    /// Glyph 0x76 on a lower bar, at the chosen value. While its item is being edited it
    /// blinks. Otherwise it is drawn if the chosen value differs from the installed one, or
    /// if the item's flag in the repair flag table is 1.
    Lower {
        item: u32,
        x: i32,
        chosen_rva: usize,
        installed_rva: usize,
    },
    /// Glyph 0x76 on an upper bar, at the percentage in a field of the player in edi.
    /// While its item is being edited it blinks.
    Upper { item: u32, x: i32, car_offset: u32 },
}

/// Puts the bar in the bar globals and draws the marker.
unsafe fn place(
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
    unsafe { blit_bar_marker(palette_at_start, &mut |call| canvas.blit_glyph(call)) }?;
    Ok(())
}

/// `palette` (the original's ebx) goes to the blitter as the palette when the percentage
/// is 0 or less, and `player` (edi) is the player for the upper bars. What the original
/// leaves in eax and esi its one caller, `screens::draw_service_area`, does not read.
pub unsafe fn draw_marker(
    marker: Marker,
    palette: u32,
    player: u32,
    canvas: &mut dyn Canvas,
) -> Result<(), Refused> {
    // Converted: `rc2k_core::screens::setup_markers::plan`; here the values read and the bar drawn.
    use rc2k_core::screens::setup_markers::{Marker as M, Menu, Plan};
    let menu = Menu {
        edit_mode: edit_mode(),
        item: menu_item(),
        blink_on: blink_on(),
    };
    let read = |rva: usize| -> Result<i32, Refused> { Ok(unsafe { game_read(rva) }) };
    let (core, item): (M, u32) = match marker {
        Marker::Always { x, .. } => (M::Always { x }, 0),
        Marker::Lower { item, x, .. } => (M::Lower { item, x }, item),
        Marker::Upper { item, x, .. } => (M::Upper { item, x }, item),
    };
    let percent = || -> Result<i32, Refused> {
        match marker {
            Marker::Always { percent_rva, .. } => read(percent_rva),
            Marker::Lower { chosen_rva, .. } => read(chosen_rva),
            Marker::Upper { car_offset, .. } => {
                if player == 0 {
                    return Err(
                        BadInput("no player in edi: the original reads through null").into(),
                    );
                }
                Ok(unsafe { (player.wrapping_add(car_offset) as *const i32).read_unaligned() })
            }
        }
    };
    let installed = || match marker {
        Marker::Lower { installed_rva, .. } => unsafe { game_read::<i32>(installed_rva) },
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
        } => unsafe { place(canvas, palette, (x, y, glyph), percent) },
    }
}
