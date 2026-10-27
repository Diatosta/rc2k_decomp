//! The scene's point records (`g_pointRecords`, 0x40000 bytes of the game's heap on
//! `stru_5C6E50`, taken by `scene::allocate_point_records`): Rust's since phase 13
//! (2026-10-27), counted on the list as the game's block was, given back by
//! `race::free_point_records`. 0x2000 records of 0x20 bytes: +0 the point's number, +0xC
//! the x on the screen, +0x10 .. +0x18 the point in the view, +0x1C the y on the screen. A
//! scene point n has record n & 0xFFF, an object's point or normal n record
//! 0x1000 + (n & 0xFFF).
//!
//! A record is named by its number + 1 (1 .. 0x2000), where the original held its
//! address: in the polygon entries' +4, beside the clipper's records, which are game
//! addresses (at or above 0x10000), so `vertex` / `vertex_bytes` read either.
//! `COURSE_LISTS.point_records` is `HELD` while the records are taken, 0 otherwise.

use crate::errors::{BadInput, Refused};
use crate::memory::owned::Owned;
use crate::race::course_lists_store::COURSE_LISTS;

/// The records.
pub const COUNT: u32 = 0x2000;
/// The block's bytes, as counted.
pub const BYTES: u32 = 0x4_0000;
/// `COURSE_LISTS.point_records` while the records are taken.
pub const HELD: u32 = 1;

pub type Record = [u32; 8];

pub static RECORDS: Owned<Vec<Record>> = Owned::new("no point records");

/// The records made, zeroed, counted on `list`.
pub fn take(list: *mut crate::memory::MemoryList) -> Result<(), Refused> {
    RECORDS.take(list, BYTES, vec![[0; 8]; COUNT as usize])?;
    COURSE_LISTS.point_records.set(HELD);
    Ok(())
}

/// Given back (the list's `free_list` counts them down).
pub fn give_back() {
    RECORDS.give_back();
}

/// Whether the records are taken.
pub fn held() -> bool {
    COURSE_LISTS.point_records.get() != 0
}

/// The record at byte `offset` of the block (the original's `records + offset`).
pub fn at_offset(offset: u32) -> u32 {
    (offset >> 5) + 1
}

/// The record of scene point `n` (`n & 0xFFF`).
pub fn of_scene_point(n: u32) -> u32 {
    (n & 0xFFF) + 1
}

/// The record of an object's point or normal `n` (`0x1000 + (n & 0xFFF)`).
pub fn of_object_point(n: u32) -> u32 {
    (n & 0xFFF) + 0x1001
}

/// Whether `v` names a record (not a game address).
pub fn is_record(v: u32) -> bool {
    v != 0 && v <= COUNT
}

/// Record `r`; refused for none or one past the block.
pub fn get(r: u32) -> Result<Record, BadInput> {
    RECORDS
        .with(|all| all.get((r as usize).checked_sub(1)?).copied())?
        .ok_or(BadInput("a point record past the block"))
}

/// Record `r` set to `record`; refused as `get`.
pub fn set(r: u32, record: Record) -> Result<(), BadInput> {
    RECORDS
        .with(|all| {
            *all.get_mut((r as usize).checked_sub(1)?)? = record;
            Some(())
        })?
        .ok_or(BadInput("a point record past the block"))
}

/// The dword at `offset` of record `r`; refused for an offset past the record.
pub fn field(r: u32, offset: u32) -> Result<u32, BadInput> {
    let record = get(r)?;
    offset
        .is_multiple_of(4)
        .then(|| record.get(offset as usize / 4).copied())
        .flatten()
        .ok_or(BadInput("a field past a point record"))
}

/// The dword at `offset` of record `r` set to `v`; refused as `field`.
pub fn put_field(r: u32, offset: u32, v: u32) -> Result<(), BadInput> {
    let mut record = get(r)?;
    let d = offset
        .is_multiple_of(4)
        .then(|| record.get_mut(offset as usize / 4))
        .flatten()
        .ok_or(BadInput("a field past a point record"))?;
    *d = v;
    set(r, record)
}

/// `f` on all the records.
pub fn with_all<R>(f: impl FnOnce(&mut [Record]) -> R) -> Result<R, BadInput> {
    RECORDS.with(|all| f(all))
}

/// The dword at `offset` of a polygon vertex's record: a point record by number, or a
/// record at a game address (the clipper's). A refusal ends the game, as the original's
/// read of a wrong address went wrong (the polygon code's reads cannot carry one).
pub fn vertex(v: u32, offset: u32) -> u32 {
    if is_record(v) {
        field(v, offset).unwrap_or_else(|e| crate::fatal::raise::end("a point record", e))
    } else {
        crate::pointers::read_or_end::<u32>(v.wrapping_add(offset), "a polygon vertex's record")
    }
}

/// A polygon vertex's whole record (as `vertex`), its bytes.
pub fn vertex_bytes(v: u32) -> Result<Vec<u8>, BadInput> {
    if is_record(v) {
        Ok(get(v)?.iter().flat_map(|d| d.to_le_bytes()).collect())
    } else {
        crate::pointers::GamePtr::new(v).read_bytes(0x20, "a polygon's point record")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn records_are_named_by_number() {
        let _lock = crate::store::test_lock();
        let figures = crate::memory::figures();
        let kept = COURSE_LISTS.point_records.get();
        let mut memory = vec![0u8; size_of::<crate::memory::MemoryList>()];
        let _region = crate::pointers::Lent::new(&memory);
        take(memory.as_mut_ptr() as *mut crate::memory::MemoryList).unwrap();
        assert_eq!(u32::from_le_bytes(memory[4..8].try_into().unwrap()), BYTES);
        assert_eq!(at_offset(0x20 * 5), of_scene_point(0x1005));
        assert_eq!(of_object_point(3), at_offset(0x20 * 0x1003));
        put_field(of_scene_point(7), 0x18, 42).unwrap();
        assert_eq!(vertex(of_scene_point(7), 0x18), 42);
        assert_eq!(get(8).unwrap()[6], 42);
        assert!(get(0).is_err() && get(COUNT + 1).is_err());
        assert!(field(1, 0x20).is_err() && field(1, 2).is_err());
        assert!(!is_record(0x1_0000));
        give_back();
        COURSE_LISTS.point_records.set(kept);
        crate::memory::with_figures(|f| *f = figures);
    }
}
