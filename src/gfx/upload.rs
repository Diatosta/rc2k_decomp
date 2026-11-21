//! The texture upload's surface description (the locked surface's pitch and pixels) and the
//! mode-trial switch. A surface's `Lock` fills the description; `d3d::texture_lock` and
//! `upload_texture_picture` read the pitch and the pixels.

use std::cell::Cell;

/// What a `Lock` reports of the surface.
pub struct SurfaceDesc {
    /// The locked surface's pitch.
    pub pitch: Cell<u32>,
    /// The locked surface's pixels.
    pub pixels: Cell<u32>,
}

pub struct Upload {
    pub description: SurfaceDesc,
    /// 1 while a screen mode is being tried.
    pub trying: Cell<u32>,
}

impl Upload {
    pub const fn new() -> Self {
        Upload {
            description: SurfaceDesc {
                pitch: Cell::new(0),
                pixels: Cell::new(0),
            },
            trying: Cell::new(0),
        }
    }
}
