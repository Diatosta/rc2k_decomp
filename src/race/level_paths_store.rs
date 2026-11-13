//! The picture and level file names the game writes numbers into (IDA 0x60AF0C .. 0x60B0BD;
//! phase 7, typed): the twelve `txxxxx.pcx` names of the texture pictures (a name and a
//! second name for each of six folders), then the course's `editpref`, `cdat00c` and
//! `localdat` paths. The fields are in the image's order. Names are readings unless IDA's.

use crate::session::PlainBytes;

#[derive(rc2k_derive::Cells)]
pub struct LevelPaths {
    /// 0x60AF0C: `files\a\txxxxx.pcx` (`scene::load_texture_picture`; the number written in).
    pub picture_a: PlainBytes<0x13>,
    /// 0x60AF1F: `files\m\txxxxx.pcx`.
    pub picture_m: PlainBytes<0x13>,
    /// 0x60AF32: `files\b\txxxxx.pcx`.
    pub picture_b: PlainBytes<0x13>,
    /// 0x60AF45: `files\b\gravel\txxxxx.pcx`.
    pub picture_gravel: PlainBytes<0x1a>,
    /// 0x60AF5F: `files\g\frnt\txxxxx.pcx`.
    pub picture_front: PlainBytes<0x18>,
    /// 0x60AF77: `files\s\extr\txxxxx.pcx`.
    pub picture_extra: PlainBytes<0x18>,
    /// 0x60AF8F: the second name of the same, `files\a\txxxxx.pcx`.
    pub second_a: PlainBytes<0x13>,
    /// 0x60AFA2: second name, `files\m\txxxxx.pcx`.
    pub second_m: PlainBytes<0x13>,
    /// 0x60AFB5: second name, `files\b\txxxxx.pcx`.
    pub second_b: PlainBytes<0x13>,
    /// 0x60AFC8: second name, `files\b\gravel\txxxxx.pcx`.
    pub second_gravel: PlainBytes<0x1a>,
    /// 0x60AFE2: second name, `files\g\frnt\txxxxx.pcx`.
    pub second_front: PlainBytes<0x18>,
    /// 0x60AFFA: second name, `files\s\extr\txxxxx.pcx`.
    pub second_extra: PlainBytes<0x18>,
    /// 0x60B012: `srclvls\level01\editpref` (`race::set_course_number` writes the number at +0xD).
    pub edit_path: PlainBytes<0x19>,
    /// 0x60B02B: `srclvls\level01\cdat00c` (`race::course_data`, `race::find_present_courses`).
    pub cdat_src: PlainBytes<0x18>,
    /// 0x60B043: `srclvls\levelxx\localdat` (`race::stage_prefs`).
    pub localdat_src: PlainBytes<0x19>,
    /// 0x60B05C: `files\levels\levelxx\localdat`.
    pub localdat_files: PlainBytes<0x1e>,
    /// 0x60B07A: `files\levels\level01\cdat00c.sq`.
    pub cdat_files: PlainBytes<0x20>,
    /// 0x60B09A: `srclvls\level01\cdat00c`, a second copy of `cdat_src`, whose number `race::level_init::number_places` writes too.
    pub cdat_src_copy: PlainBytes<0x18>,
    /// 0x60B0B2: `files\s\!!`.
    pub files_s: PlainBytes<0xb>,
}

// SAFETY: every field is a cell over a plain integer (`session`'s cells and records of
// them): any bit pattern is a valid value.

/// The piece (a field of `race::owner::Race::course`), with the image's first values.
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
