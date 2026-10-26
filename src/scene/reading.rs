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
    pub fn read<T: Copy + Default>(&self, addr: u32, what: &str) -> T {
        if self.failed.get().is_some() {
            return T::default();
        }
        GamePtr::new(addr).read(what).unwrap_or_else(|e| {
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

/// The scene's point records (32 bytes each, from `records + 0x20000`; a point or normal
/// number `n` is record `n & 0xFFF`; +0x10 its place, three words): the run is resolved
/// once, then indexed (a face's vertices are many).
pub struct PointRecords {
    bytes: crate::pointers::GameBytes,
}

impl PointRecords {
    /// The records of the block `records` (`COURSE_LISTS.point_records`).
    pub fn new(records: u32) -> Result<Self, BadInput> {
        let bytes = GamePtr::new(records.wrapping_add(0x20000)).bytes("the point records")?;
        Ok(Self { bytes })
    }

    /// The place of point or normal `number`.
    pub fn place(&self, number: u32) -> Result<[u32; 3], BadInput> {
        use rc2k_core::strings::bytes::Bytes;
        let at = ((number & 0xFFF) as usize) * 32 + 0x10;
        let mut out = [0u32; 3];
        for (n, word) in out.iter_mut().enumerate() {
            let mut le = [0u8; 4];
            for (k, byte) in le.iter_mut().enumerate() {
                *byte = self.bytes.byte(at + n * 4 + k).ok_or(BadInput(
                    "a point record past the end of its block (rc2k.log names none)",
                ))?;
            }
            *word = u32::from_le_bytes(le);
        }
        Ok(out)
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
