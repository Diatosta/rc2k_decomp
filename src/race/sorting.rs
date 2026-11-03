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
    let at = |n: usize| first.offset((n as u32).wrapping_mul(size));
    let mut list: Vec<[u32; N]> = Vec::with_capacity(count);
    for n in 0..count {
        list.push(at(n).read::<[u32; N]>("a record to sort")?);
    }
    let before = list.clone();
    quick_sort(&mut list, key, order)?;
    for (n, (now, was)) in list.iter().zip(&before).enumerate() {
        if now != was {
            at(n).write(*now, "a sorted record")?;
        }
    }
    Ok(())
}
