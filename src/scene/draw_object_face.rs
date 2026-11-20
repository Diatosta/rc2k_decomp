//! `scene::draw_object_face`: one face of an object, from its record to the work table and on
//! to the drawing.
//!
//! Nothing at all unless the lights' `draw_mode` is 0. Then, per vertex, an entry of the work
//! table: the point's number negated (an object's point), the point's place in the view copied to
//! the light's `point_place`, the coordinates, and the light: `scene::vertex_light`, or plain white
//! with the `view_kind` at 4, with the face's kind (`face_kind`) at 5, or with a kind of 10 and up
//! other than 0x1C while `face_unlit` is not 0. With reflections (the course's `shine` and the
//! second `settings_words` entry not 0, `view_kind` not 4, the face's own shine not 0) each entry
//! also gets a second pair of coordinates from `scene::cached_reflection_coordinates` and, in its
//! `amount`, the face's shine times `shine` times the face's first vertex's fog (as
//! `scene::fog_polygon` will give it), over 0x1000, held to 255; `face_any_shine` says whether any
//! of them was not 0, and the second texture is 0x87. `face_any_shine` is only cleared when this
//! part runs: a face without reflections goes by what an earlier one left. Then the first pass's
//! fade (`faded`, when `fade_alpha` is set and the face's five bits are `faded_bits`), the two
//! sides of `scene::draw_polygon` (the first texture always, the second with a reflection), mode 4;
//! or, with `see_through` not 0x100, the first sixteen entries get that as their `amount` and their
//! first coordinates as their second, the sides the other way round, the second texture = the
//! first, mode 3. Then `scene::draw_object_polygon`, mode 0, the fade off. The other way, with
//! `scene_set_up` set and `other_way_kind` at 2 (per vertex): the coordinates as the second pair,
//! 0xFF in the `amount`, the light `other_light`; second texture = first, mode 4, and the drawing
//! with the depth test off around it. Refuses a face of no vertices or of more than 64, and when
//! there are no records.

use crate::errors::{BadInput, Refused};
use crate::scene::cached_reflection_coordinates::cached_reflection_coordinates;
use crate::scene::draw_clipped_polygon::draw_clipped_polygon_in_game;
use crate::scene::draw_object_polygon::{self, draw_object_polygon};
use crate::scene::draw_polygon::{Game, Outside};
use crate::scene::project_polygon::project_polygon;
use crate::scene::vertex_light::vertex_light;

// ---- game data ----
const REFLECTION_TEXTURE: u32 = 0x87;
/// What `scene::draw_object_polygon` can keep.
const MOST: u32 = 0x40;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Step {
    Draw,
    DepthTestOff,
    DepthTestOn,
}

pub fn draw_object_face(
    ctx: &crate::race::view_scratch::DrawCtx<'_>,
    face: usize,
    step: &mut dyn FnMut(Step) -> Result<(), Refused>,
) -> Result<(), Refused> {
    let (work, light, object) = (&ctx.view.face, &ctx.view.light, &ctx.view.object);
    if ctx.reads.light_state.draw_mode.get() != 0 {
        return Ok(());
    }
    if face == 0 {
        return Err(BadInput("no face").into());
    }
    let count: u32 = ctx.reads.stage.object_faces.word(face as u32)?;
    if count == 0 || count > MOST || !crate::scene::point_records::held(ctx.reads.course_lists) {
        return Err(BadInput("a face of no vertices or more than 64, or no records").into());
    }
    // The whole face in one read: at most `MOST` vertices, so never on the heap.
    let mut whole: smallvec::SmallVec<[u8; 0x10 + MOST as usize * 0x10]> =
        smallvec::smallvec![0; 0x10 + count as usize * 0x10];
    ctx.reads
        .stage
        .object_faces
        .read_into(face as u32, &mut whole[..])?;
    let words = &whole;
    let fw = |at: usize| u32::from_le_bytes(words[at..at + 4].try_into().unwrap());
    let entries = &ctx.part.polygon.entries;
    let vertices = (0..count as usize).map(|n| (&entries[n], 0x10 + n * 0x10));
    let view_kind: u32 = light.view_kind.get();
    let shine: u32 = ctx.view.course.shine.get();

    light.face_bits.set(words[7] as u32 & 0x1F);
    let face_shine = words[9] as u32;
    let reflects = shine != 0
        && ctx.reads.vars.settings_words[1].get() != 0
        && view_kind != 4
        && face_shine != 0;
    ctx.view.face.reflects.set(reflects as u32);
    ctx.view.polygon.polygon_vertex_count.set(count);

    if ctx.reads.switches.scene_set_up.get() != 0 && ctx.view.cars.other_way_kind.get() == 2 {
        for (entry, vertex) in vertices {
            entry.point.set(fw(vertex).wrapping_neg());
            entry.u2.set(fw(vertex + 4));
            entry.v2.set(fw(vertex + 8));
            entry.amount.set(0xFF);
            entry.light.set(light.other_light.get());
        }
        ctx.view.polygon.polygon_textures[1].set(ctx.view.polygon.polygon_textures[0].get());
        ctx.view.polygon.polygon_sides[0].set(0u32);
        ctx.view.polygon.polygon_sides[1].set(1u32);
        work.polygon_mode.set(4u32);
        step(Step::DepthTestOff)?;
        step(Step::Draw)?;
        step(Step::DepthTestOn)?;
        work.polygon_mode.set(0u32);
        return Ok(());
    }

    let (face_kind, face_unlit): (u32, u32) = (work.face_kind.get(), work.face_unlit.get());
    let lit = view_kind != 4
        && (face_kind == 0x1C || (face_kind != 5 && (face_kind < 0xA || face_unlit == 0)));
    let table = crate::scene::reading::PointRecords::new(ctx.reads.course_lists, ctx.part)?;
    for (entry, vertex) in vertices.clone() {
        let point = fw(vertex);
        entry.point.set(point.wrapping_neg());
        crate::memory::session::write_bits(&light.point_place, table.place(point)?);
        entry.u.set(fw(vertex + 4));
        entry.v.set(fw(vertex + 8));
        let light = if lit {
            vertex_light(ctx.part, fw(vertex + 0xC), ctx.reads, light, object)?
        } else {
            0x00FF_FFFF
        };
        entry.light.set(light);
    }
    if reflects {
        ctx.view.face.face_any_shine.set(0u32);
        // The shine is scaled by the face's own fog: its first vertex's, as `scene::fog_polygon`
        // gives it once the face is projected (fixed: "A reflective face's shine scaled by the
        // previous polygon's fog" in `wiki/Fixed-bugs.md`).
        let first = crate::scene::project_point::project_point(
            ctx.part,
            fw(0x10).wrapping_neg(),
            ctx.reads,
            ctx.view,
        )?
        .record;
        let first_fog = u32::from(crate::scene::fog_polygon::fog_of_record(
            ctx.part, ctx.view, first,
        )?);
        for (entry, vertex) in vertices {
            object.reflection_normal_number.set(fw(vertex + 0xC));
            object
                .reflection_point_number
                .set(fw(vertex).wrapping_neg());
            cached_reflection_coordinates(
                ctx.part,
                ctx.reads,
                object,
                &ctx.view.matrices,
                ctx.view,
            )?;
            let coordinates: [u32; 2] =
                crate::memory::session::read_bits(&object.reflection_coordinates);
            entry.u2.set(coordinates[0]);
            entry.v2.set(coordinates[1]);
            let amount = (face_shine.wrapping_mul(shine).wrapping_mul(first_fog) >> 12).min(0xFF);
            if amount != 0 {
                ctx.view.face.face_any_shine.set(1u32);
            }
            entry.amount.set(amount as u8);
        }
        ctx.view.polygon.polygon_textures[1].set(REFLECTION_TEXTURE);
    }
    let faded = work.fade_alpha.get() != 0 && light.face_bits.get() == work.faded_bits.get();
    work.faded.set(faded as u32);
    if ctx.view.face.face_any_shine.get() == 0 {
        ctx.view.face.reflects.set(0u32);
    }
    ctx.view.polygon.polygon_sides[0].set(1u32);
    ctx.view.polygon.polygon_sides[1].set(ctx.view.face.reflects.get());
    let see_through: u32 = object.see_through.get();
    if see_through != 0x100 {
        for entry in &entries[..0x10] {
            entry.amount.set(see_through as u8);
            entry.u2.set(entry.u.get());
            entry.v2.set(entry.v.get());
        }
        ctx.view.polygon.polygon_sides[0].set(0u32);
        ctx.view.polygon.polygon_sides[1].set(1u32);
        work.polygon_mode.set(3u32);
        ctx.view.polygon.polygon_textures[1].set(ctx.view.polygon.polygon_textures[0].get());
    } else {
        work.polygon_mode.set(4u32);
    }
    step(Step::Draw)?;
    work.polygon_mode.set(0u32);
    work.faded.set(0u32);
    Ok(())
}

/// The function with its parts, drawing for real.
pub fn draw_object_face_in_game(
    ctx: &crate::race::view_scratch::DrawCtx<'_>,
    face: usize,
) -> Result<(), Refused> {
    draw_object_face(ctx, face, &mut |step| match step {
        Step::Draw => draw_object_polygon(
            ctx.part,
            &ctx.view.polygon,
            ctx.reads.course(),
            &ctx.view.object,
            &mut |step| match step {
                draw_object_polygon::Step::Project => {
                    project_polygon(ctx.part, &ctx.view.polygon, ctx.reads, ctx.view)
                }
                draw_object_polygon::Step::Draw => draw_clipped_polygon_in_game(ctx),
            },
        ),
        Step::DepthTestOff => {
            Game(ctx.part, ctx.view).state(0xE, 0);
            Ok(())
        }
        Step::DepthTestOn => {
            Game(ctx.part, ctx.view).state(0xE, 1);
            Ok(())
        }
    })
}
