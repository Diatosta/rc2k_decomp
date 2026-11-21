//! `scene::choose_view_window`: the view's window, then `scene::set_view_window`: without the
//! front-end switch the screen's size at 0, 0; with it 800 x 600 at 0, 0 when `scene_set_up` is set
//! or the screen mode word is 2, else 720 x 540 at 0x50, 0; the view's zoom 0x100 either way.

use crate::race::owner::Race;

/// The window chosen, then `set_view_window`.
pub fn choose_view_window(
    race: &Race,
    scene: &crate::scene::owner::Scene,
    system: &crate::system::owner::System,
) {
    let session = &system.session;
    let shared = &race.shared;
    // Logic: `rc2k_core::scene::view_window::choose`.
    use rc2k_core::scene::view_window::{ViewSettings, choose};
    let settings = ViewSettings {
        framed: shared.switches.borrow().editor_mode != 0,
        wide: shared.switches.borrow().scene_set_up != 0,
        mode: scene.draw_work().view_mode,
    };
    // The front end's window: the menus, the loading bar and the HUD's full view.
    let view = scene.scratch.front_end_polygon();
    let window = &view.window;
    let chosen = choose(settings, || {
        [
            session.screen.get().width.get(),
            session.screen.get().height.get(),
        ]
    });
    {
        let mut window = window.borrow_mut();
        window.view_size = chosen.size;
        window.view_offset = chosen.offset;
        window.view_zoom = chosen.scale;
    }
    let draw = *scene.drawing.scene.borrow();
    crate::scene::set_view_window::set_view_window(
        race.course.reads(),
        &draw,
        system.session.screen.get(),
        window,
        &race.frame.view_and_records.get().view_vars,
        &crate::race::vars_store::SCALE_FACTOR,
    );
}
