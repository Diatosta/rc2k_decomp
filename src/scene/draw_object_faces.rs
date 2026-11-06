//! `scene::draw_object_faces` (0x450A4B): draws the faces of an object that were listed
//! for drawing.
//!
//! In:  nothing
//! Out: nothing changed with an empty list; else eax, ecx, esi and ebp are the loop's
//!      leftovers (the last face's number in the table, the count, the last list
//!      entry, the last face); everything else preserved
//!
//! The list: `dword_62DB58` entries of 0x10 bytes at `dword_62DB5C`: +0 the face's number
//! within the object, +0xC how see-through it is (to `g_objectSeeThrough`). For each: the
//! number to `dword_623620`; the face is entry number + `dword_6350BC` of the table
//! `dword_9A3730`; from the face's head the first texture (the word +4 plus `dword_639954`,
//! with the top bit set), its kind (byte +6, `g_faceKind`) and byte +0xB (`dword_6D18D0`); the
//! polygon's colour is white, or with a byte +0xA not 0 that byte in all three places;
//! `scene::draw_object_face`; the colour white again. At the end `dword_624204` = 0. One
//! caller, `sub_45045D`. Refuses a face's number past the table and a face that is not there.

use crate::errors::{BadInput, Refused};
use crate::race::vars_store::RACE_VARS;
use crate::scene::owner::scene;

// ---- game data ----

const LIST_ENTRY_LEN: usize = 0x10;
const FACES: u32 = 0xC000;

// ---- Rust version ----

/// Dword `n` of the list of faces (the original reads on past its end).
fn listed_dword(n: usize) -> Result<u32, BadInput> {
    let objects = &scene().objects;
    crate::session::read_on_in(
        &objects.slots,
        &objects.slots.listed_faces,
        n,
        "the list of faces",
    )
}

pub fn draw_object_faces(
    race: &crate::race::owner::Race,
    draw: &mut dyn FnMut(usize) -> Result<(), Refused>,
) -> Result<(), Refused> {
    let scene = scene();
    let listed: u32 = scene.lights.work.listed_face_count.bits();
    if listed == 0 {
        return Ok(());
    }
    let first: u32 = scene.drawing.object_work.object_first_face.bits();
    // A face not there ends the game (a refusal), as the original went wrong on it.
    for n in 0..listed as usize {
        let entry = n * LIST_ENTRY_LEN / 4;
        let number = listed_dword(entry)?.wrapping_add(first);
        if number >= FACES {
            return Err(BadInput("a face's number past the table").into());
        }
        let face = crate::scene::objects::face_cell(number as usize).get() as usize;
        if face == 0 {
            return Err(BadInput("a face that is not there").into());
        }
        race.physics
            .ground
            .object_see_through
            .set_bits(listed_dword(entry + 3)?);
        RACE_VARS.face_number.set_bits(listed_dword(entry)?);
        let head: [u8; 0xC] = crate::scene::object_faces::array(face as u32)?;
        let texture = u16::from_le_bytes([head[4], head[5]]) as u32;
        scene.drawing.scene.polygon_textures[0].set_bits(
            texture.wrapping_add(race.physics.ground.object_texture_set.bits()) | 0x8000_0000,
        );
        race.frame.sound_spots.face_kind.set_bits(head[6] as u32);
        race.frame.sound_spots.face_unlit.set_bits(head[0xB] as u32);
        let shade = head[0xA] as u32;
        scene.drawing.scene.polygon_colour.set_bits(if shade == 0 {
            0x00FF_FFFF
        } else {
            0x0001_0101 * shade
        });
        draw(face)?;
        scene.drawing.scene.polygon_colour.set_bits(0x00FF_FFFFu32);
    }
    scene.drawing.scene.faces_done.set_bits(0u32);
    Ok(())
}
