//! `scene::full_view_window` (0x44570A): `scene::choose_view_window` with `LightTables.view_mode` 2
//! (800 x 600 when framed), the word back to 0 after.
//!
//! Callers: `hud::overlay_pass` (0x45A906), code at 0x4435B8.

use crate::scene::choose_view_window::choose_view_window;

/// `choose_view_window` with the mode 2, the mode back to 0 after.
pub fn full_view_window(
    race: &crate::race::owner::Race,
    scene: &crate::scene::owner::Scene,
    system: &crate::system::owner::System,
) {
    let light_tables = scene.light_tables.get();
    light_tables.view_mode.set(2);
    choose_view_window(race, scene, system);
    light_tables.view_mode.set(0);
}
