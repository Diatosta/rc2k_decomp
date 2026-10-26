//! `marks::init_marks`' step 14, its measuring (IDA, 2026-10-16): the lengths of the
//! lines between two objects' faces (each face's points against the other object's same
//! face's), summed into the eight settings blocks (at `g_marksSettings` + 0x29C, 0x4C8
//! apart) by the face's kind, and then made shares of what they were.

use crate::errors::{BadInput, Refused};
use crate::marks::morph_object_points::MORPH_WORK;
use crate::marks::tuning::MARKS_FLOATS;
use crate::math::x87;
use crate::pointers::GamePtr;
use crate::race::course_lists_store::COURSE_LISTS;
use crate::scene::object_slots_store::{OBJECT_SLOTS, ObjectTables};
use crate::scene::objects::FACES;
use crate::session::{Bytes, Dword, Float};
use rc2k_core::marks::lines as core;

fn get(at: u32) -> Result<u32, BadInput> {
    GamePtr::new(at).read("a game field")
}

fn put(at: u32, v: u32) -> Result<(), BadInput> {
    GamePtr::new(at).write(v, "a game field")
}

fn byte(at: u32) -> Result<u8, BadInput> {
    GamePtr::new(at).read::<u8>("a game field")
}

/// The lines of two faces (`from`, `to`): for each of `from`'s points (+0x10, 0x10 apart)
/// the length to `to`'s, added to the whole number at `sum`.
fn lines(from: u32, to: u32, sum: u32) -> Result<(), Refused> {
    lines_with(from, to, sum, x87_length)?;
    Ok(())
}

/// The x87's `length_plus_int`.
fn x87_length(d: [i32; 3], sum: i32) -> i32 {
    x87::length_plus_int(d[0], d[1], d[2], sum)
}

/// `lines` with the length worked out by `length`.
fn lines_with(
    from: u32,
    to: u32,
    sum: u32,
    length: fn([i32; 3], i32) -> i32,
) -> Result<(), Refused> {
    {
        let count = get(from)?;
        let points: u32 = COURSE_LISTS.object_points.bits();
        let mut n = 0u32;
        while n != count {
            let (a, b) = (get(from + 0x10 + 0x10 * n)?, get(to + 0x10 + 0x10 * n)?);
            let p = points.wrapping_add(a.wrapping_mul(12));
            let q = points.wrapping_add(b.wrapping_mul(12));
            let d = core::difference(
                [get(p)?, get(p + 4)?, get(p + 8)?],
                [get(q)?, get(q + 4)?, get(q + 8)?],
            );
            for (cell, v) in LINES_WORK.difference.iter().zip(d) {
                cell.set(v);
            }
            put(sum, length(d.map(|v| v as i32), get(sum)? as i32) as u32)?;
            n += 1;
        }
    }
    Ok(())
}

/// `marks::measure_slot_lines` (0x4D1167): faces `from` and `to`, block `block`: for each
/// of the block's 8 slots whose kind (the low byte of +0xD0 ..) is the face's +7, its
/// lines into the slot's sum (+0xF0 ..). The x87's, for `measure_lines`.
pub fn measure_slot_lines(from: u32, to: u32, block: u32) -> Result<(), Refused> {
    {
        for k in 0..core::SLOTS {
            if core::slot_takes(get(block + 0xD0 + 4 * k)?, byte(from + 7)?) {
                lines_with(from, to, block + 0xF0 + 4 * k, x87_length)?;
            }
        }
    }
    Ok(())
}

/// `marks::measure_kind_lines` (0x4D1225): faces `from` and `to`, block `block`: for each
/// of the block's 0x11 kinds (the low byte of +0x130 ..) that is the face's +6, with the
/// face's +0xB 1: its point count added to +0x240 .., its lines into +0x1B8 ... The x87's,
/// for `measure_lines`.
pub fn measure_kind_lines(from: u32, to: u32, block: u32) -> Result<(), Refused> {
    {
        for k in 0..core::KINDS {
            if core::kind_takes(
                get(block + 0x130 + 4 * k)?,
                byte(from + 6)?,
                byte(from + 0xB)?,
            ) {
                let count = block + 0x240 + 4 * k;
                put(count, get(count)?.wrapping_add(get(from)?))?;
                lines_with(from, to, block + 0x1B8 + 4 * k, x87_length)?;
            }
        }
    }
    Ok(())
}

/// The game's tables for `core::measured`: the settings' blocks and the objects' faces.
struct GameBlocks(u32, crate::marks::Sticky);

impl core::Blocks for GameBlocks {
    fn marked(&self, k: u32) -> bool {
        self.1.take(get(core::block_at(self.0, k) + 0xC)) != 0
    }
    fn objects(&self, k: u32) -> [u32; 2] {
        let block = core::block_at(self.0, k);
        [self.1.take(get(block)), self.1.take(get(block + 4))]
    }
    fn face_count(&self, object: u32) -> u32 {
        ObjectTables::slot_value(&OBJECT_SLOTS.face_counts, object as usize)
    }
    fn first_face(&self, object: u32) -> u32 {
        ObjectTables::slot_value(&OBJECT_SLOTS.first_face, object as usize)
    }
}

/// The blocks `measure_lines` would measure (`core::measured`): (block, its first faces,
/// its face count).
fn measured(settings: u32, every: bool) -> Result<Vec<(u32, u32, u32, u32)>, Refused> {
    let count = get(settings + 0x5C)?;
    let game = GameBlocks(settings, Default::default());
    let blocks = core::measured(count, every, &game, FACES);
    game.1.done()?;
    let blocks = blocks?;
    Ok(blocks
        .into_iter()
        .map(|m| {
            (
                core::block_at(settings, m.block),
                m.first[0],
                m.first[1],
                m.faces,
            )
        })
        .collect())
}

/// `marks::measure_lines` (0x4D0EB4): `dword_93C8BC` 0; for the blocks (+0x5C of the
/// settings, the last first), every one with `dword_93C8B4` 1 or else those with +0xC set
/// (then `dword_93C8BC` 1): the block in `dword_93C8A8`, its sums cleared (+0x1C .. +0x30,
/// +0xF0 .., +0x1B8 .., +0x240 ..); for each face of its first object (+0) with the other's
/// (+4) same: a face of kind 0x1C (+6) with a line kind (+0xB) 1 .. 6 has its lines summed
/// into +0x18 + 4 kind; then `measure_slot_lines`, `measure_kind_lines`. With
/// `dword_93C8B8` 1 each sum then becomes 100 less its share of 100 of what it was
/// (+0x34 .., +0x110 .., +0x1FC ..), with a `finit` after the first six (as the original).
/// Refuses more than 8 blocks, an object past 0xFF, faces past `g_faces`.
// Converted: `rc2k_core::marks::lines::{measured, block_at}` (the lengths and shares
// stay x87: its `finit` between)
pub fn measure_lines() -> Result<(), Refused> {
    {
        let settings: u32 = crate::marks::setup::SETTINGS.pointer.get();
        let every = LINES_WORK.every.get() == 1;
        let blocks = measured(settings, every)?;
        LINES_WORK.any.set(0u32);
        let count = get(settings + 0x5C)? as i32;
        let mut todo = blocks.into_iter().peekable();
        for k in (0..count.max(0) as u32).rev() {
            let Some(&(block, first_a, first_b, faces)) = todo.peek() else {
                continue;
            };
            if block != block_at(settings, k) {
                continue;
            }
            todo.next();
            if !every {
                LINES_WORK.any.set(1u32);
            }
            LINES_WORK.current.set(block);
            for o in (0x1C..=0x30).step_by(4) {
                put(block + o, 0)?;
            }
            for n in 0..8 {
                put(block + 0xF0 + 4 * n, 0)?;
            }
            for n in 0..0x11 {
                put(block + 0x1B8 + 4 * n, 0)?;
                put(block + 0x240 + 4 * n, 0)?;
            }
            for n in 0..(faces as i32).max(0) as u32 {
                let from: u32 = crate::scene::objects::face_cell((first_a + n) as usize).get();
                let to: u32 = crate::scene::objects::face_cell((first_b + n) as usize).get();
                if byte(from + 6)? == 0x1C {
                    let kind = byte(from + 0xB)? as u32;
                    if (1..=6).contains(&kind) {
                        lines(from, to, block + 0x18 + 4 * kind)?;
                    }
                }
                measure_slot_lines(from, to, block)?;
                measure_kind_lines(from, to, block)?;
            }
            if LINES_WORK.shares.get() == 1 {
                let hundred: u32 = MARKS_FLOATS.hundred.bits();
                let share = |at: u32, of: u32| -> Result<_, Refused> {
                    put(
                        at,
                        x87::share_left_to_int(get(at)? as i32, get(of)? as i32, hundred) as u32,
                    )?;
                    Ok(())
                };
                for o in (0x1C..=0x30).step_by(4) {
                    share(block + o, block + o + 0x18)?;
                }
                x87::finit();
                for n in (0..8).rev() {
                    share(block + 0xF0 + 4 * n, block + 0x110 + 4 * n)?;
                }
                for n in (0..0x11).rev() {
                    share(block + 0x1B8 + 4 * n, block + 0x1FC + 4 * n)?;
                }
            }
        }
    }
    Ok(())
}

fn block_at(settings: u32, k: u32) -> u32 {
    core::block_at(settings, k)
}

/// `marks::measure_starting_sums` (0x4D0DB5): `measure_lines` with every block and no
/// shares; then for each block (the last first) its sums moved to where the shares are made
/// from (+0x1C .. +0x30 to +0x34 .., +0xF0 .. to +0x110 .., +0x1B8 .. to +0x1FC ..) and
/// cleared; then `g_marksMeasureEvery` 0 and `g_marksMakeShares` 1. Refuses as
/// `measure_lines` with every block, before writing anything.
// Converted: `rc2k_core::marks::lines::{measured, block_at}`
pub fn measure_starting_sums() -> Result<(), Refused> {
    let settings: u32 = crate::marks::setup::SETTINGS.pointer.get();
    measured(settings, true)?;
    LINES_WORK.every.set(1u32);
    LINES_WORK.shares.set(0u32);
    measure_lines()?;
    let settings: u32 = crate::marks::setup::SETTINGS.pointer.get();
    let count = get(settings + 0x5C)? as i32;
    for k in (0..count.max(0) as u32).rev() {
        let block = block_at(settings, k);
        for o in (0x1C..=0x30).step_by(4) {
            put(block + o + 0x18, get(block + o)?)?;
        }
        for o in (0x1C..=0x30).step_by(4) {
            put(block + o, 0)?;
        }
        for (from, to, n) in [(0xF0, 0x110, 8u32), (0x1B8, 0x1FC, 0x11)] {
            for k in (0..n).rev() {
                put(block + to + 4 * k, get(block + from + 4 * k)?)?;
                put(block + from + 4 * k, 0)?;
            }
        }
    }
    LINES_WORK.every.set(0u32);
    LINES_WORK.shares.set(1u32);
    Ok(())
}

/// `marks::copy_block_points` (0x4D0A82): the points of block `block`'s object (+0):
/// `g_objectPointCount2` of 12 bytes from `g_objectFirstPoint` in `g_objectPoints`, to the
/// block's +0x8C. Refuses an object past 0xFF.
// Converted: `rc2k_core::marks::lines::check_object`
pub fn copy_block_points(block: u32) -> Result<(), Refused> {
    {
        let object = get(block)?;
        core::check_object(object)?;
        let mut to = get(block + 0x8C)?;
        let count = OBJECT_SLOTS.point_counts[object as usize].get() as i32;
        let first: u32 = OBJECT_SLOTS.first_point[object as usize].get();
        let mut from = first
            .wrapping_mul(12)
            .wrapping_add(COURSE_LISTS.object_points.bits());
        for _ in 0..count.max(0) {
            for k in 0..3 {
                put(to + 4 * k, get(from + 4 * k)?)?;
            }
            from = from.wrapping_add(12);
            to = to.wrapping_add(12);
        }
    }
    Ok(())
}

/// `marks::set_up_blocks` (0x4D05BD), step 14: `measure_starting_sums`; then for each
/// block (the last first) +0x90 .. +0xCC -1, a block of 12 bytes a point of its object
/// taken on its +0x4C (`memory::allocate`) into `g_marksTaken` and +0x8C,
/// `copy_block_points`, and +0x90 .. +0xAC the eight floats of `g_marksBlockStart`. Refuses
/// as `measure_starting_sums`, before anything is taken.
// Converted: `rc2k_core::marks::lines::{measured, block_at}`
pub fn set_up_blocks() -> Result<(), Refused> {
    measured(crate::marks::setup::SETTINGS.pointer.get(), true)?;
    measure_starting_sums()?;
    let settings: u32 = crate::marks::setup::SETTINGS.pointer.get();
    let count = get(settings + 0x5C)? as i32;
    for k in (0..count.max(0) as u32).rev() {
        let block = block_at(settings, k);
        for o in (0x90..=0xCC).step_by(4) {
            put(block + o, u32::MAX)?;
        }
        let points: u32 =
            ObjectTables::slot_value(&OBJECT_SLOTS.point_counts, get(block)? as usize);
        let taken = crate::memory::allocate::allocate(
            (block + 0x4C) as *mut crate::memory::MemoryList,
            points.wrapping_mul(12),
        )? as u32;
        MORPH_WORK.taken.set(taken);
        put(block + 0x8C, taken)?;
        copy_block_points(block)?;
        for n in 0..8 {
            let v = x87::loaded(LINES_WORK.block_start[n].bits());
            put(block + 0x90 + 4 * n as u32, v)?;
        }
    }
    Ok(())
}

/// The block lines' work (`lines`) (IDA 0x93C738 .. 0x93C8C0, the image's layout; phase 7, typed state).
#[repr(C)]
pub struct LinesWork {
    /// 0x93C738: `g_marksBlockStart`: the eight values a block's +0x90 .. starts with
    pub block_start: [Float; 8],
    /// 0x93C758: `flt_93C73C` .. `flt_93C794`, `dword_93C7F0` .. `flt_93C810`: read only by code in no function (no ported code reads them)
    pub _gap_93c758: Bytes<280>,
    /// 0x93C870: `dword_93C870` .. `dword_93C878`: the last line's x, y, z differences
    pub difference: [Dword; 3],
    /// 0x93C87C: Not named: 44 bytes (a gap, or unreferenced data)
    pub _gap_93c87c: Bytes<44>,
    /// 0x93C8A8: `g_marksMeasuredBlock`: the block being measured
    pub current: Dword,
    /// 0x93C8AC: Not named: 8 bytes (a gap, or unreferenced data)
    pub _gap_93c8ac: Bytes<8>,
    /// 0x93C8B4: `g_marksMeasureEvery`: measure every block
    pub every: Dword,
    /// 0x93C8B8: `g_marksMakeShares`: make the shares
    pub shares: Dword,
    /// 0x93C8BC: `g_marksAnyMeasured`: a marked block was measured
    pub any: Dword,
}

/// LinesWork's first values.
pub static LINES_WORK: LinesWork = LinesWork {
    block_start: [
        Float::from_bits(0x3F800000),
        Float::from_bits(0x3F800000),
        Float::from_bits(0x3F800000),
        Float::from_bits(0x3F800000),
        Float::from_bits(0x0),
        Float::from_bits(0x0),
        Float::from_bits(0x0),
        Float::from_bits(0x0),
    ],
    _gap_93c758: Bytes::new([
        0x0A, 0xD7, 0x23, 0xBC, 0x0A, 0xD7, 0x23, 0xBC, 0x0A, 0xD7, 0x23, 0xBC, 0x0A, 0xD7, 0x23,
        0xBC, 0x0A, 0xD7, 0x23, 0x3C, 0x0A, 0xD7, 0x23, 0x3C, 0x0A, 0xD7, 0x23, 0x3C, 0x0A, 0xD7,
        0x23, 0x3C, 0x0A, 0xD7, 0x23, 0x3C, 0x0A, 0xD7, 0x23, 0x3C, 0x0A, 0xD7, 0x23, 0x3C, 0x0A,
        0xD7, 0x23, 0x3C, 0x0A, 0xD7, 0x23, 0xBC, 0x0A, 0xD7, 0x23, 0xBC, 0x0A, 0xD7, 0x23, 0xBC,
        0x0A, 0xD7, 0x23, 0xBC, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x80, 0x47, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x43,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80, 0x42, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    ]),
    difference: [Dword::new(0x0), Dword::new(0x0), Dword::new(0x0)],
    _gap_93c87c: Bytes::new([0; 44]),
    current: Dword::new(0x0),
    _gap_93c8ac: Bytes::new([0; 8]),
    every: Dword::new(0x0),
    shares: Dword::new(0x0),
    any: Dword::new(0x0),
};

impl LinesWork {
    pub const START: usize = 0x93C738;
    pub const LEN: usize = 0x188;
    /// Where the byte IDA calls `START` is now (`moved::MOVED`'s home).
    pub fn home() -> usize {
        &LINES_WORK as *const LinesWork as usize
    }
}

#[cfg(test)]
mod lines_work_tests {
    use super::*;
    use std::mem::offset_of;

    fn bytes<T>(v: &T) -> Vec<u8> {
        let p = v as *const T as *const u8;
        (0..size_of::<T>())
            .map(|i| unsafe { p.add(i).read() })
            .collect()
    }

    #[test]
    fn lines_work_starts_as_the_games() {
        assert_eq!(size_of::<LinesWork>(), LinesWork::LEN);
        assert!(
            bytes(&LINES_WORK) == crate::moved::image_bytes(LinesWork::START, LinesWork::LEN),
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
