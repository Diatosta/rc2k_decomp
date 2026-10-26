//! The audio thread's mixing buffer, out of RAL.EXE's image (phase 5), typed (phase 7).
//! `#[repr(C)]` struct of a byte buffer; `moved::MOVED` points the image's address at it.

use crate::session::Bytes;

/// The mixing buffer (IDA 0x952AF8 .. 0x9566F8, the image's layout).
#[repr(C, align(16))]
pub struct MixBuffer {
    /// Not in the image: the bytes that keep every field the alignment its IDA address has (the struct is 16-byte aligned).
    pub _lead: Bytes<8>,
    /// 0x952AF8: `g_mixBuffer`: the mixed samples of a turn (`audio::mix_channels`, `audio::write_mix`).
    pub samples: Bytes<15360>,
}

pub static MIX_BUFFER: MixBuffer = MixBuffer {
    _lead: Bytes::new([0; 8]),
    samples: Bytes::new([0; 15360]),
};

impl MixBuffer {
    pub const START: usize = 0x952AF8;
    pub const LEN: usize = 0x3C00;
    /// Where the byte IDA calls `START` is now (`moved::MOVED`'s home).
    pub fn home() -> usize {
        crate::pointers::host_number(&MIX_BUFFER as *const MixBuffer) + 8
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mixbuffer_starts_as_the_games() {
        let size = size_of::<MixBuffer>();
        assert!(8 + MixBuffer::LEN <= size && size <= (8 + MixBuffer::LEN).next_multiple_of(16));
        let image = crate::moved::image_bytes(MixBuffer::START, MixBuffer::LEN);
        let mine = unsafe {
            std::slice::from_raw_parts(
                crate::pointers::host_at(MixBuffer::home()) as *const u8,
                MixBuffer::LEN,
            )
        };
        assert_eq!(mine, &image[..]);
    }
}
