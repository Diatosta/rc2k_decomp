//! The program's own texts: the version and the copyright.

use crate::memory::plain_buffer::PlainBuffer;
use crate::memory::session::with_runs;

pub struct StartTexts {
    /// "$VER: Rally 5.30 (4.8.2000)" (`system::read_command_line`, `install::check_at_start`).
    pub version: PlainBuffer<28>,
    /// The copyright text, "Copyright (c) Magnetic Fields .." (`install::check_installed_version`
    /// looks for it in the program).
    pub copyright: PlainBuffer<91>,
}

impl StartTexts {
    pub const fn new() -> Self {
        StartTexts {
            version: PlainBuffer::new(with_runs::<28>(&[(0, b"$VER: Rally 5.30 (4.8.2000)")])),
            copyright: PlainBuffer::new(with_runs::<91>(&[(
                0,
                b"Copyright (c) Magnetic Fields (Software Design) Ltd., 1998,1999,2000. All rights reserved.",
            )])),
        }
    }
}
