//! A frame's pixels in NV12, the layout hardware and software encoders take: a plane of luma,
//! one byte a pixel, then a plane of blue and red differences, two bytes for each 2 x 2 block.
//! The colours are BT.601 in studio range (luma 16..235, differences 16..240), as the video
//! render's conversion on the card makes them (`nv12_y.frag`, `nv12_uv.frag`).

/// The even size the planes have for a frame of `width` x `height`.
pub fn even_size(width: u32, height: u32) -> (u32, u32) {
    (width & !1, height & !1)
}

/// The bytes of a frame of `size` (both sides even) in NV12.
pub fn frame_bytes(size: (u32, u32)) -> usize {
    size.0 as usize * size.1 as usize * 3 / 2
}

fn luma(r: f32, g: f32, b: f32) -> f32 {
    16.0 + 65.481 * r + 128.553 * g + 24.966 * b
}

fn blue_difference(r: f32, g: f32, b: f32) -> f32 {
    128.0 - 37.797 * r - 74.203 * g + 112.0 * b
}

fn red_difference(r: f32, g: f32, b: f32) -> f32 {
    128.0 + 112.0 * r - 93.786 * g - 18.214 * b
}

/// The picture `bgra` (`width` x `height`, four bytes a pixel, blue first) as NV12 at the even
/// size, into `out` (resized to fit): the last column or row of an odd frame is left out.
pub fn from_bgra(bgra: &[u8], width: u32, height: u32, out: &mut Vec<u8>) {
    let size = even_size(width, height);
    let (w, h) = (size.0 as usize, size.1 as usize);
    out.clear();
    out.resize(frame_bytes(size), 0);
    let stride = width as usize * 4;
    let pixel = |x: usize, y: usize| {
        let at = y * stride + x * 4;
        let p = bgra.get(at..at + 3).unwrap_or(&[0, 0, 0]);
        [
            f32::from(p[2]) / 255.0,
            f32::from(p[1]) / 255.0,
            f32::from(p[0]) / 255.0,
        ]
    };
    let (luma_plane, chroma_plane) = out.split_at_mut(w * h);
    for y in 0..h {
        for x in 0..w {
            let [r, g, b] = pixel(x, y);
            luma_plane[y * w + x] = luma(r, g, b).round().clamp(0.0, 255.0) as u8;
        }
    }
    for y in 0..h / 2 {
        for x in 0..w / 2 {
            let mut mean = [0.0f32; 3];
            for (dx, dy) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
                let p = pixel(2 * x + dx, 2 * y + dy);
                for (m, v) in mean.iter_mut().zip(p) {
                    *m += v * 0.25;
                }
            }
            let at = (y * (w / 2) + x) * 2;
            chroma_plane[at] = blue_difference(mean[0], mean[1], mean[2])
                .round()
                .clamp(0.0, 255.0) as u8;
            chroma_plane[at + 1] = red_difference(mean[0], mean[1], mean[2])
                .round()
                .clamp(0.0, 255.0) as u8;
        }
    }
}

/// An NV12 frame of `size` as RGB, three bytes a pixel (for the numbered pictures when no video
/// encoder could start).
pub fn to_rgb(nv12: &[u8], size: (u32, u32)) -> Vec<u8> {
    let (w, h) = (size.0 as usize, size.1 as usize);
    let mut rgb = Vec::with_capacity(w * h * 3);
    for y in 0..h {
        for x in 0..w {
            let luma = f32::from(nv12[y * w + x]);
            let at = w * h + (y / 2) * w + (x / 2) * 2;
            let cb = f32::from(nv12[at]) - 128.0;
            let cr = f32::from(nv12[at + 1]) - 128.0;
            let scaled = 1.164 * (luma - 16.0);
            let r = scaled + 1.596 * cr;
            let g = scaled - 0.392 * cb - 0.813 * cr;
            let b = scaled + 2.017 * cb;
            rgb.extend([r, g, b].map(|v| v.round().clamp(0.0, 255.0) as u8));
        }
    }
    rgb
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn white_and_black_are_the_studio_range_ends() {
        let mut out = Vec::new();
        from_bgra(&[255; 16], 2, 2, &mut out);
        assert_eq!(out, [235, 235, 235, 235, 128, 128]);
        from_bgra(
            &[0, 0, 0, 255, 0, 0, 0, 255, 0, 0, 0, 255, 0, 0, 0, 255],
            2,
            2,
            &mut out,
        );
        assert_eq!(out, [16, 16, 16, 16, 128, 128]);
    }

    #[test]
    fn red_has_the_high_red_difference() {
        let mut out = Vec::new();
        let red = [0, 0, 255, 255].repeat(4);
        from_bgra(&red, 2, 2, &mut out);
        assert_eq!(out, [81, 81, 81, 81, 90, 240]);
    }

    #[test]
    fn an_odd_frame_loses_its_last_column_and_row() {
        let mut out = Vec::new();
        from_bgra(&[7; 3 * 3 * 4], 3, 3, &mut out);
        assert_eq!(out.len(), frame_bytes((2, 2)));
    }

    #[test]
    fn rgb_comes_back_close() {
        let mut out = Vec::new();
        let frame = [40, 120, 200, 255].repeat(4);
        from_bgra(&frame, 2, 2, &mut out);
        let rgb = to_rgb(&out, (2, 2));
        for (got, want) in rgb[..3].iter().zip([200u8, 120, 40]) {
            assert!(got.abs_diff(want) <= 3, "{got} against {want}");
        }
    }
}
