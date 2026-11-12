//! The program's own texts (version, copyright, window and class names), out of RAL.EXE's
//! image . `#[repr(C)]` struct of cells with the image's layout,
//! first values the image's (`tests`).

use crate::session::{PlainByte, PlainBytes, with_runs};

/// The version, copyright, window and class texts (IDA 0x4E0000 .. 0x4E0094, the image's layout).
#[repr(C, align(16))]
#[derive(rc2k_derive::Cells)]
pub struct StartTexts {
    /// 0x4E0000: `aVerRally530482`: "$VER: Rally 5.30 (4.8.2000)" (`system::read_command_line`, `install::check_at_start`).
    pub version: PlainBytes<28>,
    /// 0x4E001C: `byte_4E001C`: the copyright text, "Copyright (c) Magnetic Fields .." (`install::check_installed_version` looks for it in the program).
    pub copyright: PlainBytes<91>,
    /// 0x4E0077: `byte_4E0077`: the second text of the `-v` box (empty).
    pub second_text: PlainByte,
    /// 0x4E0078: `WindowName`: the window's name, "Rally 5".
    pub window_name: PlainBytes<8>,
    /// 0x4E0080: `ClassName`: the window class's name, "MF-SDG R5" (`system::find_running_game`).
    pub class_name: PlainBytes<10>,
    /// 0x4E008A: Not named: 10 bytes (a gap, or data nothing reads here).
    pub _gap_4e008a: PlainBytes<22>,
}

impl StartTexts {
    pub const fn new() -> Self {
        StartTexts {
    version: PlainBytes::new(with_runs::<28>(&[(0, b"$VER: Rally 5.30 (4.8.2000)")])),
    copyright: PlainBytes::new(with_runs::<91>(&[(0, b"Copyright (c) Magnetic Fields (Software Design) Ltd., 1998,1999,2000. All rights reserved.")])),
    second_text: PlainByte::new(0x0),
    window_name: PlainBytes::new(with_runs::<8>(&[(0, b"Rally 5")])),
    class_name: PlainBytes::new(with_runs::<10>(&[(0, b"MF-SDG R5")])),
    _gap_4e008a: PlainBytes::new(with_runs::<22>(&[(2, b"c\x0f_\x00M\x10\x11")])),
}
    }
}

impl StartTexts {
    pub const START: usize = 0x4E0000;
    pub const LEN: usize = 0x94;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starttexts_starts_as_the_games() {
        let size = size_of::<StartTexts>();
        assert!(StartTexts::LEN <= size && size <= (StartTexts::LEN).next_multiple_of(16));
    }
}
