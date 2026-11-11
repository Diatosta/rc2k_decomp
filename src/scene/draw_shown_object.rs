//! `scene::draw_shown_object` (0x4503A5): draws the object `g_objectNumber` unless a
//! race's tests hide it.
//!
//! `dword_62DA44` = 1 for the duration, 0 at the end.
//! In a race (`g_inRace` not 0) two tests, each of which can hide the object:
//! - `scene::object_hidden_by_kind` (0x4522EA over 0x452303, inlined): with
//!   `dword_71C3E0` not 0 or `dword_62389C` at 2, an object of kind 0x30 or 0x4E;
//! - `scene::object_hidden` (0x4503DD, inlined; leaves ebx = the object's number):
//!   objects 0x38 and 0x3A .. 0x3F are hidden with `dword_621480` set, with
//!   `dword_609330` at 0, object 0x3E with `dword_71B400` not 0 and object 0x3D with it
//!   not 1; with `dword_621480` set and `dword_635058` not 0 (eax), every object unless
//!   `dword_635058` - 1 is `dword_71B400`; and an object without faces
//!   (`g_objectFaceCounts`).
//!
//! Outside a race `scene::clear_kind_bytes` (0x46219C, inlined): the 0x100 dwords at
//! `dword_5CD688` = 0, then the seventh, eighth and tenth = 999 (eax = 0, ecx = 0, edi = the
//! table's end): here the caller's context carries the menus' marks instead
//! (`scene::car_marks::MENUS`). Then `scene::draw_object`. One caller, `scene::draw_scene_object`. Refuses, in a
//! race, an object's number of 0x100 or more (the original reads past its tables).
//!
//! Converted 2026-10-21: the race's tests and the kind bytes through
//! `rc2k_core::scene::shown` (the variables read where the tests read them).

use crate::errors::{BadInput, Refused};

// ---- game data ----
use crate::race::owner::Race;
use crate::scene::owner::scene;
use rc2k_core::scene::shown::OBJECTS;

// ---- Rust version ----

/// The game's variables for the race's tests.
struct Game<'a>(
    &'a Race,
    &'a crate::race::car_draw_state::CarDrawState,
    &'a crate::scene::object_place::ObjectPlace,
);

impl rc2k_core::scene::shown::RaceObject for Game<'_> {
    fn object(&self) -> u32 {
        self.1.object_number.get()
    }
    fn kind_switch(&self) -> u32 {
        self.0.shared.work.in_credits.get()
    }
    fn kind_mode(&self) -> u32 {
        self.0.shared.vars.run_kind.get()
    }
    fn kind(&self, object: u32) -> u32 {
        scene().objects.tables.object_kind(object)
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
        scene().objects.slots.face_counts[object as usize].get()
    }
}

/// `draw`: what stands for `scene::draw_object`. Says whether the object was drawn. What
/// the tests leave in the registers `draw_object` does not read.
// Converted: `rc2k_core::scene::shown::race_shown`
pub fn draw_shown_object(
    race: &Race,
    draw: &mut dyn FnMut() -> Result<(), Refused>,
    cars: &crate::race::car_draw_state::CarDrawState,
    object: &crate::scene::object_place::ObjectPlace,
) -> Result<bool, Refused> {
    let in_race = race.course.numbers.in_race.get() != 0;
    if in_race && cars.object_number.get() >= OBJECTS {
        return Err(BadInput("an object's number past the tables").into());
    }
    object.drawing_object.set(1u32);
    let show = if in_race {
        rc2k_core::scene::shown::race_shown(&Game(race, cars, object))
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
