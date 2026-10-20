//! PCX pictures (converted 2026-10-20 from `gfx::lay_out_pcx` 0x411A82, `read_pcx_8bit`
//! 0x411C6B, `read_pcx_24bit` 0x411D57).
//!
//! A header of 0x80 bytes (+4 .. +0xA left, top, right, bottom as words; +0x41 the planes,
//! 1 or 3; +0x42 the bytes of one plane of one row), then the rows, run-length coded: a byte
//! of 0xC0 or more is a count (less 0xC0) for the byte after it, any other byte stands for
//! itself. With one plane a byte is a palette number, and the palette (0x300 bytes) ends
//! the file. Pictures are decoded to 32-bit pixels.

/// The header's length.
pub const HEADER_LEN: usize = 0x80;
/// A screen's width of pixels, for the stride setting 0xFFFFFFFF.
const SCREEN_WIDTH: u32 = 800;

/// What the game asks of a decoding.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Target {
    /// The picture buffer's size (`g_pictureWidth`, `g_pictureHeight`).
    pub width: u32,
    pub height: u32,
    /// The widest and highest picture taken.
    pub width_limit: u32,
    pub height_limit: u32,
    /// `g_pictureFlags`: bit 8 for 8-bit pixels.
    pub flags: u32,
    /// `g_pictureRowBytes`: 0xFFFFFFFF a screen's width of pixels, 0xFFFFFFFE the buffer's
    /// width, 0xFFFFFFFD the decoded row itself, else that many bytes.
    pub row_bytes: u32,
}

/// How a picture is decoded (the game's variables at 0x5195D4 .., 0x519124 ..).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct Layout {
    pub planes: u32,
    pub line_bytes: u32,
    pub file_width: u32,
    pub file_height: u32,
    /// Cut to the buffer and the limits.
    pub width: u32,
    pub height: u32,
    pub pixel_bits: u32,
    pub pixel_bytes: u32,
    pub row_bytes: u32,
    pub row_dwords: u32,
    /// From one decoded row to the next.
    pub stride: u32,
    /// The stride less a row's bytes.
    pub row_gap: u32,
}

fn word(header: &[u8; HEADER_LEN], at: usize) -> u16 {
    u16::from_le_bytes([header[at], header[at + 1]])
}

/// The layout a header makes (`lay_out_pcx`).
pub fn lay_out(header: &[u8; HEADER_LEN], target: &Target) -> Layout {
    let planes = header[0x41] as u32;
    let line_bytes = word(header, 0x42) as u32;
    let file_width = word(header, 8)
        .wrapping_sub(word(header, 4))
        .wrapping_add(1) as u32;
    let width = file_width.min(target.width).min(target.width_limit);
    let file_height = word(header, 0xA)
        .wrapping_sub(word(header, 6))
        .wrapping_add(1) as u32;
    let height = file_height.min(target.height).min(target.height_limit);
    let pixel_bits = if target.flags & 0x100 != 0 { 8 } else { 32 };
    let pixel_bytes = pixel_bits >> 3;
    let row_bytes = pixel_bytes * width;
    // Three compares one after the other, each on what the one before left.
    let mut stride = target.row_bytes;
    if stride == 0xFFFF_FFFF {
        stride = pixel_bytes * SCREEN_WIDTH;
    }
    if stride == 0xFFFF_FFFE {
        stride = target.width.wrapping_mul(pixel_bytes);
    }
    if stride == 0xFFFF_FFFD {
        stride = row_bytes;
    }
    Layout {
        planes,
        line_bytes,
        file_width,
        file_height,
        width,
        height,
        pixel_bits,
        pixel_bytes,
        row_bytes,
        row_dwords: row_bytes >> 2,
        stride,
        row_gap: stride.wrapping_sub(row_bytes),
    }
}

/// The rows of a one-plane picture through `palette` (`read_pcx_8bit`). `next` reads the
/// file's next byte as the game's stream does: given what eax held, the eax it leaves (at
/// the end of the file, what it was given). `put` takes a pixel at a byte offset from the
/// first row's start (wrapping). A run goes on across the end of a row; a count of 0 is a
/// run of 2^32. `None` for a picture without rows (the original counts round).
pub fn decode_8bit(
    layout: &Layout,
    palette: &[u32; 256],
    mut next: impl FnMut(u32) -> u32,
    mut put: impl FnMut(u32, u32),
) -> Option<()> {
    if layout.height == 0 {
        return None;
    }
    let mut to = 0u32;
    // The pixel a run repeats, and how much of the run is left.
    let (mut pixel, mut run) = (0u32, 0u32);
    for _ in 0..layout.height {
        let mut column = 0u32;
        loop {
            if run == 0 {
                let mut byte = next(pixel) as u8;
                run = 1;
                if byte >= 0xC0 {
                    run = byte as u32 - 0xC0;
                    byte = next(byte as u32) as u8;
                }
                pixel = palette[byte as usize];
            }
            if column < layout.width {
                put(to, pixel);
                to = to.wrapping_add(4);
            }
            run = run.wrapping_sub(1);
            column = column.wrapping_add(1);
            if column >= layout.line_bytes {
                break;
            }
        }
        to = to.wrapping_add(layout.row_gap);
    }
    Some(())
}

/// The rows of a picture of planes, 8 bits each (`read_pcx_24bit`): plane n's bytes go to
/// byte `planes - 1 - n` of the pixels; then every pixel's fourth byte is 0. `put` takes a
/// byte at an offset from the first row's start (wrapping). A run goes on across the ends
/// of planes and rows, but the byte it repeats is 0 from the end of a row on. `None` for a
/// picture without rows or columns (the original counts round).
pub fn decode_planes(
    layout: &Layout,
    mut next: impl FnMut(u32) -> u32,
    mut put: impl FnMut(u32, u8),
) -> Option<()> {
    if layout.width == 0 || layout.height == 0 {
        return None;
    }
    let mut to = 0u32;
    // eax as the original keeps it (al the byte a run repeats), and what is left of the
    // run, counted down before it is looked at, as a signed number.
    let (mut eax, mut run) = (0u32, 0i32);
    for _ in 0..layout.height {
        to = to.wrapping_add(layout.planes);
        let mut plane = 0u32;
        loop {
            to = to.wrapping_sub(1);
            let mut column = 0u32;
            loop {
                run = run.wrapping_sub(1);
                if run <= 0 {
                    eax = next(eax);
                    run = 1;
                    if eax as u8 >= 0xC0 {
                        run = (eax as u8 - 0xC0) as i32;
                        eax = next(eax);
                    }
                }
                if column < layout.width {
                    put(to.wrapping_add(column.wrapping_mul(4)), eax as u8);
                }
                column = column.wrapping_add(1);
                if column >= layout.line_bytes {
                    break;
                }
            }
            plane = plane.wrapping_add(1);
            if plane >= layout.planes {
                break;
            }
        }
        eax = 0;
        for pixel in (1..=layout.width).rev() {
            put(to.wrapping_add(pixel.wrapping_mul(4)).wrapping_sub(1), 0);
        }
        to = to.wrapping_add(layout.stride);
    }
    Some(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn header(width: u16, height: u16, planes: u8, line_bytes: u16) -> [u8; HEADER_LEN] {
        let mut h = [0u8; HEADER_LEN];
        h[8..10].copy_from_slice(&(width - 1).to_le_bytes());
        h[0xA..0xC].copy_from_slice(&(height - 1).to_le_bytes());
        h[0x41] = planes;
        h[0x42..0x44].copy_from_slice(&line_bytes.to_le_bytes());
        h
    }

    const TARGET: Target = Target {
        width: 640,
        height: 480,
        width_limit: 1000,
        height_limit: 1000,
        flags: 0,
        row_bytes: 0xFFFF_FFFD,
    };

    #[test]
    fn layouts() {
        let l = lay_out(&header(4, 2, 1, 4), &TARGET);
        assert_eq!(
            (l.width, l.height, l.pixel_bits, l.stride, l.row_gap),
            (4, 2, 32, 16, 0)
        );
        let l = lay_out(
            &header(900, 2, 1, 900),
            &Target {
                row_bytes: 0xFFFF_FFFF,
                ..TARGET
            },
        );
        assert_eq!((l.file_width, l.width, l.stride), (900, 640, 3200));
        let l = lay_out(
            &header(4, 2, 1, 4),
            &Target {
                row_bytes: 0xFFFF_FFFE,
                flags: 0x100,
                ..TARGET
            },
        );
        assert_eq!((l.pixel_bytes, l.stride), (1, 640));
    }

    #[test]
    fn decodes_8bit() {
        let l = lay_out(&header(3, 2, 1, 4), &TARGET);
        let mut palette = [0u32; 256];
        for (n, p) in palette.iter_mut().enumerate() {
            *p = n as u32 * 0x10;
        }
        // Rows of 3 pixels in 4 bytes: 1, then a run of four 2s (the row's last two
        // pixels, its hidden fourth byte and row 2's first pixel), then 6, 7.
        let file = [1u8, 0xC4, 2, 6, 7];
        let mut at = 0;
        let mut out = vec![0u32; 6];
        decode_8bit(
            &l,
            &palette,
            |eax| match file.get(at) {
                Some(&b) => {
                    at += 1;
                    b as u32
                }
                None => eax,
            },
            |to, p| out[to as usize / 4] = p,
        )
        .unwrap();
        assert_eq!(out, [0x10, 0x20, 0x20, 0x20, 0x60, 0x70]);
    }

    #[test]
    fn decodes_planes() {
        let l = lay_out(&header(2, 1, 3, 2), &TARGET);
        // Planes red, green, blue of one row of two pixels.
        let file = [1u8, 2, 3, 4, 5, 6];
        let mut at = 0;
        let mut out = vec![0xAAu8; 8];
        decode_planes(
            &l,
            |eax| match file.get(at) {
                Some(&b) => {
                    at += 1;
                    b as u32
                }
                None => eax,
            },
            |to, b| out[to as usize] = b,
        )
        .unwrap();
        assert_eq!(out, [5, 3, 1, 0, 6, 4, 2, 0]);
        assert_eq!(decode_planes(&Layout::default(), |e| e, |_, _| ()), None);
    }
}
