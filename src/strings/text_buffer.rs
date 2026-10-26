//! The strings' text buffer (`strings::text_from_ids` and the screens that build a line
//! there), out of RAL.EXE's image (phase 5), typed (phase 7). `#[repr(C)]` struct of a byte
//! buffer; `moved::MOVED` points the image's address at it.

use crate::session::Bytes;

/// The strings' text buffer (IDA 0x93E430 .. 0x93E830, the image's layout).
#[repr(C, align(16))]
pub struct TextBuffer {
    /// 0x93E430: `text`: the shared text buffer (the string of an id is built here, then drawn).
    pub text: Bytes<1024>,
}

pub static TEXT_BUFFER: TextBuffer = TextBuffer {
    text: Bytes::new([0; 1024]),
};

impl TextBuffer {
    pub const START: usize = 0x93E430;
    pub const LEN: usize = 0x400;
    /// Where the byte IDA calls `START` is now (`moved::MOVED`'s home).
    pub fn home() -> usize {
        crate::pointers::game_address(&TEXT_BUFFER as *const TextBuffer) as usize
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
                crate::pointers::host_at(TextBuffer::home()) as *const u8,
                TextBuffer::LEN,
            )
        };
        assert_eq!(mine, &image[..]);
    }
}
