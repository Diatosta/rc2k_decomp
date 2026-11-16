//! The cheats' names: a player's name looked for in the list of coded names. The list holds each
//! name rot13-coded, its NUL and the cheat's bit number; a NUL where a name would start ends it.

use crate::Invalid;
use crate::strings::bytes::Bytes;
use crate::strings::case::CaseTable;
use std::cmp::Ordering;

/// The most entries looked at before the list is taken as having no end.
pub const MOST_ENTRIES: usize = 64;

/// A name found: where its entry starts in the list and the cheat's bit number.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Found {
    pub at: usize,
    pub bit: u8,
}

/// The NUL-ended text at `at` (refused: a list that ends inside it).
fn text_at(list: &(impl Bytes + ?Sized), at: usize) -> Result<Vec<u8>, Invalid> {
    let mut out = Vec::new();
    loop {
        match list.byte(at + out.len()) {
            Some(0) => return Ok(out),
            Some(b) => out.push(b),
            None => return Err(Invalid("a cheat name past the list's end")),
        }
    }
}

/// `menu::check_cheat_name`'s search: the entry whose name is `coded` (case aside, by
/// `case`), or None when the list ends first. Refused: no end in `MOST_ENTRIES` entries.
pub fn find(
    list: &(impl Bytes + ?Sized),
    coded: &[u8],
    case: &CaseTable,
) -> Result<Option<Found>, Invalid> {
    let mut at = 0;
    for _ in 0..MOST_ENTRIES {
        let name = text_at(list, at)?;
        let bit_at = at + name.len() + 1;
        if case.compare(coded, &name) == Ordering::Equal {
            let bit = list
                .byte(bit_at)
                .ok_or(Invalid("a cheat bit past the list's end"))?;
            return Ok(Some(Found { at, bit }));
        }
        at = bit_at + 1;
        match list.byte(at) {
            Some(0) => return Ok(None),
            Some(_) => {}
            None => return Err(Invalid("a cheat list past its end")),
        }
    }
    Err(Invalid("a cheat list without its end in 64 entries"))
}

/// The cheats' bits with `bit` set. Refused: a bit past 31.
pub fn with_cheat(cheats: u32, bit: u8) -> Result<u32, Invalid> {
    if bit > 31 {
        return Err(Invalid("a cheat bit past 31"));
    }
    Ok(cheats | 1 << bit)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::strings::format::rot13;

    fn coded(name: &str) -> Vec<u8> {
        name.bytes().map(rot13).collect()
    }

    fn list() -> Vec<u8> {
        let mut l = Vec::new();
        for (name, bit) in [("Arcade Action", 0u8), ("Spud", 2), ("Bad", 40)] {
            l.extend(coded(name));
            l.extend([0, bit]);
        }
        l.push(0);
        l
    }

    #[test]
    fn finds_names_case_aside() {
        let (l, case) = (list(), CaseTable::ascii());
        assert_eq!(
            find(l.as_slice(), &coded("arcade action"), &case),
            Ok(Some(Found { at: 0, bit: 0 }))
        );
        assert_eq!(
            find(l.as_slice(), &coded("SPUD"), &case),
            Ok(Some(Found { at: 15, bit: 2 }))
        );
        assert_eq!(find(l.as_slice(), &coded("Colin"), &case), Ok(None));
        let bad = find(l.as_slice(), &coded("bad"), &case).unwrap().unwrap();
        assert!(with_cheat(0, bad.bit).is_err());
        assert_eq!(with_cheat(0x8000_0000, 2), Ok(0x8000_0004));
    }

    #[test]
    fn refuses_a_list_without_end() {
        let case = CaseTable::ascii();
        let mut l = Vec::new();
        for _ in 0..MOST_ENTRIES {
            l.extend([b'x', 0, 1]);
        }
        l.push(b'y');
        assert!(find(l.as_slice(), b"z", &case).is_err());
        assert!(find([b'x', 0, 1].as_slice(), b"z", &case).is_err());
    }
}
