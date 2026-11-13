//! `menu::draw_menu`: draws a menu, its fixed labels and then its items.
//!
//! `screens::draw_menu_labels`, then `screens::draw_menu_items`. Called by
//! `menu::draw_frame` and `menu::frame_callback`, and by the screens with a frame callback of their
//! own. Refuses what the two functions refuse. Rust callers run its `entry` (`routes.rs`).

use crate::errors::Refused;
use crate::screens::draw_menu_items::draw_menu_items;
use crate::screens::draw_menu_labels::draw_menu_labels;
use crate::text::draw_text::Canvas;

/// Draws `menu`.
pub fn draw_menu(
    o: crate::owners::Owners<'static>,
    menu: crate::menu::menus_store::MenuId,
    canvas: &mut dyn Canvas,
) -> Result<(), Refused> {
    draw_menu_labels(o, menu, canvas)?;
    draw_menu_items(o, menu, canvas)
}
