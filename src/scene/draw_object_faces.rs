//! `scene::draw_object_faces` (0x450A4B): draws the faces of an object that were listed
//! for drawing.
//!
//! In:  nothing
//! Out: nothing changed with an empty list; else eax, ecx, esi and ebp are the loop's
//!      leftovers (the last face's number in the table, the count, the last list
//!      entry, the last face); everything else preserved
//!
//! The list: `dword_62DB58` entries of 0x10 bytes at `dword_62DB5C`: +0 the face's
//! number within the object, +0xC how see-through it is (to `g_objectSeeThrough`).
//! For each: the number to `dword_623620`; the face is entry number + `dword_6350BC`
//! of the table `dword_9A3730`; from the face's head the first texture (the word +4
//! plus `dword_639954`, with the top bit set), its kind (byte +6, `g_faceKind`) and
//! byte +0xB (`dword_6D18D0`); the polygon's colour is white, or with a byte +0xA
//! not 0 that byte in all three places; `scene::draw_object_face`; the colour white
//! again. At the end `dword_624204` = 0.
//! One caller, `sub_45045D`.
//! Refuses a face's number past the table and a face that is not there.
//! Shadow mode runs the original alone.
//! Self-test: a redirected copy whose call notes the face and what was set for it.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, Refused};
use crate::framework::{EAX, EBP, ECX, ESI, game_addr, game_read, game_write, rva};
use crate::scene::polygon::dword;
use crate::screens::harness::Left;

// ---- game data ----

pub const LISTED_RVA: usize = rva(0x62DB58); // dword_62DB58
pub const LIST_RVA: usize = rva(0x62DB5C); // dword_62DB5C
const LIST_ENTRY_LEN: usize = 0x10;
const FACE_NUMBER_RVA: usize = rva(0x623620); // dword_623620
const FIRST_FACE_RVA: usize = rva(0x6350BC); // dword_6350BC
const FACES_RVA: usize = rva(0x9A3730); // dword_9A3730
const FACES: u32 = 0xC000;
const SEE_THROUGH_RVA: usize = rva(0x639970); // g_objectSeeThrough
const TEXTURE_A_RVA: usize = rva(0x6240BC); // dword_6240BC
const FIRST_TEXTURE_RVA: usize = rva(0x639954); // dword_639954
const FACE_KIND_RVA: usize = rva(0x6D18CC); // g_faceKind
const FACE_UNLIT_RVA: usize = rva(0x6D18D0); // dword_6D18D0
const COLOUR_RVA: usize = rva(0x6240CC); // g_polygonColour
const DONE_RVA: usize = rva(0x624204); // dword_624204

// ---- Rust version ----

pub unsafe fn draw_object_faces(
    draw: &mut dyn FnMut(usize) -> Result<(), Refused>,
) -> Result<Left, Refused> {
    unsafe {
        let listed: u32 = game_read(LISTED_RVA);
        if listed == 0 {
            return Ok(Left::nothing());
        }
        let first: u32 = game_read(FIRST_FACE_RVA);
        // Every face is looked up before anything is written.
        let mut faces = Vec::with_capacity(listed as usize);
        for n in 0..listed as usize {
            let entry = game_addr(LIST_RVA) + n * LIST_ENTRY_LEN;
            let number = dword(entry).wrapping_add(first);
            if number >= FACES {
                return Err(BadInput("a face's number past the table").into());
            }
            let face: u32 = game_read(FACES_RVA + number as usize * 4);
            if face == 0 {
                return Err(BadInput("a face that is not there").into());
            }
            faces.push((entry, number, face as usize));
        }
        for &(entry, _, face) in &faces {
            game_write(SEE_THROUGH_RVA, dword(entry + 0xC));
            game_write(FACE_NUMBER_RVA, dword(entry));
            let texture = ((face + 4) as *const u16).read_unaligned() as u32;
            game_write(
                TEXTURE_A_RVA,
                texture.wrapping_add(game_read(FIRST_TEXTURE_RVA)) | 0x8000_0000,
            );
            game_write(FACE_KIND_RVA, ((face + 6) as *const u8).read() as u32);
            game_write(FACE_UNLIT_RVA, ((face + 0xB) as *const u8).read() as u32);
            let shade = ((face + 0xA) as *const u8).read() as u32;
            game_write(
                COLOUR_RVA,
                if shade == 0 {
                    0x00FF_FFFF
                } else {
                    0x0001_0101 * shade
                },
            );
            draw(face)?;
            game_write(COLOUR_RVA, 0x00FF_FFFFu32);
        }
        game_write(DONE_RVA, 0u32);
        let &(entry, number, face) = faces.last().expect("the list is not empty");
        Ok(Left::nothing()
            .with(EAX, number)
            .with(ECX, listed)
            .with(ESI, entry as u32)
            .with(EBP, face as u32))
    }
}
