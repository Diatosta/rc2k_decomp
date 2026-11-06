//! `scene::draw_object_face` (0x450BDA): one face of an object, from its record to the
//! work table and on to the drawing.
//!
//! In:  ebp = the face: +0 the number of vertices, +7 a byte whose low five bits go to
//!      `dword_636608`, +9 a byte for `dword_6240D8` (how much the face reflects),
//!      +0xC a dword for `dword_6240DC`, from +0x10 a vertex every 0x10 bytes: the
//!      point's number, two texture coordinates, the normal's number
//! Out: every register preserved on the usual way; on the other (below) the original
//!      leaves them changed and the Rust version does not
//!
//! Nothing at all unless `dword_6365C4` is 0. Then, per vertex, an entry of the work table
//! (`sub_450E37`): the point's number negated (an object's point), the point's place in the
//! view copied to `dword_62DA8C` .., the coordinates, and the light: `scene::vertex_light`, or
//! plain white with `dword_6365C8` at 4, with the face's kind `dword_6D18CC` at 5, or with a
//! kind of 10 and up other than 0x1C while `dword_6D18D0` is not 0. With reflections
//! (`dword_5CCE74` and `dword_6238AC` not 0, `dword_6365C8` not 4, the face's own amount not 0)
//! each entry also gets (`sub_45188C`) a second pair of coordinates from
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
//! and `dword_6214DC` at 2 (`sub_450EE2` per vertex): the coordinates as the second pair, 0xFF
//! in the byte +0x21, the light `dword_635054`; second texture = first, mode 4, and the drawing
//! with the depth test off around it. One caller, `scene::draw_object_faces`; a second call, at
//! 0x450B9B, is in code nothing refers to. Refuses a face of no vertices (the original goes
//! through 2^32) or of more than 64, and when there are no records.

use crate::errors::{BadInput, Refused};
use crate::race::switches_store::FRAME_SWITCHES;
use crate::race::vars_store::RACE_VARS;
use crate::race::view_store::RACE_VIEW;
use crate::scene::cached_reflection_coordinates::cached_reflection_coordinates;
use crate::scene::draw_clipped_polygon::draw_clipped_polygon_in_game;
use crate::scene::draw_object_polygon::{self, draw_object_polygon};
use crate::scene::draw_polygon::{Game, Outside};
use crate::scene::owner::scene;
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
) -> Result<(), Refused> {
    let scene = scene();
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
    let view_kind: u32 = scene.lights.state.view_kind.bits();
    let shine: u32 = race.course.course.shine.bits();

    scene.drawing.scene.face_word.set_bits(fw(0xC));
    scene
        .lights
        .state
        .face_bits
        .set_bits(words[7] as u32 & 0x1F);
    let face_shine = words[9] as u32;
    scene.drawing.scene.face_shine.set_bits(face_shine);
    let reflects =
        shine != 0 && RACE_VARS.settings_words[1].bits() != 0 && view_kind != 4 && face_shine != 0;
    FRAME_SWITCHES.face_reflects.set_bits(reflects as u32);
    scene.drawing.scene.polygon_vertex_count.set_bits(count);

    if FRAME_SWITCHES.other_way.bits() != 0 && FRAME_SWITCHES.other_way_kind.bits() == 2 {
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
        RACE_VIEW.polygon_mode.set_bits(4u32);
        step(Step::DepthTestOff)?;
        step(Step::Draw)?;
        step(Step::DepthTestOn)?;
        RACE_VIEW.polygon_mode.set_bits(0u32);
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
        crate::session::write_bits(&scene.lights.work.point_place, table.place(point)?);
        entry.u.set(fw(vertex + 4));
        entry.v.set(fw(vertex + 8));
        let light = if lit {
            vertex_light(&race.course, fw(vertex + 0xC))?
        } else {
            0x00FF_FFFF
        };
        entry.light.set(light);
    }
    if reflects {
        FRAME_SWITCHES.face_any_shine.set_bits(0u32);
        for (entry, vertex) in vertices {
            race.physics
                .ground
                .reflection_normal_number
                .set_bits(fw(vertex + 0xC));
            race.physics
                .ground
                .reflection_point_number
                .set_bits(fw(vertex).wrapping_neg());
            cached_reflection_coordinates(race)?;
            let coordinates: [u32; 2] =
                crate::session::read_bits(&race.physics.ground.reflection_coordinates);
            entry.u2.set(coordinates[0]);
            entry.v2.set(coordinates[1]);
            let first_fog = entries[0].fog.get() as u32;
            let amount = (face_shine.wrapping_mul(shine).wrapping_mul(first_fog) >> 12).min(0xFF);
            if amount != 0 {
                FRAME_SWITCHES.face_any_shine.set_bits(1u32);
            }
            entry.amount.set(amount as u8);
        }
        scene.drawing.scene.polygon_textures[1].set_bits(REFLECTION_TEXTURE);
    }
    let faded = race.frame.view_and_records.view_vars.fade_alpha.bits() != 0
        && scene.lights.state.face_bits.bits() == race.physics.camera.faded_bits.bits();
    scene.lights.state.faded.set_bits(faded as u32);
    if FRAME_SWITCHES.face_any_shine.bits() == 0 {
        FRAME_SWITCHES.face_reflects.set_bits(0u32);
    }
    scene.drawing.scene.polygon_sides[0].set_bits(1u32);
    scene.drawing.scene.polygon_sides[1].set_bits(FRAME_SWITCHES.face_reflects.bits());
    let see_through: u32 = race.physics.ground.object_see_through.bits();
    if see_through != 0x100 {
        for entry in &entries[..0x10] {
            entry.amount.set(see_through as u8);
            entry.u2.set(entry.u.get());
            entry.v2.set(entry.v.get());
        }
        scene.drawing.scene.polygon_sides[0].set_bits(0u32);
        scene.drawing.scene.polygon_sides[1].set_bits(1u32);
        RACE_VIEW.polygon_mode.set_bits(3u32);
        scene.drawing.scene.polygon_textures[1]
            .set_bits(scene.drawing.scene.polygon_textures[0].bits());
    } else {
        RACE_VIEW.polygon_mode.set_bits(4u32);
    }
    step(Step::Draw)?;
    RACE_VIEW.polygon_mode.set_bits(0u32);
    scene.lights.state.faded.set_bits(0u32);
    Ok(())
}

/// The function with its parts' Rust versions, drawing for real.
pub fn draw_object_face_in_game(
    race: &crate::race::owner::Race,
    face: usize,
) -> Result<(), Refused> {
    draw_object_face(race, face, &mut |step| match step {
        Step::Draw => draw_object_polygon(race, &mut |step| match step {
            draw_object_polygon::Step::Project => project_polygon(),
            draw_object_polygon::Step::Draw => draw_clipped_polygon_in_game(),
        }),
        Step::DepthTestOff => {
            Game.state(0xE, 0);
            Ok(())
        }
        Step::DepthTestOn => {
            Game.state(0xE, 1);
            Ok(())
        }
    })
}
