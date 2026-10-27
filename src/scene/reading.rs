//! Readers of the game's memory for the converted cores, whose closures cannot fail: the
//! first bad address is kept, the reader answers 0 from then on, and `finish` refuses
//! afterwards (so the core's own answer on those zeros is never used).
//!
//! Local to the callers of `pointers::GamePtr` in `scene` and `drive` (phase 8); not a
//! change of `pointers.rs`.

use crate::errors::BadInput;
use crate::pointers::GamePtr;
use std::cell::Cell;

/// Reads that remember their first refusal.
#[derive(Default)]
pub struct Reading {
    failed: Cell<Option<BadInput>>,
}

impl Reading {
    pub fn new() -> Self {
        Self::default()
    }

    /// The `T` at `addr`; its default after a refusal.
    #[track_caller]
    pub fn read<T: Copy + Default>(&self, addr: u32, what: &str) -> T {
        if self.failed.get().is_some() {
            return T::default();
        }
        GamePtr::new(addr).read(what).unwrap_or_else(|e| {
            self.failed.set(Some(e));
            T::default()
        })
    }

    /// `value`'s result kept as a read's: its default after a refusal (this one or one
    /// before).
    pub fn keep<T: Default>(&self, value: Result<T, BadInput>) -> T {
        if self.failed.get().is_some() {
            return T::default();
        }
        value.unwrap_or_else(|e| {
            self.failed.set(Some(e));
            T::default()
        })
    }

    /// The first refusal, if any read had one.
    pub fn finish(&self) -> Result<(), BadInput> {
        match self.failed.take() {
            Some(e) => Err(e),
            None => Ok(()),
        }
    }
}

/// The objects' point records (`scene::point_records`, 0x1000 + (n & 0xFFF) for a point or
/// normal number `n`; +0x10 its place, three words).
pub struct PointRecords;

impl PointRecords {
    /// Refused when the records are not taken.
    pub fn new() -> Result<Self, BadInput> {
        if !crate::scene::point_records::held() {
            return Err(BadInput("no records for the points"));
        }
        Ok(Self)
    }

    /// The place of point or normal `number`.
    pub fn place(&self, number: u32) -> Result<[u32; 3], BadInput> {
        use crate::scene::point_records::{get, of_object_point};
        let r = get(of_object_point(number))?;
        Ok([r[4], r[5], r[6]])
    }
}

/// `count` records of `N` words from `addr` on, copied out.
pub fn read_records<const N: usize>(
    addr: u32,
    count: usize,
    what: &str,
) -> Result<Vec<[u32; N]>, BadInput> {
    let len = count
        .checked_mul(4 * N)
        .ok_or(BadInput("a run of records too long"))?;
    let bytes = GamePtr::new(addr).read_bytes(len, what)?;
    Ok(bytes
        .chunks_exact(4 * N)
        .map(|record| {
            std::array::from_fn(|n| {
                u32::from_le_bytes(record[4 * n..4 * n + 4].try_into().unwrap())
            })
        })
        .collect())
}

/// The records written from `addr` on.
pub fn write_records<const N: usize>(
    addr: u32,
    records: &[[u32; N]],
    what: &str,
) -> Result<(), BadInput> {
    let bytes: Vec<u8> = records
        .iter()
        .flat_map(|record| record.iter().flat_map(|word| word.to_le_bytes()))
        .collect();
    GamePtr::new(addr).write_bytes(&bytes, what)
}
