//! The texture upload's surface description (DDSURFACEDESC) and the mode-trial switch, out
//! of RAL.EXE's image: IDA 0x5ABA18 ..
//! 0x5ABA8C. A surface's `Lock` fills the description; `d3d::texture_lock` and
//! `upload_texture_picture` hand its address on and read the pitch and the pixels.

use crate::session::PlainDword;

/// A DirectDraw surface description (DDSURFACEDESC, 0x6C bytes), filled by a surface's
/// `Lock`; the game's code reads its fields by their DirectDraw offsets.
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct SurfaceDesc {
    /// `dwSize` (`Upload.description.size`).
    pub size: PlainDword,
    /// `dwFlags`.
    pub flags: PlainDword,
    /// `dwHeight`.
    pub height: PlainDword,
    /// `dwWidth`.
    pub width: PlainDword,
    /// `lPitch` (`Upload.description.pitch`): the locked surface's pitch.
    pub pitch: PlainDword,
    /// `dwBackBufferCount`, `dwMipMapCount`, `dwAlphaBitDepth`, `dwReserved`.
    pub counts: [PlainDword; 4],
    /// `lpSurface` (`Upload.description.pixels`): the locked surface's pixels.
    pub pixels: PlainDword,
    /// The four colour keys (`ddckCKDestOverlay`, `ddckCKDestBlt`, `ddckCKSrcOverlay`,
    /// `ddckCKSrcBlt`: two dwords each).
    pub colour_keys: [PlainDword; 8],
    /// `ddpfPixelFormat` (8 dwords).
    pub pixel_format: [PlainDword; 8],
    /// `ddsCaps`.
    pub caps: PlainDword,
}

/// The upload's variables.
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct Upload {
    /// `Upload.description.size`: the description a `Lock` fills.
    pub description: SurfaceDesc,
    /// `dword_5ABA84`: nothing reads or writes it (a gap).
    pub _gap_6c: PlainDword,
    /// `dword_5ABA88`: 1 while a screen mode is tried (`try_screen_mode`).
    pub trying: PlainDword,
}

const fn zeros<const N: usize>() -> [PlainDword; N] {
    [const { PlainDword::new(0) }; N]
}

impl Upload {
    /// The upload's variables; the first values are the image's.
    pub const fn new() -> Self {
        Upload {
            description: SurfaceDesc {
                size: PlainDword::new(0),
                flags: PlainDword::new(0),
                height: PlainDword::new(0),
                width: PlainDword::new(0),
                pitch: PlainDword::new(0),
                counts: zeros(),
                pixels: PlainDword::new(0),
                colour_keys: zeros(),
                pixel_format: zeros(),
                caps: PlainDword::new(0),
            },
            _gap_6c: PlainDword::new(0),
            trying: PlainDword::new(0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gfx::owner::gfx;
    use core::mem::{offset_of, size_of};

    #[test]
    fn first_values_are_the_games() {
        assert_eq!(size_of::<SurfaceDesc>(), 0x6C);
        crate::test_support::first_values::assert_block_starts_as_the_games(&gfx().card.upload);
    }

    #[test]
    fn moved_knows_the_block() {
        assert_eq!(offset_of!(SurfaceDesc, pitch), 0x10);
        assert_eq!(offset_of!(SurfaceDesc, pixels), 0x24);
        assert_eq!(offset_of!(Upload, trying), 0x70);
    }
}
