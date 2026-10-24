//! The game's quicksorts of records in its memory (the depth and sort entries, the car's
//! points, the shadow's points) through `rc2k_core::race::quick_sort`: the records from
//! the first to the last (both included) read, sorted, the changed ones written back.

use crate::errors::{BadInput, Refused};
use rc2k_core::race::quick_sort::{Order, quick_sort};

/// The records of `N` dwords from `first` to `last` (both included) sorted by dword `key`
/// in `order` (nothing done when `first` is not below `last`, as its first test). The
/// registers the original leaves are read by none of its callers (conversion phase 3,
/// 2026-10-24). Refuses (before anything) a `last` not a whole number of records from
/// `first` (the original's two places would never meet) and a sort that would go past
/// the records.
pub unsafe fn sort_records<const N: usize>(
    first: u32,
    last: u32,
    key: usize,
    order: Order,
) -> Result<(), Refused> {
    // Converted: `rc2k_core::race::quick_sort::quick_sort`.
    if first >= last {
        return Ok(());
    }
    let size = 4 * N as u32;
    let span = last - first;
    if !span.is_multiple_of(size) {
        return Err(BadInput("a sort's last record not whole records from its first").into());
    }
    let count = (span / size + 1) as usize;
    let at = |n: usize| (first as usize + n * size as usize) as *mut [u32; N];
    let mut list: Vec<[u32; N]> = (0..count)
        .map(|n| unsafe { at(n).read_unaligned() })
        .collect();
    let before = list.clone();
    quick_sort(&mut list, key, order)?;
    for (n, (now, was)) in list.iter().zip(&before).enumerate() {
        if now != was {
            unsafe { at(n).write_unaligned(*now) };
        }
    }
    Ok(())
}
