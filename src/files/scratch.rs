//! The 64 KB scratch the game's loaders share (the packed copy of a file, the replay's
//! packing, the map's read buffer, the installer's pages, the PNG decoder's info), out of
//! RAL.EXE's image (phase 5), typed (phase 7). `#[repr(C)]` struct of a byte buffer;
//! `moved::MOVED` points the image's address at it.

use crate::session::Bytes;

/// The shared 64 KB scratch (IDA 0x93E8B0 .. 0x94E8B0, the image's layout).
#[repr(C, align(16))]
pub struct Scratch {
    /// 0x93E8B0: `g_pngInfo`: scratch space (0x10000 bytes), used by whoever needs a buffer for a moment.
    pub buffer: Bytes<65536>,
}

pub static SCRATCH: Scratch = Scratch {
    buffer: Bytes::new([0; 65536]),
};

impl Scratch {
    pub const START: usize = 0x93E8B0;
    pub const LEN: usize = 0x10000;

    /// `files::read` of `count` bytes from `file` into the scratch: (the bytes read, whether
    /// all of them were). More than the scratch holds is refused before anything is read
    /// (`what` names it).
    pub fn read_from(
        &self,
        file: u32,
        count: u32,
        what: &str,
    ) -> Result<(u32, bool), crate::errors::BadInput> {
        if count as usize > Self::LEN {
            crate::framework::log(&format!("scratch: {what}: {count:#x} bytes"));
            return Err(crate::errors::BadInput("a read past the scratch"));
        }
        let mut bytes = vec![0u8; count as usize];
        let (got, ok) = crate::files::handle::read_slice(file, &mut bytes);
        self.buffer.write_at(0, &bytes[..got as usize], what)?;
        Ok((got, ok))
    }
    /// Where the byte IDA calls `START` is now (`moved::MOVED`'s home).
    pub fn home() -> usize {
        crate::pointers::game_address(&SCRATCH as *const Scratch) as usize
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scratch_starts_as_the_games() {
        let size = size_of::<Scratch>();
        assert!(Scratch::LEN <= size && size <= (Scratch::LEN).next_multiple_of(16));
        let image = crate::moved::image_bytes(Scratch::START, Scratch::LEN);
        let mine = unsafe {
            std::slice::from_raw_parts(
                crate::pointers::host_at(Scratch::home()) as *const u8,
                Scratch::LEN,
            )
        };
        assert_eq!(mine, &image[..]);
    }
}
