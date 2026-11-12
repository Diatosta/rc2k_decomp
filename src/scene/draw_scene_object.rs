//! `scene::draw_scene_object` (0x451E3E): chooses the set of textures for the object
//! `g_objectNumber` and draws it the way the scene's mode asks for.
//!
//! An object's number of 0x100 or more is fatal error 0x80600A01. `GroundWork.object_texture_set` (added to
//! every face's texture number) = 0; then 0x108 for, outside a race, an object of kind 0x23A,
//! and in a race for objects 0x50 .. 0xEF (the cars, 0x14 objects each) whose car's block has
//! 0x21 at +0x24. Nothing more for an object without faces (the slots' face counts). Else by
//! the lights' draw mode: 0 `scene::draw_shown_object`; 1 and 2 `scene::draw_described_object` (it
//! takes an object's description and comes back here); anything else nothing. Twelve
//! callers. Refuses a car without its block.
//!
//! The logic is in the core crate: the texture set and the way on through
//! `rc2k_core::scene::shown`.

use crate::errors::{BadInput, FatalError, Refused};
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
fn texture_set(
    race: &crate::race::owner::Race,
    reads: crate::workers::DrawReads<'_>,
    cars: &crate::race::car_draw_state::CarDrawState,
) -> Result<(u32, u32), Refused> {
    let shared = &race.shared;
    use rc2k_core::scene::shown::SetError;
    {
        let object: u32 = cars.object_number.get();
        let t = rc2k_core::scene::shown::texture_set(
            object,
            race.course.numbers.in_race.is_set(),
            |object| reads.object_tables.object_kind(object),
            |car| {
                shared
                    .work
                    .has_car_block(car as usize)
                    .then(|| shared.race_car_or_end(car as usize).car_kind.get())
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

/// `draw` stands for the two calls.
// Converted: `rc2k_core::scene::shown::way`
pub fn draw_scene_object(
    race: &crate::race::owner::Race,
    draw: &mut dyn FnMut(Way) -> Result<(), Refused>,
    reads: crate::workers::DrawReads<'_>,
    cars: &crate::race::car_draw_state::CarDrawState,
    place: &crate::scene::object_place::ObjectPlace,
) -> Result<(), Refused> {
    let (set, object) = texture_set(race, reads, cars)?;
    place.texture_set.set(set);
    let faces: u32 = reads.object_slots.face_counts[object as usize].get();
    match rc2k_core::scene::shown::way(faces, reads.light_state.draw_mode.get()) {
        Some(way) => draw(way),
        None => Ok(()),
    }
}

/// `draw_scene_object` with the game's parts (`scene::draw_shown_object` and its drawing;
/// `scene::draw_described_object`, given `description`: the
/// object's description, read only on that way, the scene's mode 1 or 2; `None` where the
/// original has none).
pub fn draw_scene_object_in_game(
    ctx: &crate::race::view_scratch::DrawCtx<'_>,
    description: Option<crate::scene::draw_described_object::Description<'_>>,
) -> Result<(), Refused> {
    let (race, system) = (ctx.race, ctx.system);
    draw_scene_object(
        race,
        &mut |way| match way {
            Way::Shown => {
                // Outside a race the faces pick the menus' marks (the original's
                // `draw_shown_object` set them before every object: `scene::car_marks`).
                let ctx = &if race.course.numbers.in_race.is_set() {
                    *ctx
                } else {
                    crate::race::view_scratch::DrawCtx {
                        marks: &crate::scene::car_marks::MENUS,
                        ..*ctx
                    }
                };
                draw_shown_object(
                    race,
                    ctx.reads,
                    &mut || draw_object_in_game(ctx, system),
                    &ctx.view.cars,
                    &ctx.view.object,
                )
                .map(|_| ())
            }
            Way::Described => {
                crate::scene::draw_described_object::draw_described_object(ctx, description)
            }
        },
        ctx.reads,
        &ctx.view.cars,
        &ctx.view.object,
    )
}
