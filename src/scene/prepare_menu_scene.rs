//! `scene::prepare_menu_scene` (0x46F7FD): the menus' camera and light.
//!
//! The view's place and its three angles = 0; the window 800 x 600 (`dword_62408C`,
//! `dword_624090`); `scene::set_view_window`; the light's lowest 0x50, highest 0xFF, level 0xB4
//! and strength 0xB4, its direction's angles `dword_635094` = 0x6E and `dword_635098` = 0x1E;
//! `scene::set_view_matrices`. 29 callers (the screen loops' set-up).
use crate::race::owner::race;
use crate::scene::set_view_window::set_view_window;
use crate::scene::view_matrix::set_view_matrices;

// ---- game data ----

// ---- Rust version ----

pub fn prepare_menu_scene(scene: &crate::scene::owner::Scene) {
    let race = race();
    let frame = &race.frame;
    crate::session::write_bits(&race.frame.view_and_records.view_vars.view_place, [0u32; 3]);
    crate::session::write_first(
        &race.frame.view_and_records.view_vars.view_angles,
        &[0u32; 3],
    );
    crate::session::write_bits(
        &crate::scene::polygon_work::front_end().window.view_size,
        [0x320u32, 0x258],
    );
    set_view_window(
        race,
        scene,
        &crate::scene::polygon_work::front_end().window,
        &race.frame.view_and_records.view_vars,
    );
    let light = &crate::race::view_scratch::front_end().light;
    light.light_lowest.set(0x50);
    light.light_highest.set(0xFF);
    light.light_level.set(0xB4);
    crate::race::view_scratch::front_end()
        .object
        .light_strength
        .set(0xB4);
    crate::session::write_bits(&light.light_turns, [0x6Eu32, 0x1E]);
    set_view_matrices(
        &race.course,
        frame,
        &crate::race::view_scratch::front_end().matrices,
    );
}
