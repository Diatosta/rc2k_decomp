//! A camera's frame turned into RGBA: the pixel layouts the cameras SDL opens deliver (packed
//! and planar YUV, 24- and 32-bit RGB), read with the frame's own row pitch.
//!
//! The YUV layouts are read as the cameras send them: BT.601, the luma range 16 to 235 and
//! the chroma range 16 to 240. Every read is checked against the data's length; a frame that
//! is too short for its size and pitch gives nothing.

/// How a frame's bytes are laid out.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Layout {
    /// Three bytes a pixel: the bytes' places of red, green and blue.
    Rgb24 { r: usize, g: usize, b: usize },
    /// Four bytes a pixel (the fourth unused or alpha): the places of red, green and blue.
    Packed32 { r: usize, g: usize, b: usize },
    /// Y0 U Y1 V, two pixels in four bytes.
    Yuy2,
    /// Y0 V Y1 U.
    Yvyu,
    /// U Y0 V Y1.
    Uyvy,
    /// A plane of luma, then a plane of U and V interleaved, at half the size each way.
    Nv12,
    /// As `Nv12` with V before U.
    Nv21,
    /// A plane of luma, then the U plane, then the V plane, each chroma plane at half size.
    I420,
    /// As `I420` with the V plane first.
    Yv12,
}

fn clamp(v: i32) -> u8 {
    v.clamp(0, 255) as u8
}

/// RGBA of one pixel from its Y, U and V bytes.
fn rgba(y: u8, u: u8, v: u8) -> [u8; 4] {
    let c = 298 * (i32::from(y) - 16);
    let (d, e) = (i32::from(u) - 128, i32::from(v) - 128);
    [
        clamp((c + 409 * e + 128) >> 8),
        clamp((c - 100 * d - 208 * e + 128) >> 8),
        clamp((c + 516 * d + 128) >> 8),
        255,
    ]
}

/// The frame as RGBA, `width` x `height` pixels, rows `pitch` bytes apart in `data` (for the
/// planar layouts the pitch of the luma plane; the chroma planes follow it as SDL lays them
/// out). `None` when the size is nothing, the pitch is too small for a row or the data too
/// short.
pub fn to_rgba(
    layout: Layout,
    width: usize,
    height: usize,
    pitch: usize,
    data: &[u8],
) -> Option<Vec<u8>> {
    if width == 0 || height == 0 {
        return None;
    }
    let mut out = Vec::with_capacity(width.checked_mul(height)?.checked_mul(4)?);
    match layout {
        Layout::Rgb24 { r, g, b } => {
            let rows = rows(data, width.checked_mul(3)?, pitch, height)?;
            for row in rows {
                for p in row.as_chunks::<3>().0.iter() {
                    out.extend_from_slice(&[p[r], p[g], p[b], 255]);
                }
            }
        }
        Layout::Packed32 { r, g, b } => {
            for row in rows(data, width.checked_mul(4)?, pitch, height)? {
                for p in row.as_chunks::<4>().0.iter() {
                    out.extend_from_slice(&[p[r], p[g], p[b], 255]);
                }
            }
        }
        Layout::Yuy2 | Layout::Yvyu | Layout::Uyvy => {
            let (y0, u, y1, v) = match layout {
                Layout::Yuy2 => (0, 1, 2, 3),
                Layout::Yvyu => (0, 3, 2, 1),
                _ => (1, 0, 3, 2),
            };
            let pairs = width.div_ceil(2);
            for row in rows(data, pairs.checked_mul(4)?, pitch, height)? {
                for (n, q) in row.as_chunks::<4>().0.iter().enumerate() {
                    out.extend_from_slice(&rgba(q[y0], q[u], q[v]));
                    if n * 2 + 1 < width {
                        out.extend_from_slice(&rgba(q[y1], q[u], q[v]));
                    }
                }
            }
        }
        Layout::Nv12 | Layout::Nv21 => {
            let luma = rows(data, width, pitch, height)?;
            let half_rows = height.div_ceil(2);
            let chroma_at = pitch.checked_mul(height)?;
            let chroma = rows(
                data.get(chroma_at..)?,
                width.div_ceil(2).checked_mul(2)?,
                pitch,
                half_rows,
            )?;
            for (y, row) in luma.into_iter().enumerate() {
                let cr = &chroma[y / 2];
                for (x, &l) in row.iter().enumerate() {
                    let (a, b) = (cr[x / 2 * 2], cr[x / 2 * 2 + 1]);
                    let (u, v) = if layout == Layout::Nv12 {
                        (a, b)
                    } else {
                        (b, a)
                    };
                    out.extend_from_slice(&rgba(l, u, v));
                }
            }
        }
        Layout::I420 | Layout::Yv12 => {
            let luma = rows(data, width, pitch, height)?;
            let (cw, ch) = (width.div_ceil(2), height.div_ceil(2));
            let cp = pitch.div_ceil(2);
            let first_at = pitch.checked_mul(height)?;
            let second_at = first_at.checked_add(cp.checked_mul(ch)?)?;
            let first = rows(data.get(first_at..)?, cw, cp, ch)?;
            let second = rows(data.get(second_at..)?, cw, cp, ch)?;
            let (u_rows, v_rows) = if layout == Layout::I420 {
                (first, second)
            } else {
                (second, first)
            };
            for (y, row) in luma.into_iter().enumerate() {
                for (x, &l) in row.iter().enumerate() {
                    out.extend_from_slice(&rgba(l, u_rows[y / 2][x / 2], v_rows[y / 2][x / 2]));
                }
            }
        }
    }
    Some(out)
}

/// How many bytes a frame of `height` rows at `pitch` takes in `layout` (the chroma planes
/// included), for a caller that holds a pointer and must say how much of it is the frame.
pub fn frame_len(layout: Layout, pitch: usize, height: usize) -> Option<usize> {
    let luma = pitch.checked_mul(height)?;
    let half = height.div_ceil(2);
    match layout {
        Layout::Nv12 | Layout::Nv21 => luma.checked_add(pitch.checked_mul(half)?),
        Layout::I420 | Layout::Yv12 => {
            luma.checked_add(pitch.div_ceil(2).checked_mul(half)?.checked_mul(2)?)
        }
        _ => Some(luma),
    }
}

/// The `count` rows of `used` bytes that start `pitch` apart in `data`; none if a row would
/// be short or the pitch cannot hold one.
fn rows(data: &[u8], used: usize, pitch: usize, count: usize) -> Option<Vec<&[u8]>> {
    if pitch < used {
        return None;
    }
    let last_end = (count - 1).checked_mul(pitch)?.checked_add(used)?;
    if data.len() < last_end {
        return None;
    }
    Some(
        (0..count)
            .map(|n| &data[n * pitch..n * pitch + used])
            .collect(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The test picture: a colour that changes from pixel to pixel but slowly, so the
    /// chroma kept for two pixels does not matter.
    fn picture(w: usize, h: usize) -> Vec<[u8; 3]> {
        (0..w * h)
            .map(|n| {
                let (x, y) = (n % w, n / w);
                [
                    (40 + x * 40 / w) as u8,
                    (60 + y * 40 / h) as u8,
                    (200 - x * 30 / w) as u8,
                ]
            })
            .collect()
    }

    fn yuv(p: [u8; 3]) -> [u8; 3] {
        let (r, g, b) = (f64::from(p[0]), f64::from(p[1]), f64::from(p[2]));
        [
            (16.0 + 0.257 * r + 0.504 * g + 0.098 * b).round() as u8,
            (128.0 - 0.148 * r - 0.291 * g + 0.439 * b).round() as u8,
            (128.0 + 0.439 * r - 0.368 * g - 0.071 * b).round() as u8,
        ]
    }

    /// Every pixel within a few levels of the picture's.
    fn close(got: &[u8], want: &[[u8; 3]]) {
        assert_eq!(got.len(), want.len() * 4);
        for (n, w) in want.iter().enumerate() {
            let g = &got[n * 4..n * 4 + 4];
            assert_eq!(g[3], 255);
            for k in 0..3 {
                assert!(
                    (i32::from(g[k]) - i32::from(w[k])).abs() <= 6,
                    "pixel {n} channel {k}: {} against {}",
                    g[k],
                    w[k]
                );
            }
        }
    }

    const W: usize = 16;
    const H: usize = 8;
    const PAD: usize = 12;

    #[test]
    fn rgb_layouts_with_padded_rows() {
        let pic = picture(W, H);
        for (layout, order, bytes) in [
            (Layout::Rgb24 { r: 0, g: 1, b: 2 }, [0, 1, 2], 3),
            (Layout::Rgb24 { r: 2, g: 1, b: 0 }, [2, 1, 0], 3),
            (Layout::Packed32 { r: 0, g: 1, b: 2 }, [0, 1, 2], 4),
            (Layout::Packed32 { r: 1, g: 2, b: 3 }, [1, 2, 3], 4),
            (Layout::Packed32 { r: 2, g: 1, b: 0 }, [2, 1, 0], 4),
        ] {
            let pitch = W * bytes + PAD;
            let mut data = vec![0xAA; pitch * H];
            for (n, p) in pic.iter().enumerate() {
                let at = n / W * pitch + n % W * bytes;
                for k in 0..3 {
                    data[at + order[k]] = p[k];
                }
            }
            close(&to_rgba(layout, W, H, pitch, &data).unwrap(), &pic);
        }
    }

    /// The picture with the chroma of each pair (or block) of pixels averaged, as a camera
    /// sends it, so what is compared is what the layout can carry.
    #[test]
    fn packed_yuv_layouts_with_padded_rows() {
        let pic = picture(W, H);
        for layout in [Layout::Yuy2, Layout::Yvyu, Layout::Uyvy] {
            let pitch = W * 2 + PAD;
            let mut data = vec![0xAA; pitch * H];
            for y in 0..H {
                for x in (0..W).step_by(2) {
                    let (a, b) = (yuv(pic[y * W + x]), yuv(pic[y * W + x + 1]));
                    let u = ((u16::from(a[1]) + u16::from(b[1])) / 2) as u8;
                    let v = ((u16::from(a[2]) + u16::from(b[2])) / 2) as u8;
                    let q = match layout {
                        Layout::Yuy2 => [a[0], u, b[0], v],
                        Layout::Yvyu => [a[0], v, b[0], u],
                        _ => [u, a[0], v, b[0]],
                    };
                    let at = y * pitch + x * 2;
                    data[at..at + 4].copy_from_slice(&q);
                }
            }
            close(&to_rgba(layout, W, H, pitch, &data).unwrap(), &pic);
        }
    }

    #[test]
    fn planar_yuv_layouts_with_padded_rows() {
        let pic = picture(W, H);
        let pitch = W + PAD;
        let (cw, ch) = (W / 2, H / 2);
        let block = |cx: usize, cy: usize| {
            let mut sum = [0u32; 2];
            for (dx, dy) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
                let t = yuv(pic[(cy * 2 + dy) * W + cx * 2 + dx]);
                sum[0] += u32::from(t[1]);
                sum[1] += u32::from(t[2]);
            }
            [(sum[0] / 4) as u8, (sum[1] / 4) as u8]
        };
        for layout in [Layout::Nv12, Layout::Nv21, Layout::I420, Layout::Yv12] {
            let cp = pitch.div_ceil(2);
            let size = match layout {
                Layout::Nv12 | Layout::Nv21 => pitch * H + pitch * ch,
                _ => pitch * H + 2 * cp * ch,
            };
            let mut data = vec![0xAA; size];
            for (n, p) in pic.iter().enumerate() {
                data[n / W * pitch + n % W] = yuv(*p)[0];
            }
            for cy in 0..ch {
                for cx in 0..cw {
                    let [u, v] = block(cx, cy);
                    match layout {
                        Layout::Nv12 | Layout::Nv21 => {
                            let at = pitch * H + cy * pitch + cx * 2;
                            let (a, b) = if layout == Layout::Nv12 {
                                (u, v)
                            } else {
                                (v, u)
                            };
                            data[at] = a;
                            data[at + 1] = b;
                        }
                        _ => {
                            let (first, second) = if layout == Layout::I420 {
                                (u, v)
                            } else {
                                (v, u)
                            };
                            data[pitch * H + cy * cp + cx] = first;
                            data[pitch * H + cp * ch + cy * cp + cx] = second;
                        }
                    }
                }
            }
            close(&to_rgba(layout, W, H, pitch, &data).unwrap(), &pic);
        }
    }

    #[test]
    fn the_frames_lengths() {
        assert_eq!(frame_len(Layout::Yuy2, 40, 8), Some(320));
        assert_eq!(frame_len(Layout::Nv12, 28, 8), Some(28 * 8 + 28 * 4));
        assert_eq!(frame_len(Layout::I420, 28, 8), Some(28 * 8 + 2 * 14 * 4));
        assert_eq!(frame_len(Layout::Yuy2, usize::MAX, 8), None);
    }

    #[test]
    fn a_short_frame_or_pitch_gives_nothing() {
        let data = vec![0u8; 100];
        for layout in [Layout::Yuy2, Layout::Nv12, Layout::I420] {
            assert_eq!(to_rgba(layout, W, H, W * 2, &data), None);
        }
        let full = vec![0u8; W * H * 4];
        assert!(to_rgba(Layout::Packed32 { r: 0, g: 1, b: 2 }, W, H, W * 4, &full).is_some());
        assert_eq!(
            to_rgba(
                Layout::Packed32 { r: 0, g: 1, b: 2 },
                W,
                H,
                W * 4 - 1,
                &full
            ),
            None
        );
        assert_eq!(to_rgba(Layout::Yuy2, 0, H, 4, &full), None);
    }

    #[test]
    fn black_and_white_levels() {
        let black = to_rgba(Layout::Yuy2, 2, 1, 4, &[16, 128, 16, 128]).unwrap();
        assert_eq!(black[..3], [0, 0, 0]);
        let white = to_rgba(Layout::Yuy2, 2, 1, 4, &[235, 128, 235, 128]).unwrap();
        assert_eq!(white[..3], [255, 255, 255]);
    }
}
