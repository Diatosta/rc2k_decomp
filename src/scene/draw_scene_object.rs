//! `scene::draw_scene_object` (0x451E3E): chooses the set of textures for the object
//! `g_objectNumber` and draws it the way the scene's mode asks for.
//!
//! In:  esi, passed on to `scene::draw_described_object`
//! Out: the registers as its parts leave them; before the drawing eax = what was last
//!      looked at, ebx = the object's number, edx = the car's remainder if a car was
//!      looked at
//!
//! An object's number of 0x100 or more is fatal error 0x80600A01. `dword_639954` (added to
//! every face's texture number) = 0; then 0x108 for, outside a race, an object of kind 0x23A,
//! and in a race for objects 0x50 .. 0xEF (the cars, 0x14 objects each) whose car's block has
//! 0x21 at +0x24. Nothing more for an object without faces (`g_objectFaceCounts`). Else by
//! `dword_6365C4`: 0 `scene::draw_shown_object`; 1 and 2 `scene::draw_described_object` (it
//! takes an object's description in esi and comes back here); anything else nothing. Twelve
//! callers. Refuses a car without its block.
//!
//! Converted 2026-10-21: the texture set and the way on through
//! `rc2k_core::scene::shown`.

use crate::errors::{BadInput, FatalError, Refused};
use crate::race::owner::race;
use crate::race::switches_store::FRAME_SWITCHES;
use crate::race::work_store::RACE_WORK;
use crate::scene::draw_object::draw_object_in_game;
use crate::scene::draw_shown_object::draw_shown_object;

// ---- game data ----
// scene::draw_described_object

const TOO_HIGH: FatalError = FatalError {
    code: 0x8060_0A01,
    message: "object number too high",
};

/// The two ways on from here.
pub use rc2k_core::scene::shown::Way;

// ---- Rust version ----

/// The texture set for `g_objectNumber`; the object's number.
// Converted: `rc2k_core::scene::shown::texture_set`
fn texture_set(course: &crate::race::owner_course::Course) -> Result<(u32, u32), Refused> {
    use rc2k_core::scene::shown::SetError;
    {
        let object: u32 = FRAME_SWITCHES.object_number.bits();
        let t = rc2k_core::scene::shown::texture_set(
            object,
            course.numbers.in_race.bits() != 0,
            |object| {
                crate::scene::owner::scene()
                    .objects
                    .tables
                    .object_kind(object)
            },
            |car| {
                let block: u32 = RACE_WORK.car_block(car as usize);
                (block != 0).then(|| crate::physics::car_store::Car::at(block).car_kind.bits())
            },
        )
        .map_err(|e| -> Refused {
            match e {
                SetError::TooHigh => TOO_HIGH.into(),
                SetError::NoBlock => BadInput("a car without its block").into(),
            }
        });
        Ok((t?, object))
    }
}

/// `draw`: what stands for the two calls. What the original leaves in the registers (ebx
/// the object's number among them) none of its callers reads (register liveness,
/// 2026-10-23: only the unreached `sub_49081A` after `scene::draw_menu_object`).
// Converted: `rc2k_core::scene::shown::way`
pub fn draw_scene_object(
    race: &crate::race::owner::Race,
    draw: &mut dyn FnMut(Way) -> Result<(), Refused>,
) -> Result<(), Refused> {
    let (set, object) = texture_set(&race.course)?;
    race.physics.ground.object_texture_set.set_bits(set);
    let faces: u32 = crate::scene::owner::scene().objects.slots.face_counts[object as usize].get();
    match rc2k_core::scene::shown::way(
        faces,
        crate::scene::owner::scene().lights.state.draw_mode.bits(),
    ) {
        Some(way) => draw(way),
        None => Ok(()),
    }
}

/// `draw_scene_object` with the game's parts (`scene::draw_shown_object` and its drawing;
/// `scene::draw_described_object`, given `description`, esi at the original's entry: the
/// object's description, read only on that way, the scene's mode 1 or 2; `None` where the
/// original's esi is 0).
pub fn draw_scene_object_in_game(
    description: Option<crate::pointers::GamePtr>,
) -> Result<(), Refused> {
    let race = race();
    let course = &race.course;
    draw_scene_object(race, &mut |way| match way {
        Way::Shown => draw_shown_object(course, &mut || draw_object_in_game(race)).map(|_| ()),
        Way::Described => {
            crate::scene::draw_described_object::draw_described_object(course, description)
        }
    })
}
