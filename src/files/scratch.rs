//! The 64 KB scratch the game's loaders share (the packed copy of a file, the replay's
//! packing, the map's read buffer, the installer's pages, the PNG decoder's info), out of
//! RAL.EXE's image. Struct of a byte buffer.

use crate::memory::session::PlainBytes;

/// The shared 64 KB scratch.
#[repr(align(16))]
#[derive(rc2k_derive::Cells)]
pub struct Scratch {
    /// Scratch space (0x10000 bytes), used by whoever needs a buffer for a moment.
    pub buffer: PlainBytes<65536>,
}

impl Scratch {
    pub const fn new() -> Self {
        Scratch {
            buffer: PlainBytes::new([0; 65536]),
        }
    }
}

impl Scratch {
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
            crate::logfile::program_log::log(&format!("scratch: {what}: {count:#x} bytes"));
            return Err(crate::errors::BadInput("a read past the scratch"));
        }
        let mut bytes = vec![0u8; count as usize];
        let (got, ok) = crate::files::handle::read_slice(file, &mut bytes);
        self.buffer.write_at(0, &bytes[..got as usize], what)?;
        Ok((got, ok))
    }
}
