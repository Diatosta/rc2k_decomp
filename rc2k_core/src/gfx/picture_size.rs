//! The picture buffer's size as the loaders set it (converted 2026-10-21 from
//! `gfx::load_picture` 0x411863, `gfx::read_texture_picture` 0x405356,
//! `gfx::show_picture_file` 0x41172B and `gfx::put_surface_pixel` 0x4065BB's place).
//!
//! A picture is read at a size the decoder may fill (800 x 600, or 256 x 256 for a
//! texture) with rows that follow one another, then given the screen's size with rows of
//! four bytes a pixel; a picture whose decoded width is not the screen's is rescaled.

/// The picture's width, height and row length (`dword_4F49A4` on).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PictureSize {
    pub width: u32,
    pub height: u32,
    pub row_bytes: u32,
}

impl PictureSize {
    pub fn dwords(&self) -> [u32; 3] {
        [self.width, self.height, self.row_bytes]
    }
}

/// The row length that says the rows follow one another.
pub const ROWS_FOLLOW: u32 = 0xFFFF_FFFD;

/// A picture as it is read: the most a screen's picture has.
pub const READ: PictureSize = PictureSize {
    width: 800,
    height: 600,
    row_bytes: ROWS_FOLLOW,
};

/// A texture's picture as it is read.
pub const TEXTURE: PictureSize = PictureSize {
    width: 0x100,
    height: 0x100,
    row_bytes: ROWS_FOLLOW,
};

/// The dwords a texture's picture is filled with first (-1, its whole 256 x 256).
pub const TEXTURE_FILLED: usize = 0x100 * 0x100;

/// The picture at the screen's size.
pub fn screen(width: u32, height: u32) -> PictureSize {
    PictureSize {
        width,
        height,
        row_bytes: width << 2,
    }
}

/// Whether a picture read is rescaled to the screen: its decoded width is not the
/// screen's.
pub fn rescaled(decoded_width: u32, screen_width: u32) -> bool {
    decoded_width != screen_width
}

/// The screen mode's flag of three pages (a third buffer).
pub const THREE_PAGES: u32 = 0x4_0000;

/// `gfx::show_picture_file`: the pages the picture is shown on.
pub fn pages(mode_flags: u32) -> u32 {
    if mode_flags & THREE_PAGES != 0 { 3 } else { 2 }
}

/// `gfx::put_surface_pixel`: a pixel's address and its bytes (2 for a surface of 16 bits a
/// pixel, else 4), all wrapping.
pub fn pixel_place(pixels: u32, pitch: u32, bits: u32, x: u32, y: u32) -> (u32, u32) {
    let row = pixels.wrapping_add(y.wrapping_mul(pitch));
    let wide = if bits == 16 { 2 } else { 4 };
    (row.wrapping_add(x.wrapping_mul(wide)), wide)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sizes() {
        assert_eq!(READ.dwords(), [800, 600, 0xFFFF_FFFD]);
        assert_eq!(screen(640, 480).row_bytes, 2560);
        assert!(rescaled(800, 640) && !rescaled(640, 640));
        assert_eq!((pages(0x4_0000), pages(0x2_0000)), (3, 2));
        assert_eq!(pixel_place(0x1000, 0x200, 16, 3, 2), (0x1406, 2));
        assert_eq!(pixel_place(0x1000, 0x200, 32, 3, 2), (0x140C, 4));
    }
}
