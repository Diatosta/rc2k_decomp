//! The strings' text buffer (`strings::text_from_ids` and the screens that build a line
//! there), out of RAL.EXE's image, typed. `#[repr(C)]` struct of a byte
//! buffer; `moved::MOVED` points the image's address at it.

use crate::session::Bytes;
#[cfg(test)]
use crate::strings::owner::strings;

/// The strings' text buffer (IDA 0x93E430 .. 0x93E830, the image's layout).
#[repr(C, align(16))]
#[derive(rc2k_derive::Cells)]
pub struct TextBuffer {
    /// 0x93E430: `text`: the shared text buffer (the string of an id is built here, then drawn).
    pub text: Bytes<1024>,
}

impl TextBuffer {
    pub const fn new() -> Self {
        TextBuffer {
            text: Bytes::new([0; 1024]),
        }
    }
}

impl TextBuffer {
    pub const START: usize = 0x93E430;
    pub const LEN: usize = 0x400;
    /// Where the byte IDA calls `START` is (`moved::MOVED`'s home).
    #[cfg(test)]
    pub fn home() -> usize {
        crate::pointers::game_address(&raw const strings().text.buffer) as usize
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn textbuffer_starts_as_the_games() {
        let size = size_of::<TextBuffer>();
        assert!(TextBuffer::LEN <= size && size <= (TextBuffer::LEN).next_multiple_of(16));
        let image = crate::moved::image_bytes(TextBuffer::START, TextBuffer::LEN);
        let mine = unsafe {
            std::slice::from_raw_parts(
                crate::pointers::host_at(TextBuffer::home()).cast_const(),
                TextBuffer::LEN,
            )
        };
        assert_eq!(mine, &image[..]);
    }
}
