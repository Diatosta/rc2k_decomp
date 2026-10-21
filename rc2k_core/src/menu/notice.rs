//! The copyright notice of the edition whose notice kind is 2 (converted 2026-10-21 from
//! `menu::build_notice` 0x4165F1): one line made of the two.

/// The notice kind (`g_noticeKind`, from the language's `.loc` file) that builds it.
pub const BUILDS: u8 = 2;

/// The bytes of the first line and of the second that are read: from the twelfth (the
/// eleventh) on, so many.
pub const FIRST_FROM: usize = 11;
pub const FIRST_READ: usize = 0x26;
pub const SECOND_FROM: usize = 10;
pub const SECOND_READ: usize = 15;

/// The line built: 2 + 4 + 14 + 4 + 0x26 + 2.
pub const BUILT_LEN: usize = 0x40;

/// The line, from pieces of the two: two characters of the second from its eleventh on,
/// "2000", fourteen of it from its twelfth on, " and", 0x26 of the first from its twelfth
/// on, a space and a NUL (`first` and `second` the bytes read). The game's lines make "(c) 2000 Actualize Ltd and Magnetic Fields
/// (Software Design) Ltd, " (the hook's header).
pub fn notice(first: &[u8; FIRST_READ], second: &[u8; SECOND_READ]) -> [u8; BUILT_LEN] {
    let mut built = [0; BUILT_LEN];
    let pieces: [&[u8]; 6] = [&second[..2], b"2000", &second[1..], b" and", first, b" \0"];
    let mut at = 0;
    for piece in pieces {
        built[at..at + piece.len()].copy_from_slice(piece);
        at += piece.len();
    }
    built
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn built() {
        let first: [u8; FIRST_READ] = std::array::from_fn(|n| b'a' + (n % 26) as u8);
        let second: [u8; SECOND_READ] = std::array::from_fn(|n| b'A' + n as u8);
        let built = notice(&first, &second);
        assert_eq!(&built[..6], b"AB2000");
        assert_eq!(&built[6..20], b"BCDEFGHIJKLMNO");
        assert_eq!(&built[20..24], b" and");
        assert_eq!(&built[24..62], &first[..]);
        assert_eq!(&built[62..], b" \0");
    }
}
