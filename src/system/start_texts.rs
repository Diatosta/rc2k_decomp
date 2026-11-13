//! The program's own texts (version, copyright, window and class names), out of RAL.EXE's
//! image. Struct of cells,
//! first values the image's (`tests`).

use crate::session::{PlainByte, PlainBytes, with_runs};

/// The version, copyright, window and class texts.
#[derive(rc2k_derive::Cells)]
pub struct StartTexts {
    /// "$VER: Rally 5.30 (4.8.2000)" (`system::read_command_line`, `install::check_at_start`).
    pub version: PlainBytes<28>,
    /// The copyright text, "Copyright (c) Magnetic Fields .." (`install::check_installed_version`
    /// looks for it in the program).
    pub copyright: PlainBytes<91>,
    /// The second text of the `-v` box (empty).
    pub second_text: PlainByte,
    /// The window's name, "Rally 5".
    pub window_name: PlainBytes<8>,
    /// The window class's name, "MF-SDG R5" (`system::find_running_game`).
    pub class_name: PlainBytes<10>,
}

impl StartTexts {
    pub const fn new() -> Self {
        StartTexts {
    version: PlainBytes::new(with_runs::<28>(&[(0, b"$VER: Rally 5.30 (4.8.2000)")])),
    copyright: PlainBytes::new(with_runs::<91>(&[(0, b"Copyright (c) Magnetic Fields (Software Design) Ltd., 1998,1999,2000. All rights reserved.")])),
    second_text: PlainByte::new(0x0),
    window_name: PlainBytes::new(with_runs::<8>(&[(0, b"Rally 5")])),
    class_name: PlainBytes::new(with_runs::<10>(&[(0, b"MF-SDG R5")])),
}
    }
}
