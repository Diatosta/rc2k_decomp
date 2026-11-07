//! The sound spots of the course around the view (`sub_45C605` -> `sub_45C6B8` ->
//! `sub_45C6ED`, part R3, IDA 2026-10-18): course faces of kind 0xFF with object 0x3F
//! (sound objects) in the 5 x 5 cells of `g_courseGrid` around the view, at most 0x24, into
//! `g_soundSpots` (0x14 bytes: +0 the point, +4 .. +0xC its place turned by the view and
//! scaled by `flt_5C5DA0`, +0x10 the sound, the face's +8, 1 .. 0x15).

use crate::errors::{BadInput, Refused};
use crate::race::owner::Race;
use crate::race::owner_course::Course;

/// The grid and what lies past it that the original may read (as `race::course_grid`).
const GRID_DWORDS: u32 = 0x1084 / 4;

/// `g_courseGrid`[c]. The grid is 32 x 32 cells (1024 dwords); the check below lets 1057
/// through, and the original reads on into the variables after the grid (a slip kept).
fn grid_cell(course: &Course, c: u32) -> Result<u32, BadInput> {
    crate::session::read_on_in(
        &course.level_grid,
        &course.level_grid.course_grid,
        c as usize,
        "the variables after the course grid",
    )
}

/// Field `offset` of course record `record`.
fn record_field(record: u32, offset: u32) -> Result<u32, BadInput> {
    crate::race::course_places::record_field(record, offset, "a sound spot record")
}

/// The 25 cells' numbers, in the original's order (five across, then on by 0x1C after
/// the fifth: rows 0x20 apart, from reach - 2 on the diagonal).
fn cells(course: &Course) -> Vec<u32> {
    // Converted: `rc2k_core::race::view::cells_from`.
    rc2k_core::race::view::cells_from(course.stage_prefs.grid_reach.bits().wrapping_sub(2), 5)
}

/// What the original would read through a null or past the grid: refused before anything.
fn check(course: &Course) -> Result<(), Refused> {
    for c in cells(course) {
        if c >= GRID_DWORDS {
            return Err(BadInput("a sound spot cell past the course grid").into());
        }
        let record: u32 = grid_cell(course, c)?;
        if record >= 0x0FFF_FFFF {
            continue;
        }
        if course.course.course_records.bits() == 0
            || course.course.course_faces.bits() == 0
            || course.course.scene_points.bits() == 0
        {
            return Err(BadInput("no course records, faces or points").into());
        }
    }
    Ok(())
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
pub fn face_sound_spot(
    race: &Race,
    face_number: u32,
    scene: &crate::scene::owner::Scene,
) -> Result<(), BadInput> {
    use crate::race::skidmarks::int_80;
    {
        // Converted: `rc2k_core::race::view::{sound_object, takes_spot, spot_place}`
        // (the turn and scale stay the x87's: the callers compare exactly).
        use rc2k_core::race::view::{MOST_SPOTS, sound_object, spot_place, takes_spot};
        let f = crate::race::course_places::face_number(face_number, "a sound spot face")?;
        let kind = f.kind();
        if kind != 0xFF || !sound_object(kind, f.object()) {
            return Ok(());
        }
        let count: u32 = race.frame.sound_spots.sound_spot_count.bits();
        if count == MOST_SPOTS {
            return Ok(());
        }
        let spot = count as usize * 5; // 0x14 bytes each, in dwords
        let sound = f.sound() as u32;
        if !takes_spot(count, sound) {
            return Ok(());
        }
        let put = |o: u32, v: u32| {
            crate::session::write_on_in(
                &race.frame.sound_spots,
                &race.frame.sound_spots.sound_spots,
                spot + (o / 4) as usize,
                v,
                "a sound spot",
            )
        };
        put(0x10, sound)?;
        let point = f.point(0);
        put(0, point)?;
        let place = spot_place(
            crate::race::scene_points::point_number(point)?,
            [f.dword_at(0x30), f.dword_at(0x34), f.dword_at(0x38)],
        );
        crate::session::write_bits(&scene.drawing.object_work.place, place);
        let t = {
            crate::scene::rotate_point::rotate_point(
                &race.frame.matrices.view_matrix,
                place.map(int_80),
            )
        };
        let k: u32 = race.course.numbers.one_over_65536.bits();
        for (i, v) in t.into_iter().enumerate() {
            put(4 + 4 * i as u32, scaled(v, k))?;
        }
        race.frame.sound_spots.sound_spot_count.set_bits(count + 1);
    }
    Ok(())
}

/// `race::cell_sound_spots` (0x45C6B8): each face of cell `cell`'s (ecx) record (`g_courseGrid[c]`,
/// none from 0x0FFFFFFF; the record's +8 .. +0xC, none for -1 or a first past the last) by
/// `face_sound_spot`.
pub fn cell_sound_spots(
    race: &Race,
    cell: u32,
    scene: &crate::scene::owner::Scene,
) -> Result<(), BadInput> {
    {
        // Converted: `rc2k_core::race::view::record_faces`.
        let record: u32 = grid_cell(&race.course, cell)?;
        if record >= rc2k_core::race::view::NO_RECORD {
            return Ok(());
        }
        let mut face = record_field(record, 8)?;
        if rc2k_core::race::view::record_faces(face, record_field(record, 0xC)?).is_none() {
            return Ok(());
        }
        loop {
            face_sound_spot(race, face, scene)?;
            face = face.wrapping_add(1);
            if face > record_field(record, 0xC)? {
                break;
            }
        }
    }
    Ok(())
}

/// `race::find_sound_spots` (0x45C605): `g_soundSpotCount` 0, then the 5 x 5 cells from
/// (`g_gridReach` - 2) x 0x21 (rows 0x20 apart) by `cell_sound_spots`. Its one caller,
/// `race::draw_race_scene`, reads no register it leaves. Refuses (before anything) a cell past the grid, or no
/// records, faces or points when a cell has a record.
pub fn find_sound_spots(race: &Race, scene: &crate::scene::owner::Scene) -> Result<(), Refused> {
    {
        check(&race.course)?;
        race.frame.sound_spots.sound_spot_count.set_bits(0u32);
        for c in cells(&race.course) {
            cell_sound_spots(race, c, scene)?;
        }
        Ok(())
    }
}

#[cfg(all(test, target_arch = "x86"))]
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
