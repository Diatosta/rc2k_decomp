//! The icon upload's kept record: where a car icon goes on its texture and where it is on the sheet.

const STORED_LEN: usize = std::mem::size_of::<IconUpload>();

#[repr(C, packed)]
#[derive(Clone, Copy, Debug)]
pub struct IconUpload {
    pub texture: u32,
    /// Where on the texture the icon goes.
    pub target_x: u32,
    pub target_y: u32,
    /// The icon's piece of the sheet: x, y, width, height (`blit_components`' rectangle).
    pub piece: [u32; 4],
}

const _: () = assert!(STORED_LEN == 0x1C);

/// What the upload keeps while it works: a record as bytes (it lies at an address that is no
/// multiple of 4): `get` and `set` turn them to and from an `IconUpload`.
#[repr(transparent)]
#[derive(rc2k_derive::Cells)]
pub struct IconUploadCell(crate::memory::session::PlainBytes<STORED_LEN>);

impl IconUploadCell {
    pub fn get(&self) -> IconUpload {
        let bytes = self.0.read();
        let word =
            |n: usize| u32::from_le_bytes([bytes[n], bytes[n + 1], bytes[n + 2], bytes[n + 3]]);
        IconUpload {
            texture: word(0),
            target_x: word(4),
            target_y: word(8),
            piece: [word(12), word(16), word(20), word(24)],
        }
    }

    pub fn set(&self, upload: IconUpload) {
        let (texture, target_x, target_y, piece) = (
            upload.texture,
            upload.target_x,
            upload.target_y,
            upload.piece,
        );
        let mut bytes = [0u8; STORED_LEN];
        for (n, value) in [texture, target_x, target_y]
            .into_iter()
            .chain(piece)
            .enumerate()
        {
            bytes[4 * n..4 * n + 4].copy_from_slice(&value.to_le_bytes());
        }
        self.0.write(&bytes);
    }
}

impl IconUploadCell {
    pub const fn new() -> Self {
        IconUploadCell(crate::memory::session::PlainBytes::new([0; STORED_LEN]))
    }
}
