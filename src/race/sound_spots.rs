//! The sound spots of the course around the view (`sub_45C605` -> `sub_45C6B8` ->
//! `sub_45C6ED`, part R3, IDA 2026-10-18): course faces of kind 0xFF with object 0x3F
//! (sound objects) in the 5 x 5 cells of `g_courseGrid` around the view, at most 0x24, into
//! `g_soundSpots` (0x14 bytes: +0 the point, +4 .. +0xC its place turned by the view and
//! scaled by `flt_5C5DA0`, +0x10 the sound, the face's +8, 1 .. 0x15).

use crate::errors::{BadInput, Refused};
use crate::framework::{game_addr, game_read, game_write, rva};
use crate::race::course_store::COURSE;
use crate::race::numbers_store::NUMBERS;
use crate::scene::object_store::OBJECT_WORK;

const GRID: usize = rva(0x63E17C); // g_courseGrid
const REACH: usize = rva(0x60A87C); // g_gridReach
const SPOTS: usize = rva(0x6D1AF4); // g_soundSpots
const COUNT: usize = rva(0x6D1DC4); // g_soundSpotCount
/// The grid and what lies past it that the original may read (as `race::course_grid`).
const GRID_DWORDS: u32 = 0x1084 / 4;

unsafe fn get(a: u32) -> u32 {
    unsafe { (a as *const u32).read_unaligned() }
}

/// The 25 cells' numbers, in the original's order (five across, then on by 0x1C after
/// the fifth: rows 0x20 apart, from reach - 2 on the diagonal).
fn cells() -> Vec<u32> {
    // Converted: `rc2k_core::race::view::cells_from`.
    rc2k_core::race::view::cells_from(unsafe { game_read::<u32>(REACH) }.wrapping_sub(2), 5)
}

/// What the original would read through a null or past the grid: refused before anything.
unsafe fn check() -> Result<(), Refused> {
    unsafe {
        for c in cells() {
            if c >= GRID_DWORDS {
                return Err(BadInput("a sound spot cell past the course grid").into());
            }
            let record: u32 = game_read(GRID + 4 * c as usize);
            if record >= 0x0FFF_FFFF {
                continue;
            }
            if COURSE.course_records.bits() == 0
                || COURSE.course_faces.bits() == 0
                || COURSE.scene_points.bits() == 0
            {
                return Err(BadInput("no course records, faces or points").into());
            }
        }
        Ok(())
    }
}

/// `fld tbyte; fmul dword; fstp dword`.
fn scaled(t: [u8; 10], k: u32) -> u32 {
    use crate::math::x87_float::{from_f80, ld, st};
    st(from_f80(t) * ld(k))
}

/// `race::face_sound_spot` (0x45C6ED): face `face_number` (ebx) as a sound spot when it is kind 0xFF,
/// object 0x3F, its sound (+8) 1 .. 0x15 and there is room (fewer than 0x24): its point and
/// place (point + offset +0x30 .. into `dword_63501C` ..) turned by `g_viewMatrix` and
/// scaled. (Its test for sound 0x1B2 needing lamps can never hold: a byte.)
pub unsafe fn face_sound_spot(face_number: u32) {
    use crate::race::skidmarks::int_80;
    unsafe {
        let face = COURSE.course_faces.bits().wrapping_add(face_number << 6);
        // Converted: `rc2k_core::race::view::{sound_object, takes_spot, spot_place}`
        // (the turn and scale stay the x87's: the callers compare exactly).
        use rc2k_core::race::view::{MOST_SPOTS, sound_object, spot_place, takes_spot};
        let byte = |o: u32| ((face + o) as *const u8).read();
        let kind = byte(0);
        if kind != 0xFF || !sound_object(kind, byte(1)) {
            return;
        }
        let count: u32 = game_read(COUNT);
        if count == MOST_SPOTS {
            return;
        }
        let spot = game_addr(SPOTS) as u32 + count * 0x14;
        let sound = byte(8) as u32;
        if !takes_spot(count, sound) {
            return;
        }
        let put = |o: u32, v: u32| ((spot + o) as *mut u32).write_unaligned(v);
        put(0x10, sound);
        let point = get(face + 4);
        put(0, point);
        let p = point
            .wrapping_mul(12)
            .wrapping_add(COURSE.scene_points.bits());
        let place = spot_place(
            [get(p), get(p + 4), get(p + 8)],
            [get(face + 0x30), get(face + 0x34), get(face + 0x38)],
        );
        crate::session::write_bits(&OBJECT_WORK.place, place);
        let t = crate::scene::rotate_point::rotate_point(
            game_addr(rva(0x5EA2CC)) as u32,
            place.map(int_80),
        );
        let k: u32 = NUMBERS.one_over_65536.bits();
        for (i, v) in t.into_iter().enumerate() {
            put(4 + 4 * i as u32, scaled(v, k));
        }
        game_write(COUNT, count + 1);
    }
}

/// `race::cell_sound_spots` (0x45C6B8): each face of cell `cell`'s (ecx) record (`g_courseGrid[c]`,
/// none from 0x0FFFFFFF; the record's +8 .. +0xC, none for -1 or a first past the last) by
/// `face_sound_spot` (inside `pusha` .. `popa`).
pub unsafe fn cell_sound_spots(cell: u32) {
    unsafe {
        // Converted: `rc2k_core::race::view::record_faces`.
        let record: u32 = game_read(GRID + 4 * cell as usize);
        if record >= rc2k_core::race::view::NO_RECORD {
            return;
        }
        let at = COURSE.course_records.bits().wrapping_add(record << 5);
        let mut face = get(at + 8);
        if rc2k_core::race::view::record_faces(face, get(at + 0xC)).is_none() {
            return;
        }
        loop {
            face_sound_spot(face);
            face = face.wrapping_add(1);
            if face > get(at + 0xC) {
                break;
            }
        }
    }
}

/// `race::find_sound_spots` (0x45C605): `g_soundSpotCount` 0, then the 5 x 5 cells from
/// (`g_gridReach` - 2) x 0x21 (rows 0x20 apart) by `cell_sound_spots`. Its one caller,
/// `race::draw_race_scene`, reads no register it leaves. Refuses (before anything) a cell past the grid, or no
/// records, faces or points when a cell has a record.
pub unsafe fn find_sound_spots() -> Result<(), Refused> {
    unsafe {
        check()?;
        game_write(COUNT, 0u32);
        for c in cells() {
            cell_sound_spots(c);
        }
        Ok(())
    }
}

#[cfg(test)]
mod asm_reference {
    use super::*;
    use crate::math::x87_float::{X, f80};
    use crate::race::asm_test::{Rng, at_each_control_word, lock, same};

    /// The original block.
    fn scaled_asm(t: [u8; 10], k: u32) -> u32 {
        let (v, mut out) = ([k], 0u32);
        unsafe {
            core::arch::asm!(
                "fld tbyte ptr [{t}]",
                "fmul dword ptr [{v}]",
                "fstp dword ptr [{o}]",
                t = in(reg) t.as_ptr(),
                v = in(reg) v.as_ptr(),
                o = in(reg) &raw mut out,
                out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
                out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
                options(nostack),
            );
        }
        out
    }

    #[test]
    fn scaled_matches() {
        let _l = lock();
        let mut r = Rng(2);
        at_each_control_word(|| {
            for _ in 0..20000 {
                // the places are whole numbers or floats, held exactly (`int_80`)
                let v = if r.below(2) == 0 {
                    f64::from(r.int())
                } else {
                    f64::from(f32::from_bits(r.float()))
                };
                let t = f80(X(v));
                let k = r.float();
                let (want, got) = (scaled_asm(t, k), scaled(t, k));
                assert!(same(want, got), "{v} {k:#x}: {want:#x} {got:#x}");
            }
        });
    }
}
