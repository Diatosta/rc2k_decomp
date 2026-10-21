//! What the game asks of the machine (converted 2026-10-21 from
//! `system::check_developer_machine` 0x416292 and `system::read_keyboard_type` 0x410AA3).

/// Whether the computer is called "andrew" (its first six bytes, as three words each with
/// 0x2020 or'ed in: any case, and some other bytes, pass), and the last word compared, as
/// the original leaves it in ax. A developer's machine is kept off the best tyre-change
/// time.
pub fn developer(name: [u8; 6]) -> (bool, u16) {
    let mut last = 0;
    for (n, pair) in [*b"an", *b"dr", *b"ew"].into_iter().enumerate() {
        last = u16::from_le_bytes([name[2 * n], name[2 * n + 1]]) | 0x2020;
        if last != u16::from_le_bytes(pair) {
            return (false, last);
        }
    }
    (true, last)
}

/// `g_keyboardType` for a keyboard's subtype: 2 (Japanese) for 0xD01 ..= 0xD06.
pub fn keyboard_type(subtype: u32) -> Option<u32> {
    (0xD01..=0xD06).contains(&subtype).then_some(2)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn machine() {
        assert!(developer(*b"ANDREW").0 && developer(*b"andrew").0);
        assert_eq!(developer(*b"andy\0\0"), (false, u16::from_le_bytes(*b"dy")));
        assert_eq!((keyboard_type(0xD03), keyboard_type(4)), (Some(2), None));
    }
}
