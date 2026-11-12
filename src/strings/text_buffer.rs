//! The strings' text buffer (`strings::text_from_ids` and the screens that build a line
//! there), out of RAL.EXE's image, typed. `#[repr(C)]` struct of a byte buffer.

use crate::session::{Bytes, PlainBytes};

/// The strings' text buffer (IDA 0x93E430 .. 0x93E830, the image's layout).
#[repr(C, align(16))]
#[derive(rc2k_derive::Cells)]
pub struct TextBuffer {
    /// 0x93E430: `text`: the shared text buffer (the string of an id is built here, then drawn).
    pub text: PlainBytes<1024>,
}

/// One of the game's shared text buffers, which texts are built in and drawn from: the text
/// buffer, or one of the screens' 0x100-byte line buffers. Reads and writes stay inside it.
#[derive(Clone, Copy)]
pub enum TextBuf {
    Text(&'static PlainBytes<1024>),
    Line(&'static Bytes<0x100>),
}

impl TextBuf {
    /// The byte at `at`, or a refusal past the buffer's end.
    pub fn byte(self, at: usize, what: &str) -> Result<u8, crate::errors::BadInput> {
        match self {
            TextBuf::Text(b) => b.read_at(at, 1, what),
            TextBuf::Line(b) => b.read_at(at, 1, what),
        }
        .map(|v| v[0])
    }

    /// `bytes` written from `at`, refused past the buffer's end.
    pub fn write_at(
        self,
        at: usize,
        bytes: &[u8],
        what: &str,
    ) -> Result<(), crate::errors::BadInput> {
        match self {
            TextBuf::Text(b) => b.write_at(at, bytes, what),
            TextBuf::Line(b) => b.write_at(at, bytes, what),
        }
    }
}

impl TextBuffer {
    pub const fn new() -> Self {
        TextBuffer {
            text: PlainBytes::new([0; 1024]),
        }
    }
}

impl TextBuffer {
    pub const START: usize = 0x93E430;
    pub const LEN: usize = 0x400;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn textbuffer_starts_as_the_games() {
        let size = size_of::<TextBuffer>();
        assert!(TextBuffer::LEN <= size && size <= (TextBuffer::LEN).next_multiple_of(16));
    }
}
