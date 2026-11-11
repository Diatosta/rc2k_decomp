//! The copyright notice of the edition whose notice kind is 2 (from
//! `menu::build_notice` 0x4165F1): one line made of the two. Drawn by a menu frame
//! (from `menu::frame_callback` 0x474D4B: `drawn_now`).

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
/// (Software Design) Ltd, ".
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

/// The frame's notice: string 0x1000 at (800, 587), font `FontObjects.objects[3].first_char`.
pub const STRING: u32 = 0x1000;
pub const AT: (u32, u32) = (800, 587);

/// Whether a menu frame draws the notice now: with the notice kind that builds it and the
/// main menu's request (`Cheats.notice_due` 1, read only for that kind; the frame then clears
/// it).
pub fn drawn_now(kind: u8, requested: impl FnOnce() -> u32) -> bool {
    kind == BUILDS && requested() == 1
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

    #[test]
    fn drawn() {
        assert!(drawn_now(2, || 1));
        assert!(!drawn_now(2, || 0));
        assert!(!drawn_now(1, || unreachable!()));
    }
}
