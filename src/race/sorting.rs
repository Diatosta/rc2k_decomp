//! The game's quicksorts of records in its memory (the depth and sort entries, the car's
//! points, the shadow's points) through `rc2k_core::race::quick_sort`: the records from
//! the first to the last (both included) read, sorted, the changed ones written back.

use crate::errors::Refused;
use crate::pointers::GamePtr;
use rc2k_core::race::quick_sort::{Order, quick_sort};

/// The `count` records of `N` dwords from `first` sorted by dword `key` in `order`
/// (nothing done with fewer than two, as the original's first test). The registers the
/// original leaves are read by none of its callers (conversion phase 3, 2026-10-24).
/// Refuses (before anything) a sort that would go past the records' bound.
pub fn sort_records<const N: usize>(
    first: GamePtr,
    count: u32,
    key: usize,
    order: Order,
) -> Result<(), Refused> {
    // Converted: `rc2k_core::race::quick_sort::quick_sort`.
    if count < 2 {
        return Ok(());
    }
    let size = 4 * N as u32;
    let count = count as usize;
    // The records are read and written as raw dwords (the sort's own byte layout).
    let at = |n: usize| first.offset((n as u32).wrapping_mul(size));
    let len = count
        .checked_mul(size as usize)
        .ok_or(crate::errors::BadInput(
            "more records to sort than memory holds",
        ))?;
    let bytes = first.read_bytes(len, "the records to sort")?;
    let mut list: Vec<[u32; N]> = bytes
        .chunks_exact(size as usize)
        .map(|record| {
            std::array::from_fn(|k| {
                u32::from_le_bytes([
                    record[4 * k],
                    record[4 * k + 1],
                    record[4 * k + 2],
                    record[4 * k + 3],
                ])
            })
        })
        .collect();
    let before = list.clone();
    quick_sort(&mut list, key, order)?;
    for (n, (now, was)) in list.iter().zip(&before).enumerate() {
        if now != was {
            at(n).write(*now, "a sorted record")?;
        }
    }
    Ok(())
}
