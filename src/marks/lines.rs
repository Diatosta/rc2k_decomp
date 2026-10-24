//! `marks::init_marks`' step 14, its measuring (IDA, 2026-10-16): the lengths of the
//! lines between two objects' faces (each face's points against the other object's same
//! face's), summed into the eight settings blocks (at `g_marksSettings` + 0x29C, 0x4C8
//! apart) by the face's kind, and then made shares of what they were.

use crate::errors::Refused;
use crate::marks::store::{Slot, game_read, game_write};
use crate::math::x87;
use crate::scene::objects::{
    FACE_COUNTS_RVA, FACES, FACES_RVA, FIRST_FACE_RVA, FIRST_POINT_RVA, POINT_COUNTS_RVA,
    POINTS_RVA,
};
use rc2k_core::marks::lines as core;

pub const SETTINGS_RVA: Slot = Slot::at(0x767A28); // g_marksSettings
/// The last line's x, y, z differences (`dword_93C870` .. `dword_93C878`).
pub const DIFFERENCE_RVA: Slot = Slot::at(0x93C870);
pub const CURRENT_RVA: Slot = Slot::at(0x93C8A8); // dword_93C8A8: the block being measured
pub const EVERY_RVA: Slot = Slot::at(0x93C8B4); // dword_93C8B4: measure every block
pub const SHARES_RVA: Slot = Slot::at(0x93C8B8); // dword_93C8B8: make the shares
pub const ANY_RVA: Slot = Slot::at(0x93C8BC); // dword_93C8BC: a marked block measured
const HUNDRED_RVA: Slot = Slot::at(0x7651E8); // flt_7651E8, 100.0
pub const TAKEN_RVA: Slot = Slot::at(0x93C6CC); // g_marksTaken: the last block taken
const BLOCK_START_RVA: Slot = Slot::at(0x93C738); // g_marksBlockStart: eight floats

unsafe fn get(at: u32) -> u32 {
    unsafe { (at as *const u32).read_unaligned() }
}

unsafe fn put(at: u32, v: u32) {
    unsafe { (at as *mut u32).write_unaligned(v) };
}

unsafe fn byte(at: u32) -> u8 {
    unsafe { *(at as *const u8) }
}

/// The lines of two faces (`from`, `to`): for each of `from`'s points (+0x10, 0x10 apart)
/// the length to `to`'s, added to the whole number at `sum`.
unsafe fn lines(from: u32, to: u32, sum: u32) {
    unsafe { lines_with(from, to, sum, x87_length) }
}

/// The x87's `length_plus_int`.
fn x87_length(d: [i32; 3], sum: i32) -> i32 {
    x87::length_plus_int(d[0], d[1], d[2], sum)
}

/// `lines` with the length worked out by `length`.
unsafe fn lines_with(from: u32, to: u32, sum: u32, length: fn([i32; 3], i32) -> i32) {
    unsafe {
        let count = get(from);
        let points: u32 = game_read(POINTS_RVA);
        let mut n = 0u32;
        while n != count {
            let (a, b) = (get(from + 0x10 + 0x10 * n), get(to + 0x10 + 0x10 * n));
            let p = points.wrapping_add(a.wrapping_mul(12));
            let q = points.wrapping_add(b.wrapping_mul(12));
            let d = core::difference([0, 4, 8].map(|o| get(p + o)), [0, 4, 8].map(|o| get(q + o)));
            game_write(DIFFERENCE_RVA, d);
            put(sum, length(d.map(|v| v as i32), get(sum) as i32) as u32);
            n += 1;
        }
    }
}

/// `marks::measure_slot_lines` (0x4D1167): faces `from` and `to`, block `block`: for each
/// of the block's 8 slots whose kind (the low byte of +0xD0 ..) is the face's +7, its
/// lines into the slot's sum (+0xF0 ..). The x87's, for `measure_lines`.
pub unsafe fn measure_slot_lines(from: u32, to: u32, block: u32) {
    unsafe {
        for k in 0..core::SLOTS {
            if core::slot_takes(get(block + 0xD0 + 4 * k), byte(from + 7)) {
                lines_with(from, to, block + 0xF0 + 4 * k, x87_length);
            }
        }
    }
}

/// `marks::measure_kind_lines` (0x4D1225): faces `from` and `to`, block `block`: for each
/// of the block's 0x11 kinds (the low byte of +0x130 ..) that is the face's +6, with the
/// face's +0xB 1: its point count added to +0x240 .., its lines into +0x1B8 ... The x87's,
/// for `measure_lines`.
pub unsafe fn measure_kind_lines(from: u32, to: u32, block: u32) {
    unsafe {
        for k in 0..core::KINDS {
            if core::kind_takes(get(block + 0x130 + 4 * k), byte(from + 6), byte(from + 0xB)) {
                let count = block + 0x240 + 4 * k;
                put(count, get(count).wrapping_add(get(from)));
                lines_with(from, to, block + 0x1B8 + 4 * k, x87_length);
            }
        }
    }
}

/// The game's tables for `core::measured`: the settings' blocks and the objects' faces.
struct GameBlocks(u32);

impl core::Blocks for GameBlocks {
    fn marked(&self, k: u32) -> bool {
        unsafe { get(core::block_at(self.0, k) + 0xC) != 0 }
    }
    fn objects(&self, k: u32) -> [u32; 2] {
        let block = core::block_at(self.0, k);
        unsafe { [get(block), get(block + 4)] }
    }
    fn face_count(&self, object: u32) -> u32 {
        unsafe { game_read(FACE_COUNTS_RVA + 4 * object as usize) }
    }
    fn first_face(&self, object: u32) -> u32 {
        unsafe { game_read(FIRST_FACE_RVA + 4 * object as usize) }
    }
}

/// The blocks `measure_lines` would measure (`core::measured`): (block, its first faces,
/// its face count).
unsafe fn measured(settings: u32, every: bool) -> Result<Vec<(u32, u32, u32, u32)>, Refused> {
    let count = unsafe { get(settings + 0x5C) };
    let blocks = core::measured(count, every, &GameBlocks(settings), FACES)?;
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
pub unsafe fn measure_lines() -> Result<(), Refused> {
    unsafe {
        let settings: u32 = game_read(SETTINGS_RVA);
        let every = game_read::<u32>(EVERY_RVA) == 1;
        let blocks = measured(settings, every)?;
        game_write(ANY_RVA, 0u32);
        let count = get(settings + 0x5C) as i32;
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
                game_write(ANY_RVA, 1u32);
            }
            game_write(CURRENT_RVA, block);
            for o in (0x1C..=0x30).step_by(4) {
                put(block + o, 0);
            }
            for n in 0..8 {
                put(block + 0xF0 + 4 * n, 0);
            }
            for n in 0..0x11 {
                put(block + 0x1B8 + 4 * n, 0);
                put(block + 0x240 + 4 * n, 0);
            }
            for n in 0..(faces as i32).max(0) as u32 {
                let from: u32 = game_read(FACES_RVA + 4 * (first_a + n) as usize);
                let to: u32 = game_read(FACES_RVA + 4 * (first_b + n) as usize);
                if byte(from + 6) == 0x1C {
                    let kind = byte(from + 0xB) as u32;
                    if (1..=6).contains(&kind) {
                        lines(from, to, block + 0x18 + 4 * kind);
                    }
                }
                measure_slot_lines(from, to, block);
                measure_kind_lines(from, to, block);
            }
            if game_read::<u32>(SHARES_RVA) == 1 {
                let hundred: u32 = game_read(HUNDRED_RVA);
                let share = |at: u32, of: u32| {
                    put(
                        at,
                        x87::share_left_to_int(get(at) as i32, get(of) as i32, hundred) as u32,
                    )
                };
                for o in (0x1C..=0x30).step_by(4) {
                    share(block + o, block + o + 0x18);
                }
                x87::finit();
                for n in (0..8).rev() {
                    share(block + 0xF0 + 4 * n, block + 0x110 + 4 * n);
                }
                for n in (0..0x11).rev() {
                    share(block + 0x1B8 + 4 * n, block + 0x1FC + 4 * n);
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
pub unsafe fn measure_starting_sums() -> Result<(), Refused> {
    unsafe {
        let settings: u32 = game_read(SETTINGS_RVA);
        measured(settings, true)?;
        game_write(EVERY_RVA, 1u32);
        game_write(SHARES_RVA, 0u32);
        measure_lines()?;
        let settings: u32 = game_read(SETTINGS_RVA);
        let count = get(settings + 0x5C) as i32;
        for k in (0..count.max(0) as u32).rev() {
            let block = block_at(settings, k);
            for o in (0x1C..=0x30).step_by(4) {
                put(block + o + 0x18, get(block + o));
            }
            for o in (0x1C..=0x30).step_by(4) {
                put(block + o, 0);
            }
            for (from, to, n) in [(0xF0, 0x110, 8u32), (0x1B8, 0x1FC, 0x11)] {
                for k in (0..n).rev() {
                    put(block + to + 4 * k, get(block + from + 4 * k));
                    put(block + from + 4 * k, 0);
                }
            }
        }
        game_write(EVERY_RVA, 0u32);
        game_write(SHARES_RVA, 1u32);
        Ok(())
    }
}

/// `marks::copy_block_points` (0x4D0A82): the points of block `block`'s object (+0):
/// `g_objectPointCount2` of 12 bytes from `g_objectFirstPoint` in `g_objectPoints`, to the
/// block's +0x8C. Refuses an object past 0xFF.
// Converted: `rc2k_core::marks::lines::check_object`
pub unsafe fn copy_block_points(block: u32) -> Result<(), Refused> {
    unsafe {
        let object = get(block);
        core::check_object(object)?;
        let mut to = get(block + 0x8C);
        let count: i32 = game_read(POINT_COUNTS_RVA + 4 * object as usize);
        let first: u32 = game_read(FIRST_POINT_RVA + 4 * object as usize);
        let mut from = first.wrapping_mul(12).wrapping_add(game_read(POINTS_RVA));
        for _ in 0..count.max(0) {
            for k in 0..3 {
                put(to + 4 * k, get(from + 4 * k));
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
pub unsafe fn set_up_blocks() -> Result<(), Refused> {
    unsafe {
        measured(game_read(SETTINGS_RVA), true)?;
        measure_starting_sums()?;
        let settings: u32 = game_read(SETTINGS_RVA);
        let count = get(settings + 0x5C) as i32;
        for k in (0..count.max(0) as u32).rev() {
            let block = block_at(settings, k);
            for o in (0x90..=0xCC).step_by(4) {
                put(block + o, u32::MAX);
            }
            let points: u32 = game_read(POINT_COUNTS_RVA + 4 * get(block) as usize);
            let taken = crate::memory::allocate::allocate(
                (block + 0x4C) as *mut crate::memory::MemoryList,
                points.wrapping_mul(12),
            )? as u32;
            game_write(TAKEN_RVA, taken);
            put(block + 0x8C, taken);
            copy_block_points(block)?;
            for n in 0..8 {
                let v = x87::loaded(game_read(BLOCK_START_RVA + 4 * n));
                put(block + 0x90 + 4 * n as u32, v);
            }
        }
    }
    Ok(())
}
