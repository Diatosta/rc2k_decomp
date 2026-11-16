//! The program's own texts: the version and the copyright.

use crate::memory::session::{PlainBytes, with_runs};

#[derive(rc2k_derive::Cells)]
pub struct StartTexts {
    /// "$VER: Rally 5.30 (4.8.2000)" (`system::read_command_line`, `install::check_at_start`).
    pub version: PlainBytes<28>,
    /// The copyright text, "Copyright (c) Magnetic Fields .." (`install::check_installed_version`
    /// looks for it in the program).
    pub copyright: PlainBytes<91>,
}

impl StartTexts {
    pub const fn new() -> Self {
        StartTexts {
            version: PlainBytes::new(with_runs::<28>(&[(0, b"$VER: Rally 5.30 (4.8.2000)")])),
            copyright: PlainBytes::new(with_runs::<91>(&[(
                0,
                b"Copyright (c) Magnetic Fields (Software Design) Ltd., 1998,1999,2000. All rights reserved.",
            )])),
        }
    }
}
