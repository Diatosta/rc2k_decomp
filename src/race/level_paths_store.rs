//! The picture and level file names that numbers are written into: the twelve `txxxxx.pcx` names
//! of the texture pictures (a name and a second name for each of six folders), then the course's
//! `editpref`, `cdat00c` and `localdat` paths. The fields keep a fixed layout.

use crate::session::PlainBytes;

#[derive(rc2k_derive::Cells)]
pub struct LevelPaths {
    /// `files\a\txxxxx.pcx` (`scene::load_texture_picture` writes the number in).
    pub picture_a: PlainBytes<0x13>,
    /// `files\m\txxxxx.pcx`.
    pub picture_m: PlainBytes<0x13>,
    /// `files\b\txxxxx.pcx`.
    pub picture_b: PlainBytes<0x13>,
    /// `files\b\gravel\txxxxx.pcx`.
    pub picture_gravel: PlainBytes<0x1a>,
    /// `files\g\frnt\txxxxx.pcx`.
    pub picture_front: PlainBytes<0x18>,
    /// `files\s\extr\txxxxx.pcx`.
    pub picture_extra: PlainBytes<0x18>,
    /// The second name of the same, `files\a\txxxxx.pcx`.
    pub second_a: PlainBytes<0x13>,
    /// Second name, `files\m\txxxxx.pcx`.
    pub second_m: PlainBytes<0x13>,
    /// Second name, `files\b\txxxxx.pcx`.
    pub second_b: PlainBytes<0x13>,
    /// Second name, `files\b\gravel\txxxxx.pcx`.
    pub second_gravel: PlainBytes<0x1a>,
    /// Second name, `files\g\frnt\txxxxx.pcx`.
    pub second_front: PlainBytes<0x18>,
    /// Second name, `files\s\extr\txxxxx.pcx`.
    pub second_extra: PlainBytes<0x18>,
    /// `srclvls\level01\editpref` (`race::set_course_number` writes the number at offset 0xD).
    pub edit_path: PlainBytes<0x19>,
    /// `srclvls\level01\cdat00c` (`race::course_data`, `race::find_present_courses`).
    pub cdat_src: PlainBytes<0x18>,
    /// `srclvls\levelxx\localdat` (`race::stage_prefs`).
    pub localdat_src: PlainBytes<0x19>,
    /// `files\levels\levelxx\localdat`.
    pub localdat_files: PlainBytes<0x1e>,
    /// `files\levels\level01\cdat00c.sq`.
    pub cdat_files: PlainBytes<0x20>,
    /// `srclvls\level01\cdat00c`, a second copy of `cdat_src`, whose number
    /// `race::level_init::number_places` writes too.
    pub cdat_src_copy: PlainBytes<0x18>,
    /// `files\s\!!`.
    pub files_s: PlainBytes<0xb>,
}

// SAFETY: every field is a cell over a plain integer (`session`'s cells and records of
// them): any bit pattern is a valid value.

/// The piece (a field of `race::owner::Race::course`), with its first values.
impl LevelPaths {
    pub const fn new() -> Self {
        Self {
            picture_a: PlainBytes::new(*b"files\\a\\txxxxx.pcx\0"),
            picture_m: PlainBytes::new(*b"files\\m\\txxxxx.pcx\0"),
            picture_b: PlainBytes::new(*b"files\\b\\txxxxx.pcx\0"),
            picture_gravel: PlainBytes::new(*b"files\\b\\gravel\\txxxxx.pcx\0"),
            picture_front: PlainBytes::new(*b"files\\g\\frnt\\txxxxx.pcx\0"),
            picture_extra: PlainBytes::new(*b"files\\s\\extr\\txxxxx.pcx\0"),
            second_a: PlainBytes::new(*b"files\\a\\txxxxx.pcx\0"),
            second_m: PlainBytes::new(*b"files\\m\\txxxxx.pcx\0"),
            second_b: PlainBytes::new(*b"files\\b\\txxxxx.pcx\0"),
            second_gravel: PlainBytes::new(*b"files\\b\\gravel\\txxxxx.pcx\0"),
            second_front: PlainBytes::new(*b"files\\g\\frnt\\txxxxx.pcx\0"),
            second_extra: PlainBytes::new(*b"files\\s\\extr\\txxxxx.pcx\0"),
            edit_path: PlainBytes::new(*b"srclvls\\level01\\editpref\0"),
            cdat_src: PlainBytes::new(*b"srclvls\\level01\\cdat00c\0"),
            localdat_src: PlainBytes::new(*b"srclvls\\levelxx\\localdat\0"),
            localdat_files: PlainBytes::new(*b"files\\levels\\levelxx\\localdat\0"),
            cdat_files: PlainBytes::new(*b"files\\levels\\level01\\cdat00c.sq\0"),
            cdat_src_copy: PlainBytes::new(*b"srclvls\\level01\\cdat00c\0"),
            files_s: PlainBytes::new(*b"files\\s\\!!\0"),
        }
    }
}
