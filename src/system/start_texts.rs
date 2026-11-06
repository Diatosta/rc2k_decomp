//! The program's own texts (version, copyright, window and class names), out of RAL.EXE's
//! image (phase 5), typed (phase 7). `#[repr(C)]` struct of cells with the image's layout,
//! first values the image's (`tests`); `moved::MOVED` points the image's addresses at it.

use crate::session::{Byte, Bytes, with_runs};

/// The version, copyright, window and class texts (IDA 0x4E0000 .. 0x4E0094, the image's layout).
#[repr(C, align(16))]
#[derive(rc2k_derive::Cells)]
pub struct StartTexts {
    /// 0x4E0000: `aVerRally530482`: "$VER: Rally 5.30 (4.8.2000)" (`system::read_command_line`, `install::check_at_start`).
    pub version: Bytes<28>,
    /// 0x4E001C: `byte_4E001C`: the copyright text, "Copyright (c) Magnetic Fields .." (`install::check_installed_version` looks for it in the program).
    pub copyright: Bytes<91>,
    /// 0x4E0077: `byte_4E0077`: the second text of the `-v` box (empty).
    pub second_text: Byte,
    /// 0x4E0078: `WindowName`: the window's name, "Rally 5".
    pub window_name: Bytes<8>,
    /// 0x4E0080: `ClassName`: the window class's name, "MF-SDG R5" (`system::find_running_game`).
    pub class_name: Bytes<10>,
    /// 0x4E008A: Not named: 10 bytes (a gap, or data nothing reads here).
    pub _gap_4e008a: Bytes<22>,
}

impl StartTexts {
    pub const fn new() -> Self {
        StartTexts {
    version: Bytes::new(with_runs::<28>(&[(0, b"$VER: Rally 5.30 (4.8.2000)")])),
    copyright: Bytes::new(with_runs::<91>(&[(0, b"Copyright (c) Magnetic Fields (Software Design) Ltd., 1998,1999,2000. All rights reserved.")])),
    second_text: Byte::new(0x0),
    window_name: Bytes::new(with_runs::<8>(&[(0, b"Rally 5")])),
    class_name: Bytes::new(with_runs::<10>(&[(0, b"MF-SDG R5")])),
    _gap_4e008a: Bytes::new(with_runs::<22>(&[(2, b"c\x0f_\x00M\x10\x11")])),
}
    }
}

impl StartTexts {
    pub const START: usize = 0x4E0000;
    pub const LEN: usize = 0x94;
    /// Where the byte IDA calls `START` is now (`moved::MOVED`'s home).
    #[cfg(test)]
    pub fn home() -> usize {
        crate::pointers::game_address(&raw const crate::system::owner::system().start.texts)
            as usize
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starttexts_starts_as_the_games() {
        let size = size_of::<StartTexts>();
        assert!(StartTexts::LEN <= size && size <= (StartTexts::LEN).next_multiple_of(16));
        let image = crate::moved::image_bytes(StartTexts::START, StartTexts::LEN);
        let mine = unsafe {
            std::slice::from_raw_parts(
                crate::pointers::host_at(StartTexts::home()).cast_const(),
                StartTexts::LEN,
            )
        };
        assert_eq!(mine, &image[..]);
    }
}
