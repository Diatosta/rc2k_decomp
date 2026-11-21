//! The developers' keypad switches as the debug UI's tab: each key cycled one variable. Key 2's
//! variable has no reader, so it is only named. Key 9 also set a stage-loading flag, which
//! nothing reads. What each does is read from its readers, and marked as such in the tab's
//! texts. These are the game's own variables: the race's settings copy (`race::settings_copy`)
//! and a stage's load may put them back.

use crate::race::owner::Race;
use dear_imgui_rs::Ui;

/// A tick box over a switch of 0 and 1, with the key it had and what it does.
fn tick(ui: &Ui, key: u32, name: &str, cell: &crate::memory::session::PlainDword, about: &str) {
    let mut on = cell.is_set();
    if ui.checkbox(format!("{key}: {name}"), &mut on) {
        cell.set(u32::from(on));
    }
    ui.same_line();
    ui.text_disabled(about);
}

/// The tab.
pub fn tab(race: &Race, ui: &Ui) {
    let shared = &race.shared;
    let words = &shared.vars.get().settings_words;
    ui.text_disabled("Numbered by the keypad key each had");
    tick(
        ui,
        1,
        "Mipmaps' setting",
        &shared.vars.get().mip_setting,
        "the mipmaps: the software rasteriser and the OpenGL renderer read smaller texture levels when on (the GPU API renderer draws alike)",
    );
    ui.text_disabled("2: a switch nothing reads");
    ui.text_disabled("3: the processor's extensions: one path now, nothing to switch");
    tick(ui, 4, "Car shadows", &words[2], "race::car_shadow");
    tick(
        ui,
        5,
        "Faces' shine",
        &words[1],
        "scene::draw_object_face, a reading",
    );
    tick(
        ui,
        6,
        "Setting 0",
        &words[0],
        "the lens flare: `listener_figures[0]` gates and fades `draw_lens_flare`",
    );
    ui.text("7: See-through windows: none, the viewed car, all cars");
    for (name, value) in [("0", 0), ("1", 1), ("2", 2)] {
        ui.same_line();
        if ui.radio_button_bool(name, words[3].get() == value) {
            words[3].set(value);
        }
    }
    ui.same_line();
    ui.text_disabled("light_word: 1 and 2 differ only with other cars");
    tick(
        ui,
        8,
        "All cars' lamps light objects",
        &words[4],
        "scene::add_car_lights: only other cars' lamps change; needs rivals, at night",
    );
    let marks_before = shared.switches.borrow().view_marks_on;
    let view_marks_on = crate::memory::session::PlainDword::new(marks_before);
    tick(
        ui,
        9,
        "View marks",
        &view_marks_on,
        "the effects on the view (race::frame_views); a stage's load sets it on",
    );
    if view_marks_on.get() != marks_before {
        shared.switches.borrow_mut().view_marks_on = view_marks_on.get();
    }
    let clear = &crate::scene::set_object_light::CLEAR_KIND_2_STRENGTH;
    let mut on = clear.load(std::sync::atomic::Ordering::Relaxed);
    if ui.checkbox("O: Light kind 2 without its strength", &mut on) {
        clear.store(on, std::sync::atomic::Ordering::Relaxed);
    }
    ui.same_line();
    ui.text_disabled("scene::set_object_light: those objects lit as kind 1 (the key O held, once)");
}
