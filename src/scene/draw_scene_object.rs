//! `scene::draw_scene_object`: chooses the set of textures for the current object and draws it the
//! way the scene's mode asks for.
//!
//! An object's number of 0x100 or more is fatal error 0x80600A01. `GroundWork.object_texture_set`
//! (added to every face's texture number) = 0; then 0x108 for, outside a race, an object of kind
//! 0x23A, and in a race for objects 0x50 .. 0xEF (the cars, 0x14 objects each) whose car's block
//! has the value 0x21 in its car-kind field. Nothing more for an object without faces (the slots'
//! face counts). Else by the lights' draw mode: 0 `scene::draw_shown_object`; 1 and 2
//! `scene::draw_described_object` (it takes an object's description and comes back here); anything
//! else nothing. Refuses a car without its block.
//!
//! The logic is in the core crate: the texture set and the way on through
//! `rc2k_core::scene::shown`.

use crate::errors::{BadInput, FatalError, Refused};
use crate::scene::draw_object::draw_object_in_game;
use crate::scene::draw_shown_object::draw_shown_object;

// ---- game data ----

const TOO_HIGH: FatalError = FatalError {
    code: 0x8060_0A01,
    message: "object number too high",
};

/// The two ways on from here.
pub use rc2k_core::scene::shown::Way;

/// The texture set for the current object; the object's number.
// Logic: `rc2k_core::scene::shown::texture_set`
fn texture_set(
    reads: crate::scene::workers::DrawReads<'_>,
    cars: &crate::race::car_draw_state::CarDrawState,
) -> Result<(u32, u32), Refused> {
    use rc2k_core::scene::shown::SetError;
    {
        let object: u32 = cars.object_number.get();
        let t = rc2k_core::scene::shown::texture_set(
            object,
            reads.numbers.in_race.is_set(),
            |object| reads.object_tables.object_kind(object),
            |car| {
                reads.work.has_car_block(car as usize).then(|| {
                    crate::race::owner_shared::RaceShared::race_car_or_end_in(
                        reads.work,
                        reads.car_blocks,
                        car as usize,
                    )
                    .car_kind
                    .get()
                })
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
// Logic: `rc2k_core::scene::shown::way`
pub fn draw_scene_object(
    draw: &mut dyn FnMut(Way) -> Result<(), Refused>,
    reads: crate::scene::workers::DrawReads<'_>,
    cars: &crate::race::car_draw_state::CarDrawState,
    place: &crate::scene::object_place::ObjectPlace,
) -> Result<(), Refused> {
    let (set, object) = texture_set(reads, cars)?;
    place.texture_set.set(set);
    let faces: u32 = reads.object_slots.face_counts[object as usize].get();
    match rc2k_core::scene::shown::way(faces, reads.light_work.draw_mode) {
        Some(way) => draw(way),
        None => Ok(()),
    }
}

/// `draw_scene_object` with the game's parts (`scene::draw_shown_object` and its drawing;
/// `scene::draw_described_object`, given `description`: the object's description, read only on
/// that way, the scene's mode 1 or 2; `None` where there is none).
pub fn draw_scene_object_in_game(
    ctx: &crate::race::view_scratch::DrawCtx<'_>,
    description: Option<crate::scene::draw_described_object::Description<'_>>,
) -> Result<(), Refused> {
    draw_scene_object(
        &mut |way| match way {
            Way::Shown => {
                // Outside a race the faces pick the menus' marks (`scene::car_marks`).
                let ctx = &if ctx.reads.numbers.in_race.is_set() {
                    *ctx
                } else {
                    crate::race::view_scratch::DrawCtx {
                        marks: &crate::scene::car_marks::MENUS,
                        ..*ctx
                    }
                };
                draw_shown_object(
                    ctx.reads,
                    &mut || draw_object_in_game(ctx),
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
