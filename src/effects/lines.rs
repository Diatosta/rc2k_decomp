//! `effects::init_marks`' step 14, its measuring: the lengths of the lines between two
//! objects' faces (each face's points against the other object's same face's), summed into
//! the eight settings blocks (0x4C8 apart) by the face's kind, and then made shares of what
//! they were.

use crate::effects::paths_store::Paths;
use crate::errors::{BadInput, Refused};
use crate::math::x87;
use crate::memory::owned::Owned;
use crate::memory::session::PlainDword;
use crate::race::effects_settings_store::{CarEffects, EffectsSettings};
use crate::race::owner_course::Course;
use crate::scene::object_slots_store::SlotTable;
use crate::scene::objects::FACES;
use crate::scene::owner::Scene;
use rc2k_core::effects::lines as core;

/// A settings block is a car record's part (`core::BLOCKS_AT` from the settings).
const _: () = assert!(crate::race::effects_settings_store::CAR_BLOCK + 0x60 == core::BLOCKS_AT);

/// The lines of two faces (`from`, `to`): for each of `from`'s points (the point numbers in the
/// 0x10-byte entries after the face's header) the length to `to`'s, added to the whole number at
/// `sum`.
fn lines(course: &Course, from: u32, to: u32, sum: &PlainDword) -> Result<(), Refused> {
    lines_with(course, from, to, sum, x87_length)?;
    Ok(())
}

/// The x87's `length_plus_int`.
fn x87_length(d: [i32; 3], sum: i32) -> i32 {
    x87::length_plus_int(d[0], d[1], d[2], sum)
}

/// `lines` with the length worked out by `length`.
fn lines_with(
    course: &Course,
    from: u32,
    to: u32,
    sum: &PlainDword,
    length: fn([i32; 3], i32) -> i32,
) -> Result<(), Refused> {
    {
        let count = course.stage.object_faces.word(from)?;
        let points: u32 = course.lists.borrow().object_points;
        let mut n = 0u32;
        while n != count {
            let (a, b) = (
                course.stage.object_faces.word(from + 0x10 + 0x10 * n)?,
                course.stage.object_faces.word(to + 0x10 + 0x10 * n)?,
            );
            let p = points.wrapping_add(a.wrapping_mul(12));
            let q = points.wrapping_add(b.wrapping_mul(12));
            let d = core::difference(
                course.stage.object_points.point(p)?,
                course.stage.object_points.point(q)?,
            );
            sum.set(length(d.map(|v| v as i32), sum.get() as i32) as u32);
            n += 1;
        }
    }
    Ok(())
}

/// `effects::measure_slot_lines`: faces `from` and `to`, block `block`: for each
/// of the block's 8 slots whose kind (the low byte of `slot_kinds`) is the face's slot byte, its
/// lines into the slot's sum (`slot_sums`). The x87's, for `measure_lines`.
fn measure_slot_lines(
    course: &Course,
    from: u32,
    to: u32,
    block: &CarEffects,
) -> Result<(), Refused> {
    {
        for k in 0..core::SLOTS {
            if core::slot_takes(
                block.slot_kinds[k as usize].get(),
                course.stage.object_faces.byte(from + 7)?,
            ) {
                lines_with(course, from, to, &block.slot_sums[k as usize], x87_length)?;
            }
        }
    }
    Ok(())
}

/// `effects::measure_kind_lines`: faces `from` and `to`, block `block`: for each of the block's
/// 0x11 kinds (the low byte of `kinds`) that is the face's kind byte, with the face's line kind 1:
/// its point count added to `kind_counts`, its lines into `kind_sums`. The x87's, for
/// `measure_lines`.
fn measure_kind_lines(
    course: &Course,
    from: u32,
    to: u32,
    block: &CarEffects,
) -> Result<(), Refused> {
    {
        for k in 0..core::KINDS {
            if core::kind_takes(
                block.kinds[k as usize].get(),
                course.stage.object_faces.byte(from + 6)?,
                course.stage.object_faces.byte(from + 0xB)?,
            ) {
                let count = &block.kind_counts[k as usize];
                count.set(
                    count
                        .get()
                        .wrapping_add(course.stage.object_faces.word(from)?),
                );
                lines_with(course, from, to, &block.kind_sums[k as usize], x87_length)?;
            }
        }
    }
    Ok(())
}

/// The game's tables for `core::measured`: the settings' blocks and the objects' faces.
struct GameBlocks<'a>(&'static EffectsSettings, crate::effects::Sticky, &'a Scene);

impl GameBlocks<'_> {
    /// Block `k`; a block past the eighth is refused.
    fn block(&self, k: u32) -> Result<&CarEffects, BadInput> {
        self.0
            .cars
            .get(k as usize)
            .ok_or(BadInput("a settings block past the eighth"))
    }
}

impl core::Blocks for GameBlocks<'_> {
    fn marked(&self, k: u32) -> bool {
        self.1.take(self.block(k).map(|b| b.morph_kept.get())) != 0
    }
    fn objects(&self, k: u32) -> [u32; 2] {
        [
            self.1.take(self.block(k).map(|b| b.object.get())),
            self.1.take(self.block(k).map(|b| b.second_object.get())),
        ]
    }
    fn face_count(&self, object: u32) -> u32 {
        self.2
            .objects
            .slots
            .borrow()
            .slot_value(SlotTable::FaceCounts, object as usize)
    }
    fn first_face(&self, object: u32) -> u32 {
        self.2
            .objects
            .slots
            .borrow()
            .slot_value(SlotTable::FirstFace, object as usize)
    }
}

/// The blocks `measure_lines` would measure (`core::measured`): (block's number, its first
/// faces, its face count).
fn measured(
    scene: &Scene,
    effects: &crate::effects::owner::Effects,
    every: bool,
) -> Result<Vec<(u32, u32, u32, u32)>, Refused> {
    let marks = effects.settings()?;
    let count = marks.cars_copy.get();
    let game = GameBlocks(marks, Default::default(), scene);
    let blocks = core::measured(count, every, &game, FACES);
    game.1.done()?;
    let blocks = blocks?;
    Ok(blocks
        .into_iter()
        .map(|m| (m.block, m.first[0], m.first[1], m.faces))
        .collect())
}

/// `effects::measure_lines`: `LinesWork.any` 0; for the blocks (the settings' `cars_copy`, the last
/// first), every one with `LinesWork.every` 1 or else those with `morph_kept` set (then
/// `LinesWork.any` 1): the block in `LinesWork.current`, its sums cleared (`face_sums`,
/// `slot_sums`, `kind_sums`, `kind_counts`); for each face of its first `object` with the
/// `second_object`'s same: a face of kind 0x1C with a line kind 1 .. 6 has its lines summed into
/// the `face_sums` entry of that line kind; then `measure_slot_lines`, `measure_kind_lines`. With
/// `LinesWork.shares` 1 each sum then becomes 100 less its share of 100 of what it was
/// (`face_shares`, `slot_shares`, `kind_shares`), with a `finit` after the first six; a share of 0
/// gives 0x80000000, the integer indefinite of the x87's division by 0. Refuses more than 8 blocks,
/// an object past 0xFF, faces past the course's faces.
// The logic is in `rc2k_core::effects::lines::{measured, block_at}` (the lengths and shares
// stay x87: its `finit` between)
pub fn measure_lines(
    course: &Course,
    faces_table: crate::scene::faces_store::FacesRef,
    paths: &Paths,
    scene: &Scene,
    effects: &crate::effects::owner::Effects,
) -> Result<(), Refused> {
    {
        let marks = effects.settings()?;
        let every = paths.lines_work.borrow().every == 1;
        let blocks = measured(scene, effects, every)?;
        paths.lines_work.borrow_mut().any = 0u32;
        let count = marks.cars_copy.get() as i32;
        let mut todo = blocks.into_iter().peekable();
        for k in (0..count.max(0) as u32).rev() {
            let Some(&(number, first_a, first_b, faces)) = todo.peek() else {
                continue;
            };
            if number != k {
                continue;
            }
            let block = &marks.cars[k as usize];
            todo.next();
            if !every {
                paths.lines_work.borrow_mut().any = 1u32;
            }
            for cell in &block.face_sums {
                cell.set(0);
            }
            for cell in &block.slot_sums {
                cell.set(0);
            }
            for n in 0..0x11 {
                block.kind_sums[n].set(0);
                block.kind_counts[n].set(0);
            }
            for n in 0..(faces as i32).max(0) as u32 {
                let from: u32 = crate::scene::objects::face(faces_table, (first_a + n) as usize);
                let to: u32 = crate::scene::objects::face(faces_table, (first_b + n) as usize);
                if course.stage.object_faces.byte(from + 6)? == 0x1C {
                    let kind = course.stage.object_faces.byte(from + 0xB)? as u32;
                    if (1..=6).contains(&kind) {
                        lines(course, from, to, &block.face_sums[kind as usize - 1])?;
                    }
                }
                measure_slot_lines(course, from, to, block)?;
                measure_kind_lines(course, from, to, block)?;
            }
            if paths.lines_work.borrow().shares == 1 {
                let hundred = effects.setup.floats.hundred;
                let share = |at: &PlainDword, of: &PlainDword| {
                    // A share of nothing (no line of that kind measured) is the x87's
                    // integer indefinite, 0x80000000, which the marks then read as the sum
                    // (no division is made: it would give a NaN).
                    let left = if of.get() == 0 {
                        i32::MIN
                    } else {
                        x87::share_left_to_int(at.get() as i32, of.get() as i32, hundred)
                    };
                    at.set(left as u32);
                };
                for (sum, of) in block.face_sums.iter().zip(&block.face_shares) {
                    share(sum, of);
                }
                x87::finit();
                for n in (0..8).rev() {
                    share(&block.slot_sums[n], &block.slot_shares[n]);
                }
                for n in (0..0x11).rev() {
                    share(&block.kind_sums[n], &block.kind_shares[n]);
                }
            }
        }
    }
    Ok(())
}

/// `effects::measure_starting_sums`: `measure_lines` with every block and no shares; then for each
/// block (the last first) its sums moved to where the shares are made from (`face_sums` to
/// `face_shares`, `slot_sums` to `slot_shares`, `kind_sums` to `kind_shares`) and cleared; then the
/// measure-every flag 0 and the make-shares flag 1. Refuses as `measure_lines` with every block,
/// before writing anything.
// The logic is in `rc2k_core::effects::lines::{measured, block_at}`
fn measure_starting_sums(
    course: &Course,
    faces_table: crate::scene::faces_store::FacesRef,
    paths: &Paths,
    scene: &Scene,
    effects: &crate::effects::owner::Effects,
) -> Result<(), Refused> {
    measured(scene, effects, true)?;
    paths.lines_work.borrow_mut().every = 1u32;
    paths.lines_work.borrow_mut().shares = 0u32;
    measure_lines(course, faces_table, paths, scene, effects)?;
    let marks = effects.settings()?;
    let count = marks.cars_copy.get() as i32;
    for k in (0..count.max(0) as u32).rev() {
        let block = &marks.cars[k as usize];
        for (share, sum) in block.face_shares.iter().zip(&block.face_sums) {
            share.set(sum.get());
        }
        for sum in &block.face_sums {
            sum.set(0);
        }
        for (sums, shares) in [
            (&block.slot_sums[..], &block.slot_shares[..]),
            (&block.kind_sums[..], &block.kind_shares[..]),
        ] {
            for (sum, share) in sums.iter().zip(shares).rev() {
                share.set(sum.get());
                sum.set(0);
            }
        }
    }
    paths.lines_work.borrow_mut().every = 0u32;
    paths.lines_work.borrow_mut().shares = 1u32;
    Ok(())
}

/// The settings blocks' base points (each block's `point_block`): 12 bytes a point of the
/// block's object, counted on the block's `points_list`. A block's `point_block` (and
/// `Paths::morph_taken`)
/// holds its table's number + 1; `effects::tidy_marks` gives them back with the lists.
/// (`Paths::block_points`.)
///
/// The table `point_block` names (its number + 1).
fn block_points(paths: &Paths, handle: u32) -> Result<&Owned<Vec<[u32; 3]>>, BadInput> {
    (handle as usize)
        .checked_sub(1)
        .and_then(|n| paths.block_points.get(n))
        .ok_or(BadInput("a settings block's points past the table"))
}

/// Base point `n` of the table `handle` names (`effects::morph_object_points`).
pub(super) fn block_point(paths: &Paths, handle: u32, n: u32) -> Result<[u32; 3], BadInput> {
    block_points(paths, handle)?
        .with(|t| t.get(n as usize).copied())?
        .ok_or(BadInput("a base point past its table"))
}

/// `effects::copy_block_points`: the points of block `block`'s `object`: the object's
/// point count of 12 bytes from its first point, to the table the block's `point_block` names.
/// Refuses an object past 0xFF.
// The logic is in `rc2k_core::effects::lines::check_object`
fn copy_block_points(
    course: &Course,
    paths: &Paths,
    block: &CarEffects,
    scene: &crate::scene::owner::Scene,
) -> Result<(), Refused> {
    {
        let object = block.object.get();
        core::check_object(object)?;
        let to = block_points(paths, block.point_block.get())?;
        let count = scene.objects.slots.borrow().point_counts[object as usize] as i32;
        let first: u32 = scene.objects.slots.borrow().first_point[object as usize];
        let mut from = first
            .wrapping_mul(12)
            .wrapping_add(course.lists.borrow().object_points);
        for n in 0..count.max(0) as usize {
            let point = course.stage.object_points.point(from)?;
            to.with(|t| t.get_mut(n).map(|p| *p = point))?
                .ok_or(BadInput("a block's points past their table"))?;
            from = from.wrapping_add(12);
        }
    }
    Ok(())
}

/// `effects::set_up_blocks`, step 14: `measure_starting_sums`; then for each block (the last
/// first) `factors` and `kept_factors` -1, a block of 12 bytes a point of its object taken on
/// its `points_list` (`Paths::block_points`, its number + 1 into the taken count and
/// `point_block`), `copy_block_points`, and `factors` the eight floats of
/// `LinesWork.block_start`. Refuses as
/// `measure_starting_sums`, before anything is taken.
// The logic is in `rc2k_core::effects::lines::{measured, block_at}`
pub(super) fn set_up_blocks(
    course: &Course,
    faces_table: crate::scene::faces_store::FacesRef,
    paths: &Paths,
    scene: &crate::scene::owner::Scene,
    effects: &crate::effects::owner::Effects,
) -> Result<(), Refused> {
    measured(scene, effects, true)?;
    measure_starting_sums(course, faces_table, paths, scene, effects)?;
    let marks = effects.settings()?;
    let count = marks.cars_copy.get() as i32;
    for k in (0..count.max(0) as u32).rev() {
        let block = &marks.cars[k as usize];
        for cell in block.factors.iter().chain(&block.kept_factors) {
            // All ones: a NaN fills the factors.
            cell.set_bits(u32::MAX);
        }
        let points: u32 = scene
            .objects
            .slots
            .borrow()
            .slot_value(SlotTable::PointCounts, block.object.get() as usize);
        let bytes = points
            .checked_mul(12)
            .ok_or(BadInput("a block's point count that wraps"))?;
        let table = paths
            .block_points
            .get(k as usize)
            .ok_or(BadInput("more than 8 settings blocks"))?;
        table.take(&block.points_list, bytes, vec![[0; 3]; points as usize])?;
        let taken = k + 1;
        paths.morph_taken.set(taken);
        block.point_block.set(taken);
        copy_block_points(course, paths, block, scene)?;
        for (factor, start) in block
            .factors
            .iter()
            .zip(crate::effects::lines_store::BLOCK_START)
        {
            factor.set(x87::loaded(start));
        }
    }
    Ok(())
}

#[cfg(test)]
mod lines_work_tests {
    use super::*;

    #[test]
    fn a_blocks_points_are_named_by_number() {
        let _lock = crate::test_support::test_lock();
        let paths = &crate::test_support::own::effects().paths;
        let figures = crate::memory::figures();
        let memory = crate::memory::PlainList::empty();
        let list = &memory;
        paths.block_points[2]
            .take(list, 24, vec![[1, 2, 3], [4, 5, 6]])
            .unwrap();
        assert_eq!(memory.get().total, 24);
        assert_eq!(block_point(paths, 3, 1).unwrap(), [4, 5, 6]);
        assert!(block_point(paths, 3, 2).is_err());
        assert!(block_point(paths, 0, 0).is_err());
        assert!(block_point(paths, 9, 0).is_err());
        assert!(block_point(paths, 1, 0).is_err(), "a table not taken");
        paths.block_points[2].give_back();
        crate::memory::with_figures(|f| *f = figures);
    }
}
