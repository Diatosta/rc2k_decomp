//! The texture upload's surface description (DDSURFACEDESC) and the mode-trial switch, out
//! of RAL.EXE's image (phase 5; typed in phase 7, `notes/conversion.md`): IDA 0x5ABA18 ..
//! 0x5ABA8C. A surface's `Lock` fills the description; `d3d::texture_lock` and
//! `upload_texture_picture` hand its address on and read the pitch and the pixels.

use crate::session::Dword;

/// IDA's address of the block's first byte.
pub const START: usize = 0x5ABA18;
/// The block's length.
pub const LEN: usize = 0x5ABA8C - START;

/// A DirectDraw surface description (DDSURFACEDESC, 0x6C bytes), filled by a surface's
/// `Lock`; the game's code reads its fields by their DirectDraw offsets.
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct SurfaceDesc {
    /// `dwSize` (`dword_5ABA18`).
    pub size: Dword,
    /// `dwFlags`.
    pub flags: Dword,
    /// `dwHeight`.
    pub height: Dword,
    /// `dwWidth`.
    pub width: Dword,
    /// `lPitch` (`dword_5ABA28`): the locked surface's pitch.
    pub pitch: Dword,
    /// `dwBackBufferCount`, `dwMipMapCount`, `dwAlphaBitDepth`, `dwReserved`.
    pub counts: [Dword; 4],
    /// `lpSurface` (`dword_5ABA3C`): the locked surface's pixels.
    pub pixels: Dword,
    /// The four colour keys (`ddckCKDestOverlay`, `ddckCKDestBlt`, `ddckCKSrcOverlay`,
    /// `ddckCKSrcBlt`: two dwords each).
    pub colour_keys: [Dword; 8],
    /// `ddpfPixelFormat` (8 dwords).
    pub pixel_format: [Dword; 8],
    /// `ddsCaps`.
    pub caps: Dword,
}

/// The upload's variables.
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct Upload {
    /// `dword_5ABA18`: the description a `Lock` fills.
    pub description: SurfaceDesc,
    /// `dword_5ABA84`: nothing reads or writes it (a gap).
    pub _gap_6c: Dword,
    /// `dword_5ABA88`: 1 while a screen mode is tried (`try_screen_mode`).
    pub trying: Dword,
}

const fn zeros<const N: usize>() -> [Dword; N] {
    [const { Dword::new(0) }; N]
}

impl Upload {
    /// The upload's variables; the first values are the image's.
    pub const fn new() -> Self {
        Upload {
            description: SurfaceDesc {
                size: Dword::new(0),
                flags: Dword::new(0),
                height: Dword::new(0),
                width: Dword::new(0),
                pitch: Dword::new(0),
                counts: zeros(),
                pixels: Dword::new(0),
                colour_keys: zeros(),
                pixel_format: zeros(),
                caps: Dword::new(0),
            },
            _gap_6c: Dword::new(0),
            trying: Dword::new(0),
        }
    }
}

impl SurfaceDesc {
    /// Where the description is, for the surface's `Lock`.
    pub fn addr(&self) -> u32 {
        crate::pointers::game_address(std::ptr::from_ref::<SurfaceDesc>(self))
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
        crate::gfx::first_values_are_the_games(&gfx().card.upload, START, START + LEN);
    }

    #[test]
    fn moved_knows_the_block() {
        let card = &gfx().card;
        assert_eq!(offset_of!(SurfaceDesc, pitch), 0x10);
        assert_eq!(offset_of!(SurfaceDesc, pixels), 0x24);
        assert_eq!(offset_of!(Upload, trying), 0x5ABA88 - START);
        assert_eq!(
            crate::moved::place(START),
            card.upload.description.addr() as usize
        );
        assert_eq!(
            crate::moved::place(0x5ABA28),
            card.upload.description.pitch.addr() as usize
        );
        assert_eq!(
            crate::moved::place(0x5ABA3C),
            card.upload.description.pixels.addr() as usize
        );
        assert_eq!(
            crate::moved::place(0x5ABA88),
            card.upload.trying.addr() as usize
        );
    }
}
