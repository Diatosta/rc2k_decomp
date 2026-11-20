//! The picture and level file names that numbers are written into: the twelve `txxxxx.pcx` names
//! of the texture pictures (a name and a second name for each of six folders), then the course's
//! `editpref`, `cdat00c` and `localdat` paths. Each name is a NUL-ended text in a fixed buffer.

use crate::memory::session::TextPlace;

#[derive(Clone)]
pub struct LevelPaths {
    /// `files\a\txxxxx.pcx` (`scene::load_texture_picture` writes the number in).
    pub picture_a: [u8; 0x13],
    /// `files\m\txxxxx.pcx`.
    pub picture_m: [u8; 0x13],
    /// `files\b\txxxxx.pcx`.
    pub picture_b: [u8; 0x13],
    /// `files\b\gravel\txxxxx.pcx`.
    pub picture_gravel: [u8; 0x1a],
    /// `files\g\frnt\txxxxx.pcx`.
    pub picture_front: [u8; 0x18],
    /// `files\s\extr\txxxxx.pcx`.
    pub picture_extra: [u8; 0x18],
    /// The second name of the same, `files\a\txxxxx.pcx`.
    pub second_a: [u8; 0x13],
    /// Second name, `files\m\txxxxx.pcx`.
    pub second_m: [u8; 0x13],
    /// Second name, `files\b\txxxxx.pcx`.
    pub second_b: [u8; 0x13],
    /// Second name, `files\b\gravel\txxxxx.pcx`.
    pub second_gravel: [u8; 0x1a],
    /// Second name, `files\g\frnt\txxxxx.pcx`.
    pub second_front: [u8; 0x18],
    /// Second name, `files\s\extr\txxxxx.pcx`.
    pub second_extra: [u8; 0x18],
    /// `srclvls\level01\editpref` (`set_course_number` writes the number at offset 0xD).
    pub edit_path: [u8; 0x19],
    /// `srclvls\level01\cdat00c` (`race::course_data`, `race::find_present_courses`).
    pub cdat_src: [u8; 0x18],
    /// `srclvls\levelxx\localdat` (`race::stage_prefs`).
    pub localdat_src: [u8; 0x19],
    /// `files\levels\levelxx\localdat`.
    pub localdat_files: [u8; 0x1e],
    /// `files\levels\level01\cdat00c.sq`.
    pub cdat_files: [u8; 0x20],
    /// `srclvls\level01\cdat00c`, a second copy of `cdat_src`, whose number
    /// `set_course_number` writes too.
    pub cdat_src_copy: [u8; 0x18],
}

/// Where the two number digits stand in a path of each kind.
const EDIT_AT: usize = 0xD;
const SRC_AT: usize = 0xD;
const FILES_AT: usize = 0x12;
/// Where the digits stand in the cdat paths `set_cdat_number` writes.
const CDAT_SRC_AT: usize = 0x14;
const CDAT_FILES_AT: usize = 0x19;

/// The first values of `LevelPaths`.
impl LevelPaths {
    pub const fn new() -> Self {
        Self {
            picture_a: *b"files\\a\\txxxxx.pcx\0",
            picture_m: *b"files\\m\\txxxxx.pcx\0",
            picture_b: *b"files\\b\\txxxxx.pcx\0",
            picture_gravel: *b"files\\b\\gravel\\txxxxx.pcx\0",
            picture_front: *b"files\\g\\frnt\\txxxxx.pcx\0",
            picture_extra: *b"files\\s\\extr\\txxxxx.pcx\0",
            second_a: *b"files\\a\\txxxxx.pcx\0",
            second_m: *b"files\\m\\txxxxx.pcx\0",
            second_b: *b"files\\b\\txxxxx.pcx\0",
            second_gravel: *b"files\\b\\gravel\\txxxxx.pcx\0",
            second_front: *b"files\\g\\frnt\\txxxxx.pcx\0",
            second_extra: *b"files\\s\\extr\\txxxxx.pcx\0",
            edit_path: *b"srclvls\\level01\\editpref\0",
            cdat_src: *b"srclvls\\level01\\cdat00c\0",
            localdat_src: *b"srclvls\\levelxx\\localdat\0",
            localdat_files: *b"files\\levels\\levelxx\\localdat\0",
            cdat_files: *b"files\\levels\\level01\\cdat00c.sq\0",
            cdat_src_copy: *b"srclvls\\level01\\cdat00c\0",
        }
    }

    /// The course data's source path up to its NUL (all of it when none).
    pub fn cdat_src_text(&self) -> Vec<u8> {
        let end = self
            .cdat_src
            .iter()
            .position(|&b| b == 0)
            .unwrap_or(self.cdat_src.len());
        self.cdat_src[..end].to_vec()
    }

    /// Writes the course's two digits into the six paths that carry its number.
    pub fn set_course_number(&mut self, digits: [u8; 2]) {
        self.edit_path[EDIT_AT..EDIT_AT + 2].copy_from_slice(&digits);
        self.cdat_src[SRC_AT..SRC_AT + 2].copy_from_slice(&digits);
        self.cdat_files[FILES_AT..FILES_AT + 2].copy_from_slice(&digits);
        self.cdat_src_copy[SRC_AT..SRC_AT + 2].copy_from_slice(&digits);
        self.localdat_src[SRC_AT..SRC_AT + 2].copy_from_slice(&digits);
        self.localdat_files[FILES_AT..FILES_AT + 2].copy_from_slice(&digits);
    }

    /// Writes the two digits into both cdat paths (their own places, past the course number's).
    pub fn set_cdat_number(&mut self, digits: [u8; 2]) {
        self.cdat_src[CDAT_SRC_AT..CDAT_SRC_AT + 2].copy_from_slice(&digits);
        self.cdat_files[CDAT_FILES_AT..CDAT_FILES_AT + 2].copy_from_slice(&digits);
    }

    /// The six folders' names and second names with the offset of the number in each.
    pub fn picture_names_mut(&mut self) -> [(&mut [u8], &mut [u8], usize); 6] {
        [
            (&mut self.picture_a, &mut self.second_a, 0x0A), // files\a
            (&mut self.picture_m, &mut self.second_m, 0x0A), // files\m
            (&mut self.picture_b, &mut self.second_b, 0x0A), // files\b
            (&mut self.picture_gravel, &mut self.second_gravel, 0x11), // files\b\gravel
            (&mut self.picture_front, &mut self.second_front, 0x0F), // files\g\frnt
            (&mut self.picture_extra, &mut self.second_extra, 0x0F), // files\s\extr
        ]
    }

    /// The six folders' names and second names.
    pub fn picture_names(&self) -> [(&dyn TextPlace, &dyn TextPlace); 6] {
        [
            (&self.picture_a, &self.second_a),
            (&self.picture_m, &self.second_m),
            (&self.picture_b, &self.second_b),
            (&self.picture_gravel, &self.second_gravel),
            (&self.picture_front, &self.second_front),
            (&self.picture_extra, &self.second_extra),
        ]
    }
}
