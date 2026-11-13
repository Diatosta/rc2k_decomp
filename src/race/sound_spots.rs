//! The sound spots of the course around the view (`race::find_sound_spots` ->
//! `race::cell_sound_spots` ->
//! `race::face_sound_spot`): course faces of kind 0xFF with object 0x3F
//! (sound objects) in the 5 x 5 cells of `g_courseGrid` around the view, at most 0x24, into
//! `g_soundSpots` (0x14 bytes: +0 the point, +4 .. +0xC its place turned by the view and
//! scaled by `one_over_65536`, +0x10 the sound, the face's +8, 1 .. 0x15).

use crate::errors::{BadInput, Refused};
use crate::race::owner::Race;
use crate::race::owner_course::Course;
use crate::race::view_grid::ViewGrid;
use crate::scene::view_matrices::ViewMatrices;

/// The grid and what lies past it that the original may read (as `race::course_grid`).
const GRID_DWORDS: u32 = 0x1084 / 4;

/// Field `offset` of course record `record`.
fn record_field(course: &Course, record: u32, offset: u32) -> Result<u32, BadInput> {
    crate::race::course_places::record_field(course.reads(), record, offset)
}

/// The 25 cells' numbers, in the original's order (five across, then on by 0x1C after
/// the fifth: rows 0x20 apart, from reach - 2 on the diagonal).
fn cells(course: &Course) -> Vec<u32> {
    // Converted: `rc2k_core::race::view::cells_from`.
    rc2k_core::race::view::cells_from(course.stage_prefs.get().grid_reach.get().wrapping_sub(2), 5)
}

/// What the original would read through a null or past the grid: refused before anything.
fn check(course: &Course, grid: &ViewGrid) -> Result<(), Refused> {
    for c in cells(course) {
        if c >= GRID_DWORDS {
            return Err(BadInput("a sound spot cell past the course grid").into());
        }
        // The grid is 32 x 32 cells (1024 dwords); the check above lets 1057 through, and
        // the original would read on into the variables after the grid. The last cell is
        // (reach - 2) x 0x21 + 0x84, past the grid only for a reach of 30 or more; the
        // reach is the stage's `localdat` value for the draw distance's level (4 .. 12 in
        // the files the game ships, 0 made 1 for the other levels): inside the grid.
        let record: u32 = grid.cell(c)?.get();
        if record >= 0x0FFF_FFFF {
            continue;
        }
        if course.course.get().course_records.get() == 0
            || course.course.get().course_faces.get() == 0
            || course.course.get().scene_points.get() == 0
        {
            return Err(BadInput("no course records, faces or points").into());
        }
    }
    Ok(())
}

/// `fld tbyte; fmul dword; fstp dword`.
fn scaled(t: [u8; 10], k: impl crate::math::x87_float::Loadable) -> u32 {
    use crate::math::x87_float::{from_f80, ld, st};
    st(from_f80(t) * ld(k))
}

/// `race::face_sound_spot` (0x45C6ED): face `face_number` as a sound spot when it is kind 0xFF,
/// object 0x3F, its sound (+8) 1 .. 0x15 and there is room (fewer than 0x24): its point and place
/// (point + offset +0x30 .. into `place` ..) turned by `g_viewMatrix` (`view`'s own) and
/// scaled. (Its test for sound 0x1B2 needing lamps can never hold: a byte.)
pub fn face_sound_spot(race: &Race, view: &ViewMatrices, face_number: u32) -> Result<(), BadInput> {
    use crate::race::skidmarks::int_80;
    {
        // Converted: `rc2k_core::race::view::{sound_object, takes_spot, spot_place}`
        // (the turn and scale stay the x87's: the callers compare exactly).
        use rc2k_core::race::view::{MOST_SPOTS, sound_object, spot_place, takes_spot};
        let f = crate::race::course_places::face_number(race.course.reads(), face_number)?;
        let kind = f.kind();
        if kind != 0xFF || !sound_object(kind, f.object()) {
            return Ok(());
        }
        let count: u32 = race.frame.sound_spots().sound_spot_count.get();
        if count == MOST_SPOTS {
            return Ok(());
        }
        let spot = count as usize * 5; // 0x14 bytes each, in dwords
        let sound = f.sound() as u32;
        if !takes_spot(count, sound) {
            return Ok(());
        }
        // The count only steps by one from 0 and stops at 0x24, so the spot (five dwords
        // from count x 5, count at most 0x23) ends inside the 180 dwords.
        let put = |o: u32, v: u32| -> Result<(), BadInput> {
            race.frame
                .sound_spots()
                .sound_spots
                .get(spot + (o / 4) as usize)
                .ok_or(BadInput("a sound spot past the 0x24"))?
                .set(v);
            Ok(())
        };
        put(0x10, sound)?;
        let point = f.point(0);
        put(0, point)?;
        let place = spot_place(
            crate::race::scene_points::point_number(
                race.course.reads(),
                &crate::race::scene_points::no_own(),
                point,
            )?,
            [f.dword_at(0x30), f.dword_at(0x34), f.dword_at(0x38)],
        );
        let t = { crate::scene::rotate_point::rotate_point(&view.view_matrix, place.map(int_80)) };
        let k = race.course.numbers.get().one_over_65536.get();
        for (i, v) in t.into_iter().enumerate() {
            put(4 + 4 * i as u32, scaled(v, k))?;
        }
        race.frame.sound_spots().sound_spot_count.set(count + 1);
    }
    Ok(())
}

/// `race::cell_sound_spots` (0x45C6B8): each face of cell `cell`'s record (`g_courseGrid[c]`, none
/// from 0x0FFFFFFF; the record's +8 .. +0xC, none for -1 or a first past the last) by
/// `face_sound_spot`.
pub fn cell_sound_spots(
    race: &Race,
    view: &ViewMatrices,
    grid: &ViewGrid,
    cell: u32,
) -> Result<(), BadInput> {
    {
        // Converted: `rc2k_core::race::view::record_faces`.
        // The cells `check` passed (inside the grid).
        let record: u32 = grid.cell(cell)?.get();
        if record >= rc2k_core::race::view::NO_RECORD {
            return Ok(());
        }
        let mut face = record_field(&race.course, record, 8)?;
        if rc2k_core::race::view::record_faces(face, record_field(&race.course, record, 0xC)?)
            .is_none()
        {
            return Ok(());
        }
        loop {
            face_sound_spot(race, view, face)?;
            face = face.wrapping_add(1);
            if face > record_field(&race.course, record, 0xC)? {
                break;
            }
        }
    }
    Ok(())
}

/// `race::find_sound_spots` (0x45C605): `g_soundSpotCount` 0, then the 5 x 5 cells from
/// (`g_gridReach` - 2) x 0x21 (rows 0x20 apart) of `view`'s grid by `cell_sound_spots`, turned by
/// its matrix. Refuses (before anything) a cell past the grid, or no records, faces or points when
/// a cell has a record.
pub fn find_sound_spots(
    race: &Race,
    view: &crate::race::view_scratch::ViewScratch,
) -> Result<(), Refused> {
    let grid = &view.grid;
    {
        check(&race.course, grid)?;
        race.frame.sound_spots().sound_spot_count.set(0u32);
        for c in cells(&race.course) {
            cell_sound_spots(race, &view.matrices, grid, c)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod reference_vectors {
    use super::*;
    use crate::math::x87_float::{X, f80};
    use crate::race::asm_test::{Rng, at_each_control_word, lock, same};
    use crate::test_support::vectors::{Recorder, word};

    #[test]
    fn scaled_matches() {
        let _l = lock();
        let mut r = Rng(2);
        let mut file = Recorder::replay("race.sound_spots.scaled");
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
                file.input(&t);
                file.input(&k.to_le_bytes());
                let want = word(&mut file);
                let got = scaled(t, k);
                assert!(same(want, got), "{v} {k:#x}: {want:#x} {got:#x}");
            }
        });
        file.finish();
    }
}
