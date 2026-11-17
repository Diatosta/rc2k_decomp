//! The wash behind a menu's frame: the window's space beside the centred frame shows the menu's
//! background picture, scaled to cover the whole window, strongly blurred and darkened.
//!
//! The wash is made small (a sixteenth of the window's size on each side) and meant to be
//! magnified with a linear filter: that magnification is part of the blur, and it keeps the
//! per-change cost low.

/// How many window pixels one pixel of the wash covers on each side.
pub const SHRINK: u32 = 16;
/// The box blur's radius in the wash's pixels (applied three times, which approaches a gaussian
/// blur of about 3 % of the window's width).
const BLUR_RADIUS: usize = 3;
/// The brightness left, in 256ths.
const BRIGHTNESS: u32 = 192;

/// The wash picture: 32-bit pixels, four bytes each, blue first, as the frame's.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Wash {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
}

/// The wash of `source` (`source_size` pixels of four bytes, rows with no gaps) for a window of
/// `window` pixels. The picture keeps its shape and covers the window (the part outside is
/// cropped, centred). `None` for a picture or window with no size, or a short picture.
pub fn make(source: &[u8], source_size: (u32, u32), window: (u32, u32)) -> Option<Wash> {
    let (sw, sh) = (source_size.0 as usize, source_size.1 as usize);
    if sw == 0 || sh == 0 || window.0 == 0 || window.1 == 0 || source.len() < sw * sh * 4 {
        return None;
    }
    let width = window.0.div_ceil(SHRINK).max(1) as usize;
    let height = window.1.div_ceil(SHRINK).max(1) as usize;
    // The part of the picture that covers the window, in the picture's pixels (in 1/2 pixels
    // to stay in integers): the window's shape inside the picture.
    let (cw, ch) = if u64::from(window.0) * sh as u64 >= u64::from(window.1) * sw as u64 {
        // The window is wider: the whole width, a band of the height.
        (
            sw,
            (sw as u64 * u64::from(window.1) / u64::from(window.0)).max(1) as usize,
        )
    } else {
        (
            (sh as u64 * u64::from(window.0) / u64::from(window.1)).max(1) as usize,
            sh,
        )
    };
    let (cw, ch) = (cw.min(sw), ch.min(sh));
    let (left, top) = ((sw - cw) / 2, (sh - ch) / 2);
    // Each wash pixel is the average of the box it covers (at least one pixel).
    let mut sums = vec![[0f32; 3]; width * height];
    for (y, row) in sums.chunks_exact_mut(width).enumerate() {
        let y0 = top + y * ch / height;
        let y1 = (top + (y + 1) * ch / height).max(y0 + 1).min(sh);
        for (x, out) in row.iter_mut().enumerate() {
            let x0 = left + x * cw / width;
            let x1 = (left + (x + 1) * cw / width).max(x0 + 1).min(sw);
            let mut total = [0u32; 3];
            for sy in y0..y1 {
                for pixel in source[(sy * sw + x0) * 4..(sy * sw + x1) * 4]
                    .as_chunks::<4>()
                    .0
                {
                    total[0] += u32::from(pixel[0]);
                    total[1] += u32::from(pixel[1]);
                    total[2] += u32::from(pixel[2]);
                }
            }
            let count = ((y1 - y0) * (x1 - x0)) as f32;
            *out = total.map(|t| t as f32 / count);
        }
    }
    for _ in 0..3 {
        box_blur(&mut sums, width, height, BLUR_RADIUS);
    }
    let mut pixels = Vec::with_capacity(width * height * 4);
    for sum in &sums {
        for channel in sum {
            pixels.push(((channel * BRIGHTNESS as f32 / 256.0) + 0.5) as u8);
        }
        pixels.push(0);
    }
    Some(Wash {
        width: width as u32,
        height: height as u32,
        pixels,
    })
}

/// A picture centred in a frame wider than it: `picture` (`picture_width` x `height` pixels of
/// four bytes) in the middle of a frame `width` wide, the sides filled with the picture's wash
/// (magnified with a linear filter as the window shows it). The picture's pixels are kept as
/// they are. `None` when the picture is not narrower than the frame or has no size.
pub fn centred_in_wash(
    picture: &[u8],
    picture_width: u32,
    width: u32,
    height: u32,
) -> Option<Vec<u8>> {
    if picture_width == 0 || picture_width >= width || height == 0 {
        return None;
    }
    let wash = make(picture, (picture_width, height), (width, height))?;
    let (ww, wh) = (wash.width as usize, wash.height as usize);
    let at = |x: usize, y: usize, c: usize| f32::from(wash.pixels[(y * ww + x) * 4 + c]);
    let left = ((width - picture_width) / 2) as usize;
    let (w, h, pw) = (width as usize, height as usize, picture_width as usize);
    let mut out = vec![0u8; w * h * 4];
    for y in 0..h {
        let fy = ((y as f32 + 0.5) * wh as f32 / h as f32 - 0.5).clamp(0.0, (wh - 1) as f32);
        let (y0, ty) = (fy as usize, fy.fract());
        let y1 = (y0 + 1).min(wh - 1);
        let row = &mut out[y * w * 4..(y + 1) * w * 4];
        for x in (0..left).chain(left + pw..w) {
            let fx = ((x as f32 + 0.5) * ww as f32 / w as f32 - 0.5).clamp(0.0, (ww - 1) as f32);
            let (x0, tx) = (fx as usize, fx.fract());
            let x1 = (x0 + 1).min(ww - 1);
            for c in 0..3 {
                let top = at(x0, y0, c) * (1.0 - tx) + at(x1, y0, c) * tx;
                let bottom = at(x0, y1, c) * (1.0 - tx) + at(x1, y1, c) * tx;
                row[x * 4 + c] = (top * (1.0 - ty) + bottom * ty + 0.5) as u8;
            }
        }
        row[left * 4..(left + pw) * 4].copy_from_slice(&picture[y * pw * 4..(y + 1) * pw * 4]);
    }
    Some(out)
}

/// One box blur of `radius` over the rows, then over the columns; the picture's edge repeats.
fn box_blur(data: &mut [[f32; 3]], width: usize, height: usize, radius: usize) {
    let mut line = Vec::new();
    for y in 0..height {
        line.clear();
        line.extend_from_slice(&data[y * width..(y + 1) * width]);
        blur_line(&line, &mut data[y * width..(y + 1) * width], radius);
    }
    for x in 0..width {
        line.clear();
        line.extend((0..height).map(|y| data[y * width + x]));
        let mut out = vec![[0f32; 3]; height];
        blur_line(&line, &mut out, radius);
        for (y, value) in out.into_iter().enumerate() {
            data[y * width + x] = value;
        }
    }
}

/// `line` averaged over `2 * radius + 1` neighbours (clamped at its ends) into `out`.
fn blur_line(line: &[[f32; 3]], out: &mut [[f32; 3]], radius: usize) {
    let last = line.len() - 1;
    let count = (2 * radius + 1) as f32;
    for (i, out) in out.iter_mut().enumerate() {
        let mut total = [0f32; 3];
        for k in 0..=2 * radius {
            let at = (i + k).saturating_sub(radius).min(last);
            for (sum, value) in total.iter_mut().zip(line[at]) {
                *sum += value;
            }
        }
        *out = total.map(|t| t / count);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn flat(width: u32, height: u32, colour: [u8; 4]) -> Vec<u8> {
        colour.repeat((width * height) as usize)
    }

    #[test]
    fn a_flat_picture_gives_a_darker_flat_wash_of_a_sixteenth_the_window() {
        let wash = make(&flat(800, 600, [200, 100, 40, 0]), (800, 600), (1920, 1080)).unwrap();
        assert_eq!((wash.width, wash.height), (120, 68));
        for pixel in wash.pixels.as_chunks::<4>().0 {
            assert_eq!(*pixel, [150, 75, 30, 0]);
        }
    }

    #[test]
    fn the_wash_follows_the_picture_and_is_blurred() {
        // Left half white, right half black.
        let mut picture = vec![0u8; 800 * 600 * 4];
        for row in picture.as_chunks_mut::<{ 800 * 4 }>().0 {
            row[..400 * 4].fill(255);
        }
        let wash = make(&picture, (800, 600), (1600, 1200)).unwrap();
        let w = wash.width as usize;
        let at = |x: usize, y: usize| wash.pixels[(y * w + x) * 4];
        assert_eq!(at(0, 10), 191);
        assert_eq!(at(w - 1, 10), 0);
        // The edge in the middle is a ramp, not a step.
        let ramp: Vec<u8> = (w / 2 - 6..w / 2 + 6).map(|x| at(x, 10)).collect();
        assert!(ramp.windows(2).all(|p| p[0] >= p[1]));
        assert!(ramp.iter().filter(|&&v| v > 0 && v < 191).count() >= 6);
    }

    #[test]
    fn a_wide_window_crops_the_picture_to_its_shape() {
        // Rows of the top and bottom quarters differ from the middle.
        let mut picture = vec![0u8; 800 * 600 * 4];
        for (y, row) in picture
            .as_chunks_mut::<{ 800 * 4 }>()
            .0
            .iter_mut()
            .enumerate()
        {
            if (150..450).contains(&y) {
                row.fill(255);
            }
        }
        // 16:9 shows the middle 450 of 600 rows (75..525): the middle is white, no black row
        // is wholly in view but the edges blend towards the black outside.
        let wash = make(&picture, (800, 600), (1920, 1080)).unwrap();
        let w = wash.width as usize;
        assert!(wash.pixels[(34 * w + 60) * 4] > 150);
        assert!(wash.pixels[60 * 4] < wash.pixels[(34 * w + 60) * 4]);
    }

    #[test]
    fn a_picture_is_centred_in_its_own_wash() {
        let picture = flat(4, 4, [200, 100, 40, 255]);
        let out = centred_in_wash(&picture, 4, 12, 4).unwrap();
        assert_eq!(out.len(), 12 * 4 * 4);
        let row = &out[..12 * 4];
        assert_eq!(&row[4 * 4..8 * 4], &picture[..4 * 4]);
        assert_eq!(&row[..4], [150, 75, 30, 0]);
        assert_eq!(&row[11 * 4..], [150, 75, 30, 0]);
        assert!(centred_in_wash(&picture, 12, 12, 4).is_none());
    }

    #[test]
    fn a_picture_with_no_size_gives_nothing() {
        assert!(make(&[], (0, 0), (100, 100)).is_none());
        assert!(make(&[0; 16], (2, 2), (0, 100)).is_none());
        assert!(make(&[0; 8], (2, 2), (100, 100)).is_none());
    }
}
