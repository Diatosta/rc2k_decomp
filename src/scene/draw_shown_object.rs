//! `scene::draw_shown_object` (0x4503A5): draws the current object unless a race's tests hide
//! it.
//!
//! The drawing-object flag is 1 for the duration, 0 at the end. In a race (`in_race` not 0)
//! two tests, each of which can hide the object:
//! - `scene::object_hidden_by_kind`: with `in_credits` not 0 or the run kind at 2, an object
//!   of kind 0x30 or 0x4E;
//! - `scene::object_hidden`: objects 0x38 and 0x3A .. 0x3F are hidden with `scene_set_up`
//!   set, with `shown` at 0, object 0x3E with `stage_direction` not 0 and object 0x3D with
//!   it not 1; with `scene_set_up` set and `only_object` not 0, every object unless
//!   `only_object` - 1 is `stage_direction`; and an object without faces (the slots'
//!   `face_counts`).
//!
//! Outside a race `scene::clear_kind_bytes`: the 0x100 `car_mark_textures` = 0, then the
//! seventh, eighth and tenth = 999: here the caller's context carries the menus' marks
//! instead (`scene::car_marks::MENUS`). Then `scene::draw_object`. One caller,
//! `scene::draw_scene_object`. Refuses, in a race, an object's number of 0x100 or more (the
//! original reads past its tables).
//!
//! The race's tests and the kind bytes are in `rc2k_core::scene::shown` (the variables are
//! read where the tests read them).

use crate::errors::{BadInput, Refused};

// ---- game data ----
use crate::race::owner::Race;
use rc2k_core::scene::shown::OBJECTS;

// ---- Rust version ----

/// The game's variables for the race's tests.
struct Game<'a>(
    &'a Race,
    &'a crate::race::car_draw_state::CarDrawState,
    &'a crate::scene::object_place::ObjectPlace,
    crate::workers::DrawReads<'a>,
);

impl rc2k_core::scene::shown::RaceObject for Game<'_> {
    fn object(&self) -> u32 {
        self.1.object_number.get()
    }
    fn kind_switch(&self) -> bool {
        self.0.shared.work.in_credits.is_set()
    }
    fn kind_mode(&self) -> u32 {
        self.0.shared.vars.run_kind.get()
    }
    fn kind(&self, object: u32) -> u32 {
        self.3.object_tables.object_kind(object)
    }
    fn other_way(&self) -> u32 {
        self.0.shared.switches.scene_set_up.get()
    }
    fn shown(&self) -> u32 {
        self.0.course.stage_prefs.shown.get()
    }
    fn which(&self) -> u32 {
        self.0.shared.work.stage_direction.get()
    }
    fn only(&self) -> u32 {
        self.2.only_object.get()
    }
    fn faces(&self, object: u32) -> u32 {
        self.3.object_slots.face_counts[object as usize].get()
    }
}

/// `draw` stands for `scene::draw_object`.
// Converted: `rc2k_core::scene::shown::race_shown`
pub fn draw_shown_object(
    race: &Race,
    reads: crate::workers::DrawReads<'_>,
    draw: &mut dyn FnMut() -> Result<(), Refused>,
    cars: &crate::race::car_draw_state::CarDrawState,
    object: &crate::scene::object_place::ObjectPlace,
) -> Result<bool, Refused> {
    let in_race = race.course.numbers.in_race.is_set();
    if in_race && cars.object_number.get() >= OBJECTS {
        return Err(BadInput("an object's number past the tables").into());
    }
    object.drawing_object.set(1u32);
    let show = if in_race {
        rc2k_core::scene::shown::race_shown(&Game(race, cars, object, reads))
    } else {
        // The original set the 0x100 kind dwords here (`scene::clear_kind_bytes`): the
        // caller's context has the menus' marks (`scene::car_marks`).
        true
    };
    if show {
        draw()?;
    }
    object.drawing_object.set(0u32);
    Ok(show)
}
