//! The texture upload's surface description (DDSURFACEDESC) and the mode-trial switch. A surface's
//! `Lock` fills the description; `d3d::texture_lock` and `upload_texture_picture` hand its address
//! on and read the pitch and the pixels.

use crate::session::PlainDword;

/// A DirectDraw surface description (DDSURFACEDESC, 0x6C bytes), filled by a surface's `Lock`; the
/// game's code reads its fields by their DirectDraw offsets.
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

#[derive(rc2k_derive::Cells)]
pub struct Upload {
    /// `Upload.description.size`: the description a `Lock` fills.
    pub description: SurfaceDesc,
    /// 1 while a screen mode is tried (`try_screen_mode`).
    pub trying: PlainDword,
}

const fn zeros<const N: usize>() -> [PlainDword; N] {
    [const { PlainDword::new(0) }; N]
}

impl Upload {
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
            trying: PlainDword::new(0),
        }
    }
}
