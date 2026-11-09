//! `scene::draw_object_face` (0x450BDA): one face of an object, from its record to the
//! work table and on to the drawing.
//!
//! Nothing at all unless `dword_6365C4` is 0. Then, per vertex, an entry of the work table
//! (0x450E37): the point's number negated (an object's point), the point's place in the
//! view copied to `dword_62DA8C` .., the coordinates, and the light: `scene::vertex_light`, or
//! plain white with `dword_6365C8` at 4, with the face's kind `dword_6D18CC` at 5, or with a
//! kind of 10 and up other than 0x1C while `dword_6D18D0` is not 0. With reflections
//! (`dword_5CCE74` and `dword_6238AC` not 0, `dword_6365C8` not 4, the face's own amount not 0)
//! each entry also gets (0x45188C) a second pair of coordinates from
//! `scene::cached_reflection_coordinates` and, in its byte +0x21, the face's amount times
//! `dword_5CCE74` times the first entry's byte +0x20, over 0x1000, held to 255; `dword_621504`
//! says whether any of them was not 0, and the second texture is 0x87. `dword_621504` is only
//! cleared when this part runs: a face without it goes by what an earlier one left. Then the
//! first pass's fade (`dword_63660C`, when `dword_606284` is set and the face's five bits are
//! `dword_608CAC`), the two switches of `scene::draw_polygon` (the first texture always, the
//! second with a reflection), mode 4; or, with `dword_639970` not 0x100, the first sixteen
//! entries get that as their byte +0x21 and their first coordinates as their second, the
//! switches the other way round, the second texture = the first, mode 3. Then
//! `scene::draw_object_polygon`, mode 0, the fade off. The other way, with `dword_621480` set
//! and `dword_6214DC` at 2 (0x450EE2 per vertex): the coordinates as the second pair, 0xFF
//! in the byte +0x21, the light `dword_635054`; second texture = first, mode 4, and the drawing
//! with the depth test off around it. One caller, `scene::draw_object_faces`; a second call, at
//! 0x450B9B, is in code nothing refers to. Refuses a face of no vertices (the original goes
//! through 2^32) or of more than 64, and when there are no records.

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

// ---- Rust version ----

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Step {
    Draw,
    DepthTestOff,
    DepthTestOn,
}

pub fn draw_object_face(
    race: &crate::race::owner::Race,
    face: usize,
    step: &mut dyn FnMut(Step) -> Result<(), Refused>,
    scene: &crate::scene::owner::Scene,
    light: &crate::scene::object_light::ObjectLight,
    system: &crate::system::owner::System,
    object: &crate::scene::object_place::ObjectPlace,
) -> Result<(), Refused> {
    if scene.lights.state.draw_mode.bits() != 0 {
        return Ok(());
    }
    if face == 0 {
        return Err(BadInput("no face").into());
    }
    let count: u32 = crate::scene::object_faces::word(face as u32)?;
    if count == 0 || count > MOST || !crate::scene::point_records::held() {
        return Err(BadInput("a face of no vertices or more than 64, or no records").into());
    }
    // The whole face in one read: at most `MOST` vertices.
    let mut whole: smallvec::SmallVec<[u8; 0x10 + 8 * 0x10]> =
        smallvec::smallvec![0; 0x10 + count as usize * 0x10];
    crate::scene::object_faces::read_into(face as u32, &mut whole[..])?;
    let words = &whole;
    let fw = |at: usize| u32::from_le_bytes(words[at..at + 4].try_into().unwrap());
    let entries = &crate::scene::polygon_store::POLYGON_WORK.entries;
    let vertices = (0..count as usize).map(|n| (&entries[n], 0x10 + n * 0x10));
    let view_kind: u32 = light.view_kind.bits();
    let shine: u32 = race.course.course.shine.bits();

    scene.drawing.scene.face_word.set_bits(fw(0xC));
    light.face_bits.set_bits(words[7] as u32 & 0x1F);
    let face_shine = words[9] as u32;
    scene.drawing.scene.face_shine.set_bits(face_shine);
    let reflects = shine != 0
        && race.shared.vars.settings_words[1].bits() != 0
        && view_kind != 4
        && face_shine != 0;
    race.shared.switches.face_reflects.set_bits(reflects as u32);
    scene.drawing.scene.polygon_vertex_count.set_bits(count);

    if race.shared.switches.other_way.bits() != 0 && race.shared.switches.other_way_kind.bits() == 2
    {
        for (entry, vertex) in vertices {
            entry.point.set(fw(vertex).wrapping_neg());
            entry.u2.set(fw(vertex + 4));
            entry.v2.set(fw(vertex + 8));
            entry.amount.set(0xFF);
            entry
                .light
                .set(scene.drawing.object_work.other_light.bits());
        }
        scene.drawing.scene.polygon_textures[1]
            .set_bits(scene.drawing.scene.polygon_textures[0].bits());
        scene.drawing.scene.polygon_sides[0].set_bits(0u32);
        scene.drawing.scene.polygon_sides[1].set_bits(1u32);
        race.shared.view.polygon_mode.set_bits(4u32);
        step(Step::DepthTestOff)?;
        step(Step::Draw)?;
        step(Step::DepthTestOn)?;
        race.shared.view.polygon_mode.set_bits(0u32);
        return Ok(());
    }

    let (face_kind, face_unlit): (u32, u32) = (
        race.frame.sound_spots.face_kind.bits(),
        race.frame.sound_spots.face_unlit.bits(),
    );
    let lit = view_kind != 4
        && (face_kind == 0x1C || (face_kind != 5 && (face_kind < 0xA || face_unlit == 0)));
    let table = crate::scene::reading::PointRecords::new()?;
    for (entry, vertex) in vertices.clone() {
        let point = fw(vertex);
        entry.point.set(point.wrapping_neg());
        crate::session::write_bits(&light.point_place, table.place(point)?);
        entry.u.set(fw(vertex + 4));
        entry.v.set(fw(vertex + 8));
        let light = if lit {
            vertex_light(race, fw(vertex + 0xC), scene, light, system)?
        } else {
            0x00FF_FFFF
        };
        entry.light.set(light);
    }
    if reflects {
        race.shared.switches.face_any_shine.set_bits(0u32);
        for (entry, vertex) in vertices {
            object.reflection_normal_number.set_bits(fw(vertex + 0xC));
            object
                .reflection_point_number
                .set_bits(fw(vertex).wrapping_neg());
            cached_reflection_coordinates(race, object)?;
            let coordinates: [u32; 2] = crate::session::read_bits(&object.reflection_coordinates);
            entry.u2.set(coordinates[0]);
            entry.v2.set(coordinates[1]);
            let first_fog = entries[0].fog.get() as u32;
            let amount = (face_shine.wrapping_mul(shine).wrapping_mul(first_fog) >> 12).min(0xFF);
            if amount != 0 {
                race.shared.switches.face_any_shine.set_bits(1u32);
            }
            entry.amount.set(amount as u8);
        }
        scene.drawing.scene.polygon_textures[1].set_bits(REFLECTION_TEXTURE);
    }
    let faded = race.frame.fade_alpha.bits() != 0
        && light.face_bits.bits() == race.physics.camera.faded_bits.bits();
    scene.lights.state.faded.set_bits(faded as u32);
    if race.shared.switches.face_any_shine.bits() == 0 {
        race.shared.switches.face_reflects.set_bits(0u32);
    }
    scene.drawing.scene.polygon_sides[0].set_bits(1u32);
    scene.drawing.scene.polygon_sides[1].set_bits(race.shared.switches.face_reflects.bits());
    let see_through: u32 = object.see_through.bits();
    if see_through != 0x100 {
        for entry in &entries[..0x10] {
            entry.amount.set(see_through as u8);
            entry.u2.set(entry.u.get());
            entry.v2.set(entry.v.get());
        }
        scene.drawing.scene.polygon_sides[0].set_bits(0u32);
        scene.drawing.scene.polygon_sides[1].set_bits(1u32);
        race.shared.view.polygon_mode.set_bits(3u32);
        scene.drawing.scene.polygon_textures[1]
            .set_bits(scene.drawing.scene.polygon_textures[0].bits());
    } else {
        race.shared.view.polygon_mode.set_bits(4u32);
    }
    step(Step::Draw)?;
    race.shared.view.polygon_mode.set_bits(0u32);
    scene.lights.state.faded.set_bits(0u32);
    Ok(())
}

/// The function with its parts' Rust versions, drawing for real.
pub fn draw_object_face_in_game(
    ctx: &crate::race::view_scratch::DrawCtx<'_>,
    face: usize,
    system: &crate::system::owner::System,
) -> Result<(), Refused> {
    let (race, scene) = (ctx.race, ctx.scene);
    draw_object_face(
        race,
        face,
        &mut |step| match step {
            Step::Draw => draw_object_polygon(race, &ctx.view.object, &mut |step| match step {
                draw_object_polygon::Step::Project => project_polygon(race, scene),
                draw_object_polygon::Step::Draw => draw_clipped_polygon_in_game(ctx),
            }),
            Step::DepthTestOff => {
                Game.state(0xE, 0);
                Ok(())
            }
            Step::DepthTestOn => {
                Game.state(0xE, 1);
                Ok(())
            }
        },
        scene,
        &ctx.view.light,
        system,
        &ctx.view.object,
    )
}
