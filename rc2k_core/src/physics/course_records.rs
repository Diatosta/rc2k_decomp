//! The course's records by grid cell (Converted from
//! `physics::find_course_record` 0x4220B6): a record (0x20 bytes, `g_courseRecords`) is
//! looked for by its key (its first dword) among its kind's records, `g_kindFirstRecords`
//! ..= `g_kindLastRecords` of the kind (-1 first: none). The key is a grid cell by the
//! callers: (z >> 21) << 11 + (x >> 21).

use crate::Invalid;

/// A key's kind and the record offset: (kind, ((key >> 11) & 0xF) << 4); the kind is
/// (key & 0xF) + that.
pub fn kind_of(key: u32) -> (u32, u32) {
    let high = ((key >> 11) & 0xF) << 4;
    ((key & 0xF) + high, high)
}

/// The search's end.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Search {
    /// The kind has no records (its first -1).
    Empty,
    /// Record `number` has the key.
    Found { number: u32 },
    /// None has it: `next` the last tried + 1.
    Missing { next: u32 },
}

/// `physics::find_course_record`: the records `first` ..= `last` of the key's kind,
/// `key_of(n)` read in order until one is `key`. Refused (before reading a record) when a
/// kind's last is at or past `count` or before its first.
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
