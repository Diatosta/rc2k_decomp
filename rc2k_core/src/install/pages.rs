//! The installer's page actions: what a page sets up at its start and takes at its end (a negative
//! argument). The items and their marks: `install::model`. Where the pages' answers lead:
//! `install::flow`.

/// When a page action is called.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum When {
    /// The argument is 0.
    Start,
    /// The argument is negative.
    End,
    /// Any other argument.
    Other,
}

impl When {
    pub fn of(argument: u32) -> When {
        match argument as i32 {
            0 => When::Start,
            n if n < 0 => When::End,
            _ => When::Other,
        }
    }
}

/// The drives' texts: 26 of them, 0x10 bytes each, the first byte the letter.
pub(crate) const DRIVE_TEXTS: u32 = 0x1A;
pub const DRIVE_TEXT: u32 = 0x10;

/// The drive page at its start: the place of the typed letter (lowered) among the drives' texts
/// (`letter_at`, asked in order; 0x1A when none), and the highlight it gives, that place plus 2,
/// when that is one of the page's `items` (compared as bytes).
pub fn drive_highlight(letter: u8, letter_at: impl Fn(u32) -> u8, items: u8) -> (u32, Option<u32>) {
    let mut place = 0u32;
    while letter_at(place) != letter && place < DRIVE_TEXTS {
        place += 1;
    }
    let highlight = place.wrapping_add(2);
    (place, ((highlight as u8) < items).then_some(highlight))
}

/// The drive page at its end: the highlighted drive's text (its offset), unless the
/// highlight is on one of the two items before the drives (signed).
pub fn chosen_drive(highlight: u32) -> Option<u32> {
    let n = highlight.wrapping_sub(2);
    ((n as i32) >= 0).then(|| n.wrapping_mul(DRIVE_TEXT))
}

/// Whether the language page is not shown: this version installed already,
/// or at most one language (unsigned).
pub fn language_page_skipped(same_version: bool, languages: u32) -> bool {
    same_version || languages <= 1
}

/// The removal page: answer 2 keeps the saves and settings (the answer is then made 1, the
/// uninstall's).
pub fn keeps_saves(answer: u8) -> bool {
    answer == 2
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pages() {
        assert_eq!(When::of(0), When::Start);
        assert_eq!(When::of(u32::MAX), When::End);
        assert_eq!(When::of(3), When::Other);
        let letters = |n: u32| b"cdf".get(n as usize).copied().unwrap_or(0);
        assert_eq!(drive_highlight(b'd', letters, 10), (1, Some(3)));
        assert_eq!(drive_highlight(b'd', letters, 3), (1, None));
        assert_eq!(drive_highlight(b'z', letters, 0xFF), (0x1A, Some(0x1C)));
        assert_eq!(chosen_drive(3), Some(0x10));
        assert_eq!(chosen_drive(1), None);
        assert!(language_page_skipped(false, 1));
        assert!(!language_page_skipped(false, 2));
        assert!(keeps_saves(2) && !keeps_saves(1));
    }
}
