//! The keyboard layouts the game types with: the built-in one, the keyboard file's two (and
//! the one it may pick past them), held as owned tables
//! (`rc2k_core::input::layouts`), and which of them is current.
//!
//! A later `load_language` keeps what an earlier one left: the layouts stay unless a new file
//! replaces them, and the new file is read over the old bytes. `language::load_language` fills
//! this at start: the built-in layout and mode 1, then the keyboard file's (its layout mode,
//! layouts and pick). The readers are
//! `input::key_to_char`, `append_key_name`, `yes_no_key` and the name editors.

use crate::input::owner::Input;
use rc2k_core::input::layouts::{FILE_LEN, File, Layout, Picked};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Current {
    /// Before `load_language`: there is no layout.
    None,
    BuiltIn,
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
    file: [Layout; 2],
    file_picked: Layout,
    /// The keyboard file's block as last read (the next file is read over it).
    file_bytes: [u8; FILE_LEN],
}

impl Layouts {
    /// No layout yet: `load_language` fills it in.
    pub const fn new() -> Self {
        Self {
            mode: 0,
            current: Current::None,
            built_in: Layout::EMPTY,
            file: [Layout::EMPTY, Layout::EMPTY],
            file_picked: Layout::EMPTY,
            file_bytes: [0; FILE_LEN],
        }
    }

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
}

#[cfg(test)]
mod tests {
    #[test]
    fn the_built_in_tables_have_their_lengths() {
        let input = crate::test_support::own::input();
        assert_eq!(input.keyboard().plain.read().len(), 0x80);
        assert_eq!(input.keyboard().shifted.read().len(), 0x80);
        assert_eq!(input.keyboard().key_names.read().len(), 0x100);
    }
}
