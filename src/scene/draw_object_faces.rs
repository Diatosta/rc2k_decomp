//! `scene::draw_object_faces` (0x450A4B): draws the faces of an object that were listed
//! for drawing.
//!
//! The list: the view's listed faces, entries of 0x10 bytes: +0 the face's number within the
//! object, +0xC how see-through it is (to the object's `see_through`). For each: the number
//! to the view's face number; the face is entry number + the object's first face of the faces
//! table; from the face's head the first texture (the word +4 plus the object's texture
//! base, with the top bit set), its kind (byte +6) and byte +0xB; the polygon's colour is
//! white, or with a byte +0xA not 0 that byte in all three places; `scene::draw_object_face`;
//! the colour white again. At the end a clip work cell is 0. One
//! //! caller, `scene::draw_object`. Refuses a face's number past the table and a face that is not
//! there.

use crate::errors::{BadInput, Refused};

// ---- game data ----

const FACES: u32 = 0xC000;

// ---- Rust version ----

pub fn draw_object_faces(
    poly: &crate::scene::polygon_work::PolygonWork,
    object: &crate::scene::object_place::ObjectPlace,
    work: &crate::race::face_work::FaceWork,
    list: &crate::scene::face_list::FaceList,
    draw: &mut dyn FnMut(usize) -> Result<(), Refused>,
) -> Result<(), Refused> {
    if list.is_empty() {
        return Ok(());
    }
    let first: u32 = object.first_face.get();
    // A face not there ends the game (a refusal), as the original went wrong on it.
    for listed in list {
        let number = listed[0].wrapping_add(first);
        if number >= FACES {
            return Err(BadInput("a face's number past the table").into());
        }
        let face = crate::scene::objects::face_cell(number as usize).get() as usize;
        if face == 0 {
            return Err(BadInput("a face that is not there").into());
        }
        object.see_through.set(listed[3]);
        work.face_number.set(listed[0]);
        let head: [u8; 0xC] = crate::scene::object_faces::array(face as u32)?;
        let texture = u16::from_le_bytes([head[4], head[5]]) as u32;
        poly.polygon_textures[0].set(texture.wrapping_add(object.texture_set.get()) | 0x8000_0000);
        work.face_kind.set(head[6] as u32);
        work.face_unlit.set(head[0xB] as u32);
        let shade = head[0xA] as u32;
        poly.polygon_colour.set(if shade == 0 {
            0x00FF_FFFF
        } else {
            0x0001_0101 * shade
        });
        draw(face)?;
        poly.polygon_colour.set(0x00FF_FFFFu32);
    }
    poly.window.faces_done.set(0u32);
    Ok(())
}
