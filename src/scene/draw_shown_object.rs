//! `scene::draw_shown_object`: draws the current object unless a race's tests hide it.
//!
//! The drawing-object flag is 1 for the duration, 0 at the end. In a race (`in_race` not 0)
//! two tests, each of which can hide the object:
//! - `scene::kind_is_hidden`: with `in_credits` not 0 or the run kind at 2, an object of
//!   kind 0x30 or 0x4E;
//! - the second test: objects 0x38 and 0x3A ..= 0x3F are hidden with `scene_set_up` set,
//!   with `shown` at 0, object 0x3E with `stage_direction` not 0 and object 0x3D with it not 1;
//!   with `scene_set_up` set and `only_object` not 0, every object unless `only_object` - 1 is
//!   `stage_direction`; and an object without faces (the slots' `face_counts`).
//!
//! Outside a race the caller's context carries the menus' marks (`scene::car_marks::MENUS`). Then
//! `scene::draw_object`. Refuses, in a race, an object's number of 0x100 or more.
//!
//! The race's tests and the kind bytes are in `rc2k_core::scene::shown`.

use crate::errors::{BadInput, Refused};

// ---- game data ----
use rc2k_core::scene::shown::OBJECTS;

/// The game's variables for the race's tests.
struct Game<'a>(
    &'a crate::race::car_draw_state::CarDrawState,
    &'a crate::scene::object_place::ObjectPlace,
    crate::scene::workers::DrawReads<'a>,
);

impl rc2k_core::scene::shown::RaceObject for Game<'_> {
    fn object(&self) -> u32 {
        self.0.borrow().object_number
    }
    fn kind_switch(&self) -> bool {
        self.2.work.in_credits()
    }
    fn kind_mode(&self) -> u32 {
        self.2.vars.run_kind()
    }
    fn kind(&self, object: u32) -> u32 {
        self.2.object_tables.object_kind(object)
    }
    fn other_way(&self) -> u32 {
        self.2.switches.scene_set_up()
    }
    fn shown(&self) -> u32 {
        self.2.stage_prefs.read(|p| p.shown)
    }
    fn which(&self) -> u32 {
        self.2.work.borrow().stage_direction
    }
    fn only(&self) -> u32 {
        self.1.borrow().only_object
    }
    fn faces(&self, object: u32) -> u32 {
        self.2.object_slots.face_count(object as usize)
    }
}

/// `draw` stands for `scene::draw_object`.
// Logic: `rc2k_core::scene::shown::race_shown`
pub fn draw_shown_object(
    reads: crate::scene::workers::DrawReads<'_>,
    draw: &mut dyn FnMut() -> Result<(), Refused>,
    cars: &crate::race::car_draw_state::CarDrawState,
    object: &crate::scene::object_place::ObjectPlace,
) -> Result<bool, Refused> {
    let in_race = reads.number_vars.in_race();
    if in_race && cars.borrow().object_number >= OBJECTS {
        return Err(BadInput("an object's number past the tables").into());
    }
    object.borrow_mut().drawing_object = 1u32;
    let show = if in_race {
        rc2k_core::scene::shown::race_shown(&Game(cars, object, reads))
    } else {
        // The caller's context has the menus' marks (`scene::car_marks`).
        true
    };
    if show {
        draw()?;
    }
    object.borrow_mut().drawing_object = 0u32;
    Ok(show)
}
