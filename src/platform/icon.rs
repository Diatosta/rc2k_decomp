//! The program's icon (`assets/icon.png`, 512 x 512, built in): read into pixels for the window's
//! icon, and its file for the installer's menu entries on Linux.

use rc2k_core::gfx::png;

/// The icon's PNG file.
pub const PNG: &[u8] = include_bytes!("../../assets/icon.png");

/// The window's icon: its width, height and pixels, three bytes each (red, green, blue), rows
/// without a gap.
pub struct Pixels {
    pub width: usize,
    pub height: usize,
    pub rgb: Vec<u8>,
}

/// The icon read, then made `scale` times smaller (each pixel the mean of a square of the
/// file's), as window systems want a small one. None when the file is not an 8-bit RGB PNG.
pub fn pixels(scale: usize) -> Option<Pixels> {
    let (width, height, rgb) = decode(PNG)?;
    let scale = scale.max(1);
    let (w, h) = (width / scale, height / scale);
    let mut small = vec![0u8; w * h * 3];
    for y in 0..h {
        for x in 0..w {
            for c in 0..3 {
                let mut sum = 0usize;
                for dy in 0..scale {
                    for dx in 0..scale {
                        sum += rgb[((y * scale + dy) * width + x * scale + dx) * 3 + c] as usize;
                    }
                }
                small[(y * w + x) * 3 + c] = (sum / (scale * scale)) as u8;
            }
        }
    }
    Some(Pixels {
        width: w,
        height: h,
        rgb: small,
    })
}

/// An 8-bit RGB PNG, not interlaced, its data in one or more chunks: (width, height, pixels).
fn decode(file: &[u8]) -> Option<(usize, usize, Vec<u8>)> {
    if file.get(..8)? != png::SIGNATURE {
        return None;
    }
    let be = |at: usize| Some(u32::from_be_bytes(file.get(at..at + 4)?.try_into().ok()?));
    let (mut at, mut size, mut data) = (8usize, None, Vec::new());
    while at + 8 <= file.len() {
        let length = be(at)? as usize;
        let body = file.get(at + 8..at + 8 + length)?;
        match &file[at + 4..at + 8] {
            b"IHDR" => {
                // 8 bits, RGB, not interlaced
                if body.get(8..13)? != [8, 2, 0, 0, 0] {
                    return None;
                }
                size = Some((be(at + 8)? as usize, be(at + 12)? as usize));
            }
            b"IDAT" => data.extend_from_slice(body),
            b"IEND" => break,
            _ => {}
        }
        at += 12 + length;
    }
    let (width, height) = size?;
    let mut area = miniz_oxide::inflate::decompress_to_vec_zlib(&data).ok()?;
    let row_bytes = width * 3;
    if area.len() < (row_bytes + 1) * height {
        return None;
    }
    png::unfilter(&mut area, row_bytes, height as u32, 3, &|_| 0).ok()?;
    area.truncate(row_bytes * height);
    Some((width, height, area))
}

#[cfg(test)]
mod tests {
    #[test]
    fn the_built_in_icon_reads_and_shrinks() {
        let full = super::pixels(1).expect("the icon reads");
        assert_eq!(
            (full.width, full.height, full.rgb.len()),
            (512, 512, 512 * 512 * 3)
        );
        let small = super::pixels(4).unwrap();
        assert_eq!((small.width, small.height), (128, 128));
        // the corner is the plain blue background
        assert_eq!(&small.rgb[..3], &[51, 58, 161]);
    }
}
