//! `scene::choose_view_window` (0x44FE1A): the view's window, then `scene::set_view_window`:
//! without the front-end switch the screen's size at 0, 0; with it 800 x 600 at 0, 0 when
//! `scene_set_up` is set or the screen mode word is 2, else 720 x 540 at 0x50, 0; the view's
//! zoom 0x100 either way.
//!
//! Callers: `scene::full_view_window`, `race::loading_bar_frame`.
//! //! `set_view_window` does the rest.

use crate::race::owner::Race;

/// The window chosen, then `set_view_window`.
pub fn choose_view_window(
    race: &Race,
    scene: &crate::scene::owner::Scene,
    system: &crate::system::owner::System,
) {
    let session = &system.session;
    let shared = &race.shared;
    // Converted: `rc2k_core::scene::view_window::choose`.
    use rc2k_core::scene::view_window::{ViewSettings, choose};
    let settings = ViewSettings {
        framed: shared.switches.editor_mode.get() != 0,
        wide: shared.switches.scene_set_up.get() != 0,
        mode: scene.light_tables.view_mode.get(),
    };
    // The front end's window: the menus, the loading bar and the HUD's full view.
    let view = crate::scene::polygon_work::front_end();
    let window = &view.window;
    let chosen = choose(settings, || {
        [session.screen.width.get(), session.screen.height.get()]
    });
    window.view_size[0].set(chosen.size[0]);
    window.view_size[1].set(chosen.size[1]);
    window.view_offset[0].set(chosen.offset[0]);
    window.view_offset[1].set(chosen.offset[1]);
    window.view_zoom.set(chosen.scale);
    crate::scene::set_view_window::set_view_window(
        race,
        scene,
        window,
        &race.frame.view_and_records.view_vars,
    );
}
