//! Quicksorts of records in the race's tables (the depth and sort entries, the car's points, the
//! shadow's points) through `rc2k_core::race::quick_sort`: the records from the first to the last
//! (both included) are read, sorted, and the changed ones written back.

use crate::errors::Refused;
use crate::memory::session::PlainDword;
use rc2k_core::race::quick_sort::{Order, quick_sort};

/// The `count` records of `records` (each as its `N` dwords, `words`) sorted by dword `key` in
/// `order`, the changed records written back whole (nothing is done with fewer than two).
/// Refuses (before anything) more records than `records` holds.
pub fn sort_records<T, const N: usize>(
    records: &[T],
    count: u32,
    key: usize,
    order: Order,
    words: impl Fn(&T) -> [&PlainDword; N],
) -> Result<(), Refused> {
    if count < 2 {
        return Ok(());
    }
    let records = records
        .get(..count as usize)
        .ok_or(crate::errors::BadInput(
            "more records to sort than the list holds",
        ))?;
    let mut list: Vec<[u32; N]> = records
        .iter()
        .map(|record| words(record).map(PlainDword::get))
        .collect();
    let before = list.clone();
    quick_sort(&mut list, key, order)?;
    for ((now, was), record) in list.iter().zip(&before).zip(records) {
        if now != was {
            for (cell, value) in words(record).into_iter().zip(now) {
                cell.set(*value);
            }
        }
    }
    Ok(())
}

/// The `count` records of `N` dwords at the start of `cells` sorted as `sort_records` does,
/// on the cells themselves (the changed records written back). Refuses (before anything)
/// more records than `cells` holds.
pub fn sort_cells<const N: usize>(
    cells: &[PlainDword],
    count: u32,
    key: usize,
    order: Order,
) -> Result<(), Refused> {
    let (records, _) = cells.as_chunks::<N>();
    sort_records(records, count, key, order, <[PlainDword; N]>::each_ref)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cells(values: &[u32]) -> Vec<PlainDword> {
        values.iter().map(|&v| PlainDword::new(v)).collect()
    }

    fn values(cells: &[PlainDword]) -> Vec<u32> {
        cells.iter().map(PlainDword::get).collect()
    }

    /// Records of two dwords sorted up and down by their first dword, the second carried along;
    /// the records after `count` stay.
    #[test]
    fn sorts_the_first_records_and_carries_their_words() {
        let list = cells(&[5, 50, 1, 10, 3, 30, 9, 90, 0, 0]);
        sort_cells::<2>(&list, 4, 0, Order::Up).unwrap();
        assert_eq!(values(&list), [1, 10, 3, 30, 5, 50, 9, 90, 0, 0]);
        sort_cells::<2>(&list, 4, 0, Order::Down).unwrap();
        assert_eq!(values(&list), [9, 90, 5, 50, 3, 30, 1, 10, 0, 0]);
        let list = cells(&[2, 1, 1, 0]);
        sort_cells::<2>(&list, 2, 1, Order::Up).unwrap();
        assert_eq!(values(&list), [1, 0, 2, 1]);
    }

    /// Fewer than two records are left alone, more than the list holds are refused.
    #[test]
    fn few_records_are_left_and_too_many_refused() {
        let list = cells(&[7, 70, 6, 60]);
        sort_cells::<2>(&list, 1, 0, Order::Up).unwrap();
        assert_eq!(values(&list), [7, 70, 6, 60]);
        assert!(sort_cells::<2>(&list, 3, 0, Order::Up).is_err());
        assert_eq!(values(&list), [7, 70, 6, 60]);
    }
}
