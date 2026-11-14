//! Quicksorts of records in the race's tables (the depth and sort entries, the car's points, the
//! shadow's points) through `rc2k_core::race::quick_sort`: the records from the first to the last
//! (both included) are read, sorted, and the changed ones written back.

use crate::cells::Store;
use crate::errors::Refused;
use rc2k_core::race::quick_sort::{Order, quick_sort};

/// The `count` records of `N` dwords from `first` (a store) sorted by dword `key` in `order`
/// (nothing is done with fewer than two). Refuses (before anything) a sort
/// that would go past the records' bound.
pub fn sort_records<const N: usize>(
    first: &dyn Store,
    count: u32,
    key: usize,
    order: Order,
) -> Result<(), Refused> {
    if count < 2 {
        return Ok(());
    }
    let size = 4 * N as u32;
    let count = count as usize;
    // The records are read and written as raw dwords, in the layout the sort uses.
    let len = count
        .checked_mul(size as usize)
        .ok_or(crate::errors::BadInput(
            "more records to sort than memory holds",
        ))?;
    let mut bytes = vec![0; len];
    first.load(0, &mut bytes)?;
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
            let mut record = Vec::with_capacity(size as usize);
            for value in now {
                record.extend_from_slice(&value.to_le_bytes());
            }
            first.store(n * size as usize, &record)?;
        }
    }
    Ok(())
}

/// The `count` records of `N` dwords at the start of `cells` sorted as `sort_records` does,
/// on the cells themselves (the changed records written back). Refuses (before anything)
/// more records than `cells` holds.
pub fn sort_cells<const N: usize>(
    cells: &[crate::session::PlainDword],
    count: u32,
    key: usize,
    order: Order,
) -> Result<(), Refused> {
    if count < 2 {
        return Ok(());
    }
    let records = cells
        .get(..N.saturating_mul(count as usize))
        .ok_or(crate::errors::BadInput(
            "more records to sort than the list holds",
        ))?;
    let (records, _) = records.as_chunks::<N>();
    let mut list: Vec<[u32; N]> = records
        .iter()
        .map(|record| std::array::from_fn(|k| record[k].get()))
        .collect();
    let before = list.clone();
    quick_sort(&mut list, key, order)?;
    for ((now, was), record) in list.iter().zip(&before).zip(records) {
        if now != was {
            for (cell, value) in record.iter().zip(now) {
                cell.set(*value);
            }
        }
    }
    Ok(())
}
