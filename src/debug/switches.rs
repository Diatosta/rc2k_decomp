//! The developers' keypad switches as the debug UI's tab: each key cycled one variable. Key 2's
//! variable has no reader, so it is only named. Key 9 also set a stage-loading flag, which
//! nothing reads. What each does is read from its readers, and marked as such in the tab's
//! texts. These are the game's own variables: the race's settings copy (`race::settings_copy`)
//! and a stage's load may put them back.

use crate::race::owner::Race;
use crate::scene::owner::Scene;
use crate::system::owner::System;
use dear_imgui_rs::Ui;
use std::sync::atomic::{AtomicU32, Ordering::Relaxed};

/// The developers' switch, 1 by default; nothing reads it.
pub static EXTENSIONS: AtomicU32 = AtomicU32::new(1);

/// A tick box over a switch of 0 and 1, with the key it had and what it does.
fn tick(ui: &Ui, key: u32, name: &str, cell: &crate::session::PlainDword, about: &str) {
    let mut on = cell.is_set();
    if ui.checkbox(format!("{key}: {name}"), &mut on) {
        cell.set(u32::from(on));
    }
    ui.same_line();
    ui.text_disabled(about);
}

/// The tab.
pub fn tab(race: &Race, scene: &Scene, system: &System, ui: &Ui) {
    let shared = &race.shared;
    let words = &shared.vars.get().settings_words;
    ui.text_disabled("Numbered by the keypad key each had");
    tick(
        ui,
        1,
        "Mipmaps' setting",
        &shared.vars.get().mip_setting,
        "the original's mipmaps; our rasteriser filters alike either way: no change",
    );
    ui.text_disabled("2: dword_6218E0: nothing reads it");
    // Key 3: the switch masked with what the processor has.
    let use_ = scene.draw_switches.get();
    let mut on = EXTENSIONS.load(Relaxed) != 0;
    if ui.checkbox("3: Processor's extensions", &mut on) {
        let v = u32::from(on);
        EXTENSIONS.store(v, Relaxed);
        use_.use_sse
            .set(on && system.machine().has_sse.get() & 1 != 0);
        // The 3DNow way is not ported and would end the program, so it stays off on a
        // processor that has it.
        use_.use_3dnow.set(0);
        use_.use_mmx.set(v & system.machine().has_mmx.get());
    }
    ui.same_line();
    let used = [
        (use_.use_sse.is_set(), "SSE"),
        (use_.use_3dnow.get() != 0, "3DNow!"),
        (use_.use_mmx.get() != 0, "MMX"),
    ]
    .iter()
    .filter(|&&(in_use, _)| in_use)
    .map(|&(_, name)| name)
    .collect::<Vec<_>>()
    .join(", ");
    ui.text_disabled(format!(
        "using: {}; the game's SSE paths, physics' ground too: a replay may not match",
        if used.is_empty() { "none" } else { &used }
    ));
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
    tick(
        ui,
        9,
        "View marks",
        &shared.switches.get().view_marks_on,
        "the effects on the view (race::frame_views); a stage's load sets it on",
    );
}
