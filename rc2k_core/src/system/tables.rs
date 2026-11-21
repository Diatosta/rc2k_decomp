//! Tables of 8-byte entries: the video modes', the screen modes', DirectDraw's and DirectInput's.
//! The searches take the entries as they come, so no more of a table is read than needed.

pub type Entry = [u32; 2];

/// The first entry whose first dword is `key`.
pub fn by_key(t: impl IntoIterator<Item = Entry>, key: u32) -> Option<usize> {
    t.into_iter().position(|e| e[0] == key)
}

#[cfg(test)]
/// The first entry whose second dword is `value`.
pub(crate) fn by_value(t: impl IntoIterator<Item = Entry>, value: u32) -> Option<usize> {
    t.into_iter().position(|e| e[1] == value)
}

/// The first entry whose first dword has any of `bits`.
pub(crate) fn with_any_bit(t: impl IntoIterator<Item = Entry>, bits: u32) -> Option<usize> {
    t.into_iter().position(|e| e[0] & bits != 0)
}

/// The first entry whose first dword has all of `bits`.
pub fn with_bits(t: impl IntoIterator<Item = Entry>, bits: u32) -> Option<usize> {
    t.into_iter().position(|e| e[0] & bits == bits)
}

/// Where a search leaves its two registers: the entry and the entries left with it, or
/// past the table and 0.
pub fn left(table: u32, count: u32, found: Option<usize>) -> (u32, u32) {
    match found {
        Some(n) => (table.wrapping_add(8 * n as u32), count - n as u32),
        None => (table.wrapping_add(count.wrapping_mul(8)), 0),
    }
}

/// One step on (the entry's address, the entries left with it): with none left nothing
/// changes and the first value is true; else on by 8 and one less, true when none are
/// left or the address wrapped.
pub fn next(entry: u32, count: u32) -> (bool, u32, u32) {
    if count == 0 {
        return (true, entry, count);
    }
    let (next, wrapped) = entry.overflowing_add(8);
    (count == 1 || wrapped, next, count - 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tables() {
        let t: [Entry; 3] = [[1, 9], [6, 8], [4, 7]];
        assert_eq!(
            (
                by_key(t, 4),
                by_value(t, 8),
                with_any_bit(t, 2),
                with_bits(t, 6)
            ),
            (Some(2), Some(1), Some(1), Some(1))
        );
        assert_eq!(
            (left(0x100, 3, Some(1)), left(0x100, 3, None)),
            ((0x108, 2), (0x118, 0))
        );
        assert_eq!(
            (next(0x100, 1), next(0xFFFF_FFF8, 5), next(4, 0)),
            ((true, 0x108, 0), (true, 0, 4), (true, 4, 0))
        );
    }
}
