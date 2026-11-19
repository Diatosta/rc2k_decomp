//! A rectangle of the drawing surface drawn at its own size and then enlarged in place: for
//! text the game's fonts draw only at one pixel size, over a screen larger than the 800 x 600 the
//! fonts and the HUD's pictures are laid out for. The HUD's pictures grow with the screen; this
//! grows the drawn text to match, with bilinear filtering, so its edges stay smooth.

use crate::gfx::rows::Rows;

/// A pixel as red, green and blue (0 .. 255) from its bytes (16 bits: 5-6-5; 32 bits:
/// 0x00RRGGBB).
fn unpack(bytes: &[u8]) -> [f32; 3] {
    if bytes.len() == 2 {
        let p = u16::from_le_bytes([bytes[0], bytes[1]]) as u32;
        let (r, g, b) = (p >> 11 & 0x1F, p >> 5 & 0x3F, p & 0x1F);
        [
            (r << 3 | r >> 2) as f32,
            (g << 2 | g >> 4) as f32,
            (b << 3 | b >> 2) as f32,
        ]
    } else {
        [bytes[2] as f32, bytes[1] as f32, bytes[0] as f32]
    }
}

/// The bytes of a pixel of `rgb` (rounded) in a surface of `bytes` bytes a pixel.
fn pack(rgb: [f32; 3], bytes: usize, out: &mut [u8]) {
    let [r, g, b] = rgb.map(|c| (c + 0.5).clamp(0.0, 255.0) as u32);
    if bytes == 2 {
        let p = (r >> 3) << 11 | (g >> 2) << 5 | b >> 3;
        out.copy_from_slice(&(p as u16).to_le_bytes());
    } else {
        out.copy_from_slice(&(r << 16 | g << 8 | b).to_le_bytes());
    }
}

/// The pixels of a `width` x `height` picture enlarged to `to_width` x `to_height`, bilinear, the
/// pixels as `unpack` reads them, `bytes` a pixel.
pub fn enlarged(
    pixels: &[u8],
    bytes: usize,
    (width, height): (usize, usize),
    (to_width, to_height): (usize, usize),
) -> Vec<u8> {
    use rayon::prelude::*;
    let mut out = vec![0u8; to_width * to_height * bytes];
    // The source read once, and each output column's two source columns and the weight between.
    let source: Vec<[f32; 3]> = pixels.chunks_exact(bytes).map(unpack).collect();
    let columns: Vec<(usize, usize, f32)> = (0..to_width)
        .map(|x| {
            let sx = ((x as f32 + 0.5) * width as f32 / to_width as f32 - 0.5)
                .clamp(0.0, (width - 1) as f32);
            let x0 = sx as usize;
            (x0, (x0 + 1).min(width - 1), sx.fract())
        })
        .collect();
    // Each output row is made on its own: the rows are shared out over the threads.
    out.par_chunks_mut(to_width * bytes)
        .enumerate()
        .for_each(|(y, row)| {
            // the source place of the output pixel's centre, less half a pixel
            let sy = ((y as f32 + 0.5) * height as f32 / to_height as f32 - 0.5)
                .clamp(0.0, (height - 1) as f32);
            let (y0, fy) = (sy as usize, sy.fract());
            let y1 = (y0 + 1).min(height - 1);
            let (upper, lower) = (
                &source[y0 * width..][..width],
                &source[y1 * width..][..width],
            );
            for (pixel, &(x0, x1, fx)) in row.chunks_exact_mut(bytes).zip(&columns) {
                let (a, b, c, d) = (upper[x0], upper[x1], lower[x0], lower[x1]);
                let mut rgb = [0f32; 3];
                for k in 0..3 {
                    let top = a[k] + (b[k] - a[k]) * fx;
                    let bottom = c[k] + (d[k] - c[k]) * fx;
                    rgb[k] = top + (bottom - top) * fy;
                }
                pack(rgb, bytes, pixel);
            }
        });
    out
}

/// The rectangle (x, y, width, height) `from` of the surface, enlarged to `to_size` and written
/// with its top left at `to`. False when a rectangle is off the surface or the surface is
/// neither 16 nor 32 bits. The surface must be locked.
pub fn enlarge(
    gfx: &crate::gfx::owner::Gfx,
    from: [i32; 4],
    to: (i32, i32),
    to_size: (i32, i32),
) -> bool {
    let surface = crate::gfx::surface::surface(gfx);
    let bytes = match surface.bits_per_pixel {
        16 => 2,
        32 => 4,
        _ => return false,
    };
    let Ok(rows) = Rows::surface(gfx) else {
        return false;
    };
    let [x, y, w, h] = from;
    let (tx, ty) = to;
    let (tw, th) = to_size;
    if [x, y, tx, ty].iter().any(|&v| v < 0) || w <= 0 || h <= 0 || tw <= 0 || th <= 0 {
        return false;
    }
    let clip = crate::gfx::surface::clip(gfx);
    if tx + tw - 1 > clip.right || ty + th - 1 > clip.bottom {
        return false;
    }
    let mut pixels = Vec::with_capacity(w as usize * h as usize * bytes);
    for row in 0..h {
        match rows.read(
            (y + row) as u32,
            x as u32 * bytes as u32,
            w as usize * bytes,
            "a region",
        ) {
            Ok(line) => pixels.extend_from_slice(&line),
            Err(_) => return false,
        }
    }
    let big = enlarged(
        &pixels,
        bytes,
        (w as usize, h as usize),
        (tw as usize, th as usize),
    );
    for row in 0..th {
        let line = &big[row as usize * tw as usize * bytes..][..tw as usize * bytes];
        if rows
            .write(
                (ty + row) as u32,
                tx as u32 * bytes as u32,
                line,
                "a region",
            )
            .is_err()
        {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_flat_picture_stays_flat_and_an_edge_is_graded() {
        let flat = enlarged(&[0x20, 0x20, 0x20, 0].repeat(16), 4, (4, 4), (10, 10));
        assert!(flat.chunks(4).all(|p| p == [0x20, 0x20, 0x20, 0]));
        // two columns, black and white (32 bits): the enlarged middle is grey
        let mut two = Vec::new();
        for _ in 0..2 {
            two.extend_from_slice(&[0, 0, 0, 0, 255, 255, 255, 0]);
        }
        let big = enlarged(&two, 4, (2, 2), (8, 2));
        let reds: Vec<u8> = (0..8).map(|x| big[x * 4 + 2]).collect();
        assert_eq!(reds[0], 0);
        assert_eq!(reds[7], 255);
        assert!(reds.windows(2).all(|w| w[0] <= w[1]));
        assert!(reds[3] > 0 && reds[3] < 255);
    }

    #[test]
    fn sixteen_bit_pixels_round_trip() {
        for p in [0u16, 0xFFFF, 0x1234, 0xF81F] {
            let mut out = [0u8; 2];
            pack(unpack(&p.to_le_bytes()), 2, &mut out);
            assert_eq!(u16::from_le_bytes(out), p);
        }
    }
}
