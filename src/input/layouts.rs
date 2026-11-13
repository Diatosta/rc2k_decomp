//! The keyboard layouts the game types with: the built-in one, the keyboard file's two (and
//! the one it may pick past them) and the NEC keyboard's, held as owned tables
//! (`rc2k_core::input::layouts`), and which of them is current.
//!
//! A later `load_language` keeps what an earlier one left: the layouts stay unless a new file
//! replaces them, and the new file is read over the old bytes. `strings::load_language` fills
//! this at start: the built-in layout and mode 1, then the keyboard file's (its layout mode,
//! layouts and pick), then with keyboard type 2 (NEC) its own layout. The readers are
//! `input::key_to_char`, `append_key_name`, `yes_no_key` and the name editors.

use crate::input::owner::Input;
use crate::strings::owner::Strings;
use rc2k_core::input::layouts::{FILE_LEN, File, Layout, Picked};
use std::sync::{Mutex, MutexGuard, PoisonError};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Current {
    /// Before `load_language`: there is no layout.
    None,
    BuiltIn,
    /// The NEC keyboard's, with keyboard type 2.
    Nec,
    /// The first (0) or second (1) of the keyboard file's layouts.
    File(usize),
    /// The layout past those two that the keyboard file picked.
    FilePicked,
}

pub struct Layouts {
    /// 2 lets Alt (or AltGr) with a shift key switch the layout.
    mode: u8,
    current: Current,
    built_in: Layout,
    nec: Layout,
    file: [Layout; 2],
    file_picked: Layout,
    /// The keyboard file's block as last read (the next file is read over it) and, after
    /// keyboard type 2, with the NEC head over its first 0x30 bytes.
    file_bytes: [u8; FILE_LEN],
}

static LAYOUTS: Mutex<Layouts> = Mutex::new(Layouts {
    mode: 0,
    current: Current::None,
    built_in: Layout::EMPTY,
    nec: Layout::EMPTY,
    file: [Layout::EMPTY, Layout::EMPTY],
    file_picked: Layout::EMPTY,
    file_bytes: [0; FILE_LEN],
});

pub fn layouts() -> MutexGuard<'static, Layouts> {
    LAYOUTS.lock().unwrap_or_else(PoisonError::into_inner)
}

/// The NEC layout block; its tables follow at +0x14 (plain), +0x94 (shifted) and +0x114 (key
/// names, 0x100 bytes).
const NEC_TABLES: (usize, usize, usize, usize) = (0x14, 0x94, 0x114, 0x214);
/// `crate::strings::owner::strings().language.keyboard.nec_head`'s layout mode byte (+0xA).
const NEC_MODE_AT: usize = 0xA;

impl Layouts {
    pub fn mode(&self) -> u8 {
        self.mode
    }

    pub fn current(&self) -> Current {
        self.current
    }

    pub fn set_current(&mut self, current: Current) {
        self.current = current;
    }

    /// The layout `which` names, `None` for no layout.
    pub fn layout(&self, which: Current) -> Option<&Layout> {
        match which {
            Current::None => None,
            Current::BuiltIn => Some(&self.built_in),
            Current::Nec => Some(&self.nec),
            Current::File(n) => self.file.get(n),
            Current::FilePicked => Some(&self.file_picked),
        }
    }

    /// The start of `load_language`: the built-in layout, in mode 1.
    pub fn use_built_in(&mut self, input: &Input) {
        self.built_in = Layout::simple(
            &input.keyboard().plain.read(),
            &input.keyboard().shifted.read(),
            &input.keyboard().key_names.read(),
        );
        self.current = Current::BuiltIn;
        self.mode = 1;
    }

    /// The block the next keyboard file is read over (what the last load left in it).
    pub fn file_bytes(&self) -> [u8; FILE_LEN] {
        self.file_bytes
    }

    /// The keyboard file read: its mode, its layouts and the one it picks.
    pub fn use_file(&mut self, file: File) {
        self.file_bytes = file.bytes;
        self.mode = file.mode;
        self.file = file.layouts;
        match file.picked {
            Picked::Listed(n) => self.current = Current::File(n),
            Picked::Unlisted(layout) => {
                self.file_picked = layout;
                self.current = Current::FilePicked;
            }
        }
    }

    /// Keyboard type 2: the NEC keyboard's head (its mode) and layout take the file's place.
    pub fn use_nec(&mut self, strings: &Strings) {
        let tables = strings.language.keyboard.nec_layout;
        let (plain, shifted, names, end) = NEC_TABLES;
        self.nec = Layout::simple(
            &tables[plain..shifted],
            &tables[shifted..names],
            &tables[names..end],
        );
        let head = strings.language.keyboard.nec_head;
        self.file_bytes[..head.len()].copy_from_slice(&head);
        self.mode = head[NEC_MODE_AT];
        self.current = Current::Nec;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::owner::input;
    use crate::strings::owner::strings;

    #[test]
    fn the_built_in_tables_have_their_lengths() {
        let input = input();
        assert_eq!(input.keyboard().plain.read().len(), 0x80);
        assert_eq!(input.keyboard().shifted.read().len(), 0x80);
        assert_eq!(input.keyboard().key_names.read().len(), 0x100);
    }

    #[test]
    fn the_nec_tables_have_their_lengths() {
        let strings = strings();
        let (_, _, names, end) = NEC_TABLES;
        assert_eq!(end - names, 0x100);
        assert!(end <= strings.language.keyboard.nec_layout.len());
        assert_eq!(strings.language.keyboard.nec_head[NEC_MODE_AT], 1);
    }
}
