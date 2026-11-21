//! `scene::full_view_window`: `scene::choose_view_window` with `DrawWork.view_mode` 2 (800 x 600
//! when framed), the word back to 0 after.

use crate::scene::choose_view_window::choose_view_window;

/// `choose_view_window` with the mode 2, the mode back to 0 after.
pub fn full_view_window(
    race: &crate::race::owner::Race,
    scene: &crate::scene::owner::Scene,
    system: &crate::system::owner::System,
) {
    scene.draw_work_mut().view_mode = 2;
    choose_view_window(race, scene, system);
    scene.draw_work_mut().view_mode = 0;
}
