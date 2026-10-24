//! `scene::face_sort_key` (0x450564): the depth a face is sorted by.
//!
//! In:  ebp = the face, ecx = the number of its vertices
//! Out: `dword_62407C` = the key; eax = the last point's record as an offset, ebx = the
//!      number of vertices as `g_polygonVertexCount` has it, ecx = 0, edx = the last
//!      point's record, esi = the face's end; everything else preserved
//!
//! The depths of the face's points (each point's record 0x1000 + (number & 0xFFF), its +0x18)
//! are added one by one, each sum rounded to a float; the sum times the entry
//! `g_polygonVertexCount` of the table `flt_5C6950` (0x100 floats the game fills: one over the
//! number, by the use) is rounded to a whole number (`fistp`). One caller,
//! `scene::list_object_face`. Refuses a count of 0 (the original goes through 2^32), a vertex
//! count past the table, no face and no records.

use crate::d3d::draw_fan_unclipped::COUNT_RVA;
use crate::errors::{BadInput, Refused};
use crate::framework::{game_read, game_write, rva};
use crate::math::x87;
use crate::scene::polygon::dword;
use crate::scene::project_point::RECORDS_RVA;

// ---- game data ----

pub const KEY_RVA: usize = rva(0x62407C); // dword_62407C
const FACTORS_RVA: usize = rva(0x5C6950); // flt_5C6950
const FACTORS: u32 = 0x100;

// ---- Rust version ----

pub unsafe fn face_sort_key(face: usize, count: u32) -> Result<(), Refused> {
    unsafe {
        let records: u32 = game_read(RECORDS_RVA);
        let vertices: u32 = game_read(COUNT_RVA);
        if face == 0 || records == 0 || count == 0 || count > 0x1000 || vertices >= FACTORS {
            return Err(
                BadInput("no face, no records, or a count the key cannot be made for").into(),
            );
        }
        let mut sum = 0f32.to_bits();
        for n in 0..count as usize {
            let offset = ((dword(face + 0x10 + n * 0x10) & 0xFFF) + 0x1000) << 5;
            let record = records.wrapping_add(offset);
            sum = x87::add(dword(record as usize + 0x18), sum);
        }
        let factor: u32 = game_read(FACTORS_RVA + vertices as usize * 4);
        game_write(KEY_RVA, x87::mul_to_int(sum, factor));
        Ok(())
    }
}
