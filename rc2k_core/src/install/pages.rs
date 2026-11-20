//! The installer's page actions: what a page sets up at its start and takes at its end (a negative
//! argument), and which items are marked (an item's first byte: 0x40 hidden, 0x80 not to be
//! chosen). Where the pages' answers lead: `install::flow`.

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

/// The item marks: hidden, and not to be chosen.
pub const HIDDEN: u8 = 0x40;
pub const UNUSABLE: u8 = 0x80;

/// A change to an item's first byte: `(offset in the list, or, and)`, made in order.
pub type Mark = (u32, u8, u8);

/// The string id of the update script's title.
pub(crate) const UPDATE_TITLE: u16 = 0xBD;

/// The main page's marks: not this version, "play" (+0x28) unusable; no installation, "uninstall"
/// (+0x38) unusable; for the update script, "play" usable again and "uninstall" hidden.
pub fn main_page_marks(same_version: bool, installed: bool, title: u16) -> Vec<Mark> {
    let mut marks = Vec::new();
    if !same_version {
        marks.push((0x28, UNUSABLE, 0xFF));
    }
    if !installed {
        marks.push((0x38, UNUSABLE, 0xFF));
    }
    if title == UPDATE_TITLE {
        marks.extend([(0x28, 0, !UNUSABLE), (0x38, HIDDEN, 0xFF)]);
    }
    marks
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

/// The folder page's problems: 1 a bad folder, 2 no room.
pub(crate) const BAD_FOLDER: u32 = 1;
pub(crate) const NO_ROOM: u32 = 2;

/// Which of the folder page's two notices are hidden (+0x38 and +0x48 for a bad folder,
/// +0x58 and +0x68 for no room): those that are not the problem.
pub fn hidden_notices(problem: u32) -> (bool, bool) {
    (problem != BAD_FOLDER, problem != NO_ROOM)
}

/// Whether the language page is not shown: this version installed already,
/// or at most one language (unsigned).
pub fn language_page_skipped(same_version: bool, languages: u32) -> bool {
    same_version || languages <= 1
}

/// The language page's items past the languages, hidden: from item `languages` on, 9 less the
/// languages of them (wrapping; 0: none).
pub fn hidden_languages(languages: u32) -> (u32, u32) {
    (languages.wrapping_mul(0x10), 9u32.wrapping_sub(languages))
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
        assert_eq!(main_page_marks(true, true, 0), vec![]);
        assert_eq!(
            main_page_marks(false, true, UPDATE_TITLE),
            vec![(0x28, 0x80, 0xFF), (0x28, 0, 0x7F), (0x38, 0x40, 0xFF)]
        );
        let letters = |n: u32| b"cdf".get(n as usize).copied().unwrap_or(0);
        assert_eq!(drive_highlight(b'd', letters, 10), (1, Some(3)));
        assert_eq!(drive_highlight(b'd', letters, 3), (1, None));
        assert_eq!(drive_highlight(b'z', letters, 0xFF), (0x1A, Some(0x1C)));
        assert_eq!(chosen_drive(3), Some(0x10));
        assert_eq!(chosen_drive(1), None);
        assert_eq!(hidden_notices(1), (false, true));
        assert_eq!(hidden_notices(0), (true, true));
        assert!(language_page_skipped(false, 1));
        assert!(!language_page_skipped(false, 2));
        assert_eq!(hidden_languages(3), (0x30, 6));
        assert!(keeps_saves(2) && !keeps_saves(1));
    }
}
