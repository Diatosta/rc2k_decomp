//! `effects::init_marks`' step 14, its measuring (IDA): the lengths of the
//! lines between two objects' faces (each face's points against the other object's same
//! face's), summed into the eight settings blocks (at `g_marksSettings` + 0x29C, 0x4C8
//! apart) by the face's kind, and then made shares of what they were.

#[cfg(test)]
use crate::effects::owner::effects;
use crate::effects::paths::Paths;
use crate::errors::{BadInput, Refused};
use crate::math::x87;
use crate::memory::owned::Owned;
use crate::race::effects_settings_store::{CarEffects, EffectsSettings};
use crate::race::owner_course::Course;
use crate::scene::object_slots_store::ObjectTables;
use crate::scene::objects::FACES;
use crate::session::{Dword, PlainBytes, PlainDword, PlainFloat};
use rc2k_core::effects::lines as core;

/// A settings block is a car record's part (`core::BLOCKS_AT` from the settings).
const _: () = assert!(crate::race::effects_settings_store::CAR_BLOCK + 0x60 == core::BLOCKS_AT);

/// The lines of two faces (`from`, `to`): for each of `from`'s points (+0x10, 0x10 apart)
/// the length to `to`'s, added to the whole number at `sum`.
fn lines(course: &Course, paths: &Paths, from: u32, to: u32, sum: &Dword) -> Result<(), Refused> {
    lines_with(course, paths, from, to, sum, x87_length)?;
    Ok(())
}

/// The x87's `length_plus_int`.
fn x87_length(d: [i32; 3], sum: i32) -> i32 {
    x87::length_plus_int(d[0], d[1], d[2], sum)
}

/// `lines` with the length worked out by `length`.
fn lines_with(
    course: &Course,
    paths: &Paths,
    from: u32,
    to: u32,
    sum: &Dword,
    length: fn([i32; 3], i32) -> i32,
) -> Result<(), Refused> {
    {
        let count = crate::scene::object_faces::word(from)?;
        let points: u32 = course.lists.object_points.get();
        let mut n = 0u32;
        while n != count {
            let (a, b) = (
                crate::scene::object_faces::word(from + 0x10 + 0x10 * n)?,
                crate::scene::object_faces::word(to + 0x10 + 0x10 * n)?,
            );
            let p = points.wrapping_add(a.wrapping_mul(12));
            let q = points.wrapping_add(b.wrapping_mul(12));
            let d = core::difference(
                crate::scene::object_points::point(p)?,
                crate::scene::object_points::point(q)?,
            );
            for (cell, v) in paths.lines_work.difference.iter().zip(d) {
                cell.set(v);
            }
            sum.set(length(d.map(|v| v as i32), sum.get() as i32) as u32);
            n += 1;
        }
    }
    Ok(())
}

/// `effects::measure_slot_lines` (0x4D1167): faces `from` and `to`, block `block`: for each
/// of the block's 8 slots whose kind (the low byte of +0xD0 ..) is the face's +7, its
/// lines into the slot's sum (+0xF0 ..). The x87's, for `measure_lines`.
pub fn measure_slot_lines(
    course: &Course,
    paths: &Paths,
    from: u32,
    to: u32,
    block: &CarEffects,
) -> Result<(), Refused> {
    {
        for k in 0..core::SLOTS {
            if core::slot_takes(
                block.slot_kinds[k as usize].get(),
                crate::scene::object_faces::byte(from + 7)?,
            ) {
                lines_with(
                    course,
                    paths,
                    from,
                    to,
                    &block.slot_sums[k as usize],
                    x87_length,
                )?;
            }
        }
    }
    Ok(())
}

/// `effects::measure_kind_lines` (0x4D1225): faces `from` and `to`, block `block`: for each
/// of the block's 0x11 kinds (the low byte of +0x130 ..) that is the face's +6, with the
/// face's +0xB 1: its point count added to +0x240 .., its lines into +0x1B8 ... The x87's,
/// for `measure_lines`.
pub fn measure_kind_lines(
    course: &Course,
    paths: &Paths,
    from: u32,
    to: u32,
    block: &CarEffects,
) -> Result<(), Refused> {
    {
        for k in 0..core::KINDS {
            if core::kind_takes(
                block.kinds[k as usize].get(),
                crate::scene::object_faces::byte(from + 6)?,
                crate::scene::object_faces::byte(from + 0xB)?,
            ) {
                let count = &block.kind_counts[k as usize];
                count.set(
                    count
                        .get()
                        .wrapping_add(crate::scene::object_faces::word(from)?),
                );
                lines_with(
                    course,
                    paths,
                    from,
                    to,
                    &block.kind_sums[k as usize],
                    x87_length,
                )?;
            }
        }
    }
    Ok(())
}

/// The game's tables for `core::measured`: the settings' blocks and the objects' faces.
struct GameBlocks(&'static EffectsSettings, crate::effects::Sticky);

impl GameBlocks {
    /// Block `k`; a block past the eighth is refused.
    fn block(&self, k: u32) -> Result<&CarEffects, BadInput> {
        self.0
            .cars
            .get(k as usize)
            .ok_or(BadInput("a settings block past the eighth"))
    }
}

impl core::Blocks for GameBlocks {
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
        ObjectTables::slot_value(
            &crate::scene::owner::scene().objects.slots.face_counts,
            object as usize,
        )
    }
    fn first_face(&self, object: u32) -> u32 {
        ObjectTables::slot_value(
            &crate::scene::owner::scene().objects.slots.first_face,
            object as usize,
        )
    }
}

/// The blocks `measure_lines` would measure (`core::measured`): (block's number, its first
/// faces, its face count).
fn measured(
    effects: &crate::effects::owner::Effects,
    every: bool,
) -> Result<Vec<(u32, u32, u32, u32)>, Refused> {
    let marks = effects.settings()?;
    let count = marks.cars_copy.get();
    let game = GameBlocks(marks, Default::default());
    let blocks = core::measured(count, every, &game, FACES);
    game.1.done()?;
    let blocks = blocks?;
    Ok(blocks
        .into_iter()
        .map(|m| (m.block, m.first[0], m.first[1], m.faces))
        .collect())
}

/// `effects::measure_lines` (0x4D0EB4): `LinesWork.any` 0; for the blocks (+0x5C of the
/// settings, the last first), every one with `LinesWork.every` 1 or else those with +0xC set
/// (then `LinesWork.any` 1): the block in `LinesWork.current`, its sums cleared (+0x1C .. +0x30,
/// +0xF0 .., +0x1B8 .., +0x240 ..); for each face of its first object (+0) with the other's
/// (+4) same: a face of kind 0x1C (+6) with a line kind (+0xB) 1 .. 6 has its lines summed
/// into +0x18 + 4 kind; then `measure_slot_lines`, `measure_kind_lines`. With
/// `LinesWork.shares` 1 each sum then becomes 100 less its share of 100 of what it was
/// (+0x34 .., +0x110 .., +0x1FC ..), with a `finit` after the first six (as the original).
/// Refuses more than 8 blocks, an object past 0xFF, faces past `g_faces`.
// Converted: `rc2k_core::effects::lines::{measured, block_at}` (the lengths and shares
// stay x87: its `finit` between)
pub fn measure_lines(
    course: &Course,
    paths: &Paths,
    effects: &crate::effects::owner::Effects,
) -> Result<(), Refused> {
    {
        let marks = effects.settings()?;
        let every = paths.lines_work.every.get() == 1;
        let blocks = measured(effects, every)?;
        paths.lines_work.any.set(0u32);
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
                paths.lines_work.any.set(1u32);
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
                let from: u32 = crate::scene::objects::face_cell((first_a + n) as usize).get();
                let to: u32 = crate::scene::objects::face_cell((first_b + n) as usize).get();
                if crate::scene::object_faces::byte(from + 6)? == 0x1C {
                    let kind = crate::scene::object_faces::byte(from + 0xB)? as u32;
                    if (1..=6).contains(&kind) {
                        lines(course, paths, from, to, &block.face_sums[kind as usize - 1])?;
                    }
                }
                measure_slot_lines(course, paths, from, to, block)?;
                measure_kind_lines(course, paths, from, to, block)?;
            }
            if paths.lines_work.shares.get() == 1 {
                let hundred = effects.setup.floats.hundred.get();
                let share = |at: &Dword, of: &Dword| {
                    at.set(
                        x87::share_left_to_int(at.get() as i32, of.get() as i32, hundred) as u32,
                    );
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

/// `effects::measure_starting_sums` (0x4D0DB5): `measure_lines` with every block and no
/// shares; then for each block (the last first) its sums moved to where the shares are made
/// from (+0x1C .. +0x30 to +0x34 .., +0xF0 .. to +0x110 .., +0x1B8 .. to +0x1FC ..) and
/// cleared; then `g_marksMeasureEvery` 0 and `g_marksMakeShares` 1. Refuses as
/// `measure_lines` with every block, before writing anything.
// Converted: `rc2k_core::effects::lines::{measured, block_at}`
pub fn measure_starting_sums(
    course: &Course,
    paths: &Paths,
    effects: &crate::effects::owner::Effects,
) -> Result<(), Refused> {
    measured(effects, true)?;
    paths.lines_work.every.set(1u32);
    paths.lines_work.shares.set(0u32);
    measure_lines(course, paths, effects)?;
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
    paths.lines_work.every.set(0u32);
    paths.lines_work.shares.set(1u32);
    Ok(())
}

/// The settings blocks' base points (each block's +0x8C; `g_marksTaken` the last taken):
/// 12 bytes a point of the block's object, blocks counted on the block's +0x4C list as the original's were. A block's
/// +0x8C (and `MORPH_WORK.taken`) holds its table's number + 1; `effects::tidy_marks` gives
/// them back with the lists.
/// (`Paths::block_points`.)
///
/// The table `+0x8C` names (its number + 1).
fn block_points(paths: &Paths, handle: u32) -> Result<&Owned<Vec<[u32; 3]>>, BadInput> {
    (handle as usize)
        .checked_sub(1)
        .and_then(|n| paths.block_points.get(n))
        .ok_or(BadInput("a settings block's points past the table"))
}

/// Base point `n` of the table `handle` names (`effects::morph_object_points`).
pub fn block_point(paths: &Paths, handle: u32, n: u32) -> Result<[u32; 3], BadInput> {
    block_points(paths, handle)?
        .with(|t| t.get(n as usize).copied())?
        .ok_or(BadInput("a base point past its table"))
}

/// `effects::copy_block_points` (0x4D0A82): the points of block `block`'s object (+0):
/// `g_objectPointCount2` of 12 bytes from `g_objectFirstPoint` in `g_objectPoints`, to the
/// table the block's +0x8C names. Refuses an object past 0xFF.
// Converted: `rc2k_core::effects::lines::check_object`
pub fn copy_block_points(
    course: &Course,
    paths: &Paths,
    block: &CarEffects,
    scene: &crate::scene::owner::Scene,
) -> Result<(), Refused> {
    {
        let object = block.object.get();
        core::check_object(object)?;
        let to = block_points(paths, block.point_block.get())?;
        let count = scene.objects.slots.point_counts[object as usize].get() as i32;
        let first: u32 = scene.objects.slots.first_point[object as usize].get();
        let mut from = first
            .wrapping_mul(12)
            .wrapping_add(course.lists.object_points.bits());
        for n in 0..count.max(0) as usize {
            let point = crate::scene::object_points::point(from)?;
            to.with(|t| t.get_mut(n).map(|p| *p = point))?
                .ok_or(BadInput("a block's points past their table"))?;
            from = from.wrapping_add(12);
        }
    }
    Ok(())
}

/// `effects::set_up_blocks` (0x4D05BD), step 14: `measure_starting_sums`; then for each
/// block (the last first) +0x90 .. +0xCC -1, a block of 12 bytes a point of its object
/// taken on its +0x4C (`BLOCK_POINTS`, its number + 1 into `g_marksTaken` and +0x8C),
/// `copy_block_points`, and +0x90 .. +0xAC the eight floats of `g_marksBlockStart`. Refuses
/// as `measure_starting_sums`, before anything is taken.
// Converted: `rc2k_core::effects::lines::{measured, block_at}`
pub fn set_up_blocks(
    course: &Course,
    paths: &Paths,
    scene: &crate::scene::owner::Scene,
    effects: &crate::effects::owner::Effects,
) -> Result<(), Refused> {
    measured(effects, true)?;
    measure_starting_sums(course, paths, effects)?;
    let marks = effects.settings()?;
    let count = marks.cars_copy.get() as i32;
    for k in (0..count.max(0) as u32).rev() {
        let block = &marks.cars[k as usize];
        for cell in block.factors.iter().chain(&block.kept_factors) {
            cell.set_bits(u32::MAX);
        }
        let points: u32 = ObjectTables::slot_value(
            &scene.objects.slots.point_counts,
            block.object.get() as usize,
        );
        let bytes = points
            .checked_mul(12)
            .ok_or(BadInput("a block's point count that wraps"))?;
        let table = paths
            .block_points
            .get(k as usize)
            .ok_or(BadInput("more than 8 settings blocks"))?;
        table.take(&block.points_list, bytes, vec![[0; 3]; points as usize])?;
        let taken = k + 1;
        paths.morph_work.taken.set(taken);
        block.point_block.set(taken);
        copy_block_points(course, paths, block, scene)?;
        for n in 0..8 {
            let v = x87::loaded(&paths.lines_work.block_start[n]);
            block.factors[n].set(v);
        }
    }
    Ok(())
}

/// The block lines' work (`lines`) (IDA 0x93C738 .. 0x93C8C0).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct LinesWork {
    /// 0x93C738: `g_marksBlockStart`: the eight values a block's +0x90 .. starts with
    pub block_start: [PlainFloat; 8],
    /// 0x93C758: `flt_93C73C` .. `flt_93C794`, `dword_93C7F0` .. `flt_93C810`: read only by code in no function (no ported code reads them)
    pub _gap_93c758: PlainBytes<280>,
    /// 0x93C870: `dword_93C870` .. `dword_93C878`: the last line's x, y, z differences
    pub difference: [PlainDword; 3],
    /// 0x93C87C: Not named: 44 bytes (a gap, or unreferenced data)
    pub _gap_93c87c: PlainBytes<44>,
    /// 0x93C8A8: `g_marksMeasuredBlock`: the block being measured. Not used: nothing reads it (the original kept the block's address)
    pub current: PlainDword,
    /// 0x93C8AC: Not named: 8 bytes (a gap, or unreferenced data)
    pub _gap_93c8ac: PlainBytes<8>,
    /// 0x93C8B4: `g_marksMeasureEvery`: measure every block
    pub every: PlainDword,
    /// 0x93C8B8: `g_marksMakeShares`: make the shares
    pub shares: PlainDword,
    /// 0x93C8BC: `g_marksAnyMeasured`: a marked block was measured
    pub any: PlainDword,
}

impl LinesWork {
    /// The values the game starts with (the image's bytes).
    pub const fn new() -> Self {
        LinesWork {
            block_start: [
                PlainFloat::new(1.0),
                PlainFloat::new(1.0),
                PlainFloat::new(1.0),
                PlainFloat::new(1.0),
                PlainFloat::new(0.0),
                PlainFloat::new(0.0),
                PlainFloat::new(0.0),
                PlainFloat::new(0.0),
            ],
            _gap_93c758: PlainBytes::new([
                0x0A, 0xD7, 0x23, 0xBC, 0x0A, 0xD7, 0x23, 0xBC, 0x0A, 0xD7, 0x23, 0xBC, 0x0A, 0xD7,
                0x23, 0xBC, 0x0A, 0xD7, 0x23, 0x3C, 0x0A, 0xD7, 0x23, 0x3C, 0x0A, 0xD7, 0x23, 0x3C,
                0x0A, 0xD7, 0x23, 0x3C, 0x0A, 0xD7, 0x23, 0x3C, 0x0A, 0xD7, 0x23, 0x3C, 0x0A, 0xD7,
                0x23, 0x3C, 0x0A, 0xD7, 0x23, 0x3C, 0x0A, 0xD7, 0x23, 0xBC, 0x0A, 0xD7, 0x23, 0xBC,
                0x0A, 0xD7, 0x23, 0xBC, 0x0A, 0xD7, 0x23, 0xBC, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x80, 0x47, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x43, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x80, 0x42, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            ]),
            difference: [
                PlainDword::new(0x0),
                PlainDword::new(0x0),
                PlainDword::new(0x0),
            ],
            _gap_93c87c: PlainBytes::new([0; 44]),
            current: PlainDword::new(0x0),
            _gap_93c8ac: PlainBytes::new([0; 8]),
            every: PlainDword::new(0x0),
            shares: PlainDword::new(0x0),
            any: PlainDword::new(0x0),
        }
    }
    pub const START: usize = 0x93C738;
    pub const LEN: usize = 0x188;
}

#[cfg(test)]
mod lines_work_tests {
    use super::*;
    use std::mem::offset_of;

    fn bytes<T>(v: &T) -> Vec<u8> {
        let p = std::ptr::from_ref::<T>(v).cast::<u8>();
        (0..size_of::<T>())
            .map(|i| unsafe { p.add(i).read() })
            .collect()
    }

    #[test]
    fn a_blocks_points_are_named_by_number() {
        let _lock = crate::store::test_lock();
        let paths = &effects().paths;
        let figures = crate::memory::figures();
        let memory = crate::memory::ListCells::empty();
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

    #[test]
    fn lines_work_starts_as_the_games() {
        assert_eq!(size_of::<LinesWork>(), LinesWork::LEN);
        assert!(
            bytes(&crate::effects::owner::effects().paths.lines_work)
                == crate::moved::image_bytes(LinesWork::START, LinesWork::LEN),
            "LINES_WORK starts differently from the image"
        );
        assert_eq!(
            offset_of!(LinesWork, block_start),
            0x93C738 - LinesWork::START
        );
        assert_eq!(
            offset_of!(LinesWork, difference),
            0x93C870 - LinesWork::START
        );
        assert_eq!(offset_of!(LinesWork, current), 0x93C8A8 - LinesWork::START);
        assert_eq!(offset_of!(LinesWork, every), 0x93C8B4 - LinesWork::START);
        assert_eq!(offset_of!(LinesWork, shares), 0x93C8B8 - LinesWork::START);
        assert_eq!(offset_of!(LinesWork, any), 0x93C8BC - LinesWork::START);
    }
}
