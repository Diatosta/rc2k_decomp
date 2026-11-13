//! The course's records by grid cell: a 0x20-byte record is found by its key (its first dword)
//! among its kind's records, a range per kind (first -1: none). The key is a grid cell:
//! (z >> 21) << 11 + (x >> 21).

use crate::Invalid;

/// A key's kind and record offset: ((key & 0xF) + offset, offset), where
/// offset = ((key >> 11) & 0xF) << 4.
pub fn kind_of(key: u32) -> (u32, u32) {
    let high = ((key >> 11) & 0xF) << 4;
    ((key & 0xF) + high, high)
}

/// How a search for a record ended.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Search {
    /// The kind has no records.
    Empty,
    /// Record `number` has the key.
    Found { number: u32 },
    /// None has it; `next` is the last tried + 1.
    Missing { next: u32 },
}

/// Finds the record with `key` among the key's kind's records `first` ..= `last`, reading
/// `key_of(n)` in order. Refused, before reading a record, when `last` is at or past `count` or
/// before `first`.
pub fn find_record(
    key: u32,
    first: u32,
    last: u32,
    count: u32,
    key_of: impl Fn(u32) -> u32,
) -> Result<Search, Invalid> {
    if first == u32::MAX {
        return Ok(Search::Empty);
    }
    if last >= count || first > last {
        return Err(Invalid(
            "a kind's records past g_courseRecordCount, or ending before they begin",
        ));
    }
    for n in first..=last {
        if key_of(n) == key {
            return Ok(Search::Found { number: n });
        }
    }
    Ok(Search::Missing { next: last + 1 })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn search() {
        assert_eq!(kind_of(0x1803), (0x33, 0x30));
        let keys = [5u32, 7, 9, 11];
        let k = |n: u32| keys[n as usize];
        assert_eq!(find_record(9, 1, 3, 4, k), Ok(Search::Found { number: 2 }));
        assert_eq!(find_record(4, 1, 3, 4, k), Ok(Search::Missing { next: 4 }));
        assert_eq!(
            find_record(4, u32::MAX, 0, 4, |_| unreachable!()),
            Ok(Search::Empty)
        );
        assert!(find_record(4, 1, 4, 4, |_| unreachable!()).is_err());
        assert!(find_record(4, 3, 2, 4, |_| unreachable!()).is_err());
    }
}
