//! Pixels, palettes and textures.
//!
//! The graphics work in "components": a dword whose bytes 0, 1, 2 are a colour and byte 3
//! the alpha. A `PixelFormat` says how a surface's or texture's pixel holds them.

/// How a pixel holds the four components.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct PixelFormat {
    /// With the sign bit set: four rotation counts, one byte per component, lowest first.
    /// Otherwise the number of the pixel's top bit, and the conversions go bit by bit.
    pub layout: u32,
    /// The bits of the pixel each of the four components occupies.
    pub masks: [u32; 4],
}

impl PixelFormat {
    /// The four rotation counts, if this is the kind of layout that has them.
    pub(crate) fn rotations(&self) -> Option<[u32; 4]> {
        ((self.layout as i32) < 0).then(|| self.layout.to_le_bytes().map(|b| b as u32 & 31))
    }
}

/// Empty, or one run of bits.
fn one_run(mask: u32) -> bool {
    let run = mask.checked_shr(mask.trailing_zeros()).unwrap_or(0);
    run & run.wrapping_add(1) == 0
}

/// The number of the top bit, 0xFF for none.
fn top_bit(mask: u32) -> u8 {
    if mask == 0 {
        0xFF
    } else {
        31 - mask.leading_zeros() as u8
    }
}

/// The layout and the format's number for four masks. If each mask is
/// empty or one run of bits: the masks' top bits less 7, 15, 23, 31 as rotation counts
/// with the sign bit, and, if the masks together are one run too, the number `numbers`
/// (number, layout) gives that layout (else 0). With a gap in a mask: the top bit of all
/// and 0.
pub fn describe(masks: [u32; 4], numbers: &[(u32, u32)]) -> (u32, u32) {
    let all = masks.iter().fold(0, |all, &mask| all | mask);
    if !masks.iter().all(|&mask| one_run(mask)) {
        return (top_bit(all) as u32, 0);
    }
    let counts = [7u8, 15, 23, 31];
    let bytes: [u8; 4] = std::array::from_fn(|n| top_bit(masks[n]).wrapping_sub(counts[n]));
    let layout = u32::from_le_bytes(bytes) | 0x8000_0000;
    let number = one_run(all)
        .then(|| numbers.iter().find(|&&(_, known)| known == layout))
        .flatten()
        .map_or(0, |&(number, _)| number);
    (layout, number)
}

/// Components made a pixel, and for a bit-by-bit format what is left of the four bytes it shifts
/// the components out of (the caller keeps them).
pub fn pack(format: &PixelFormat, components: u32) -> (u32, Option<[u8; 4]>) {
    if components == 0 {
        return (0, None);
    }
    if let Some(rotations) = format.rotations() {
        let pixel = (0..4).fold(0, |all, n| {
            all | components.rotate_left(rotations[n]) & format.masks[n]
        });
        return (pixel, None);
    }
    let mut left = components.to_le_bytes();
    let mut pixel = 0u32;
    let mut bit = 1u32 << (format.layout & 31);
    while bit != 0 {
        let next = match format.masks.iter().position(|mask| mask & bit != 0) {
            Some(n) => {
                let top = left[n] >> 7;
                left[n] <<= 1;
                top as u32
            }
            None => 0,
        };
        pixel = pixel << 1 | next;
        bit >>= 1;
    }
    (pixel, Some(left))
}

/// A pixel taken apart into components: rotated back, or bit by
/// bit from the top, at most eight a component, left-aligned when fewer.
pub fn unpack(format: &PixelFormat, pixel: u32) -> u32 {
    if pixel == 0 {
        return 0;
    }
    if let Some(rotations) = format.rotations() {
        return (0..4).fold(0, |all, n| {
            all | (pixel & format.masks[n]).rotate_right(rotations[n])
        });
    }
    let mut components = [0u8; 4];
    for (n, component) in components.iter_mut().enumerate() {
        // A 1 marks how far the byte has been filled: when it falls out, there are eight.
        let mut bits: u8 = 1;
        let mut full = false;
        let mut bit = format.layout as i32;
        while bit >= 0 && !full {
            let at = bit as u32 & 31;
            if format.masks[n] >> at & 1 != 0 {
                full = bits & 0x80 != 0;
                bits = bits << 1 | (pixel >> at & 1) as u8;
            }
            bit -= 1;
        }
        while !full {
            full = bits & 0x80 != 0;
            bits <<= 1;
        }
        *component = bits;
    }
    u32::from_le_bytes(components)
}

/// The components of the format's least colour: the lowest bit of each run of each mask.
pub fn least_colour(format: &PixelFormat) -> u32 {
    let pixel = format
        .masks
        .iter()
        .fold(0, |acc, &m| acc | (m & !m.wrapping_mul(2)));
    unpack(format, pixel)
}

/// One entry of a palette between two: each of the three colour
/// components first + ((second - first) * `towards` >> 8); the fourth is `old`'s top byte
/// (what the entry held before). The pixel, and the bytes `pack` leaves.
pub fn blend_entry(
    format: &PixelFormat,
    first: u32,
    second: u32,
    old: u32,
    towards: u32,
) -> (u32, Option<[u8; 4]>) {
    let a = unpack(format, first).to_le_bytes();
    let b = unpack(format, second).to_le_bytes();
    let mut entry = old.to_le_bytes();
    for c in 0..3 {
        let (from, to) = (a[c] as i32, b[c] as i32);
        let step = (to - from).wrapping_mul(towards as i32) >> 8;
        entry[c] = from.wrapping_add(step) as u8;
    }
    pack(format, u32::from_le_bytes(entry))
}

/// One byte brought toward another: s + ((t - s) * amount >> 8), held to 0 .. 0xFF. Gives the
/// byte and the high dword of the product (the caller keeps it).
pub fn fade_step(s: u32, t: u32, amount: u32) -> (u32, u32) {
    let p = i64::from(t.wrapping_sub(s) as i32) * i64::from(amount as i32);
    let v = ((p as u32 as i32) >> 8).wrapping_add(s as i32);
    let v = if v < 0 {
        0
    } else if v.wrapping_sub(0xFF) < 0 {
        v as u32
    } else {
        0xFF
    };
    (v, (p >> 32) as u32)
}

/// A palette entry of three bytes as a dword: the first byte up 16, the second up 8.
pub fn palette_entry([first, second, third]: [u8; 3]) -> u32 {
    (first as u32) << 16 | (second as u32) << 8 | third as u32
}

/// n / 3 (as the thirds table holds it, n up to 0x2FF).
fn thirds(n: u32) -> u32 {
    n / 3
}

/// Where a texture's second picture is, in bytes after the first's start.
pub const KEPT: usize = 0x40000;

/// Each of `count` pixels' alpha made the inverted first byte of the pixel `KEPT` bytes on, in
/// order.
pub fn alpha_from_kept(buffer: &mut [u8], count: usize) -> Option<()> {
    for n in 0..count {
        let kept = *buffer.get(KEPT + n * 4)?;
        *buffer.get_mut(n * 4 + 3)? = !kept;
    }
    Some(())
}

#[cfg(test)]
/// Alpha made 0xFF; a colour whose top five bits are all 0 becomes the key, magenta
/// (0xF800F8 by those bits) with alpha 0.
pub(crate) fn key_colours(pixels: &mut [u32], key: u32) {
    let key = key | 0xFF00_0000;
    for p in pixels {
        *p = match *p & 0xF8_F8F8 {
            0 => key,
            0xF8_00F8 => 0,
            _ => *p | 0xFF00_0000,
        };
    }
}

/// `key_colours` on pixels held as little-endian bytes, in place.
pub fn key_colours_in_bytes(pixels: &mut [u8], key: u32) {
    let key = key | 0xFF00_0000;
    for bytes in pixels.as_chunks_mut::<4>().0 {
        let p = u32::from_le_bytes(*bytes);
        let keyed = match p & 0xF8_F8F8 {
            0 => key,
            0xF8_00F8 => 0,
            _ => p | 0xFF00_0000,
        };
        *bytes = keyed.to_le_bytes();
    }
}

/// The table of 0x100 bytes a grading scale makes.
pub fn grades(scale: u32) -> [u8; 0x100] {
    let mut table = [0u8; 0x100];
    let mut sum = 0u32;
    for n in 0..0x100 {
        table[n] = (sum >> 8) as u8;
        sum = sum.wrapping_add(scale);
        if sum > 0xFF00 {
            table[n..].fill(0xFF);
            break;
        }
    }
    table
}

/// Whether textures are graded: the reading switch at 1 and the two settings not both 0x100.
pub fn grading(reading: u32, scale: u32, saturation: u32) -> bool {
    reading == 1 && !(scale == 0x100 && saturation == 0x100)
}

/// A texture's colours made brighter or duller: each pixel of an alpha
/// above 0 through `table`, then drawn towards or away from the colours' average by
/// `saturation`; one left black by the top five bits becomes the key.
pub fn grade(pixels: &mut [u32], key: u32, saturation: u32, table: &[u8; 0x100]) {
    let key = key & 0xFF_FFFF;
    let saturation = saturation as i32;
    for pixel in pixels {
        let mut p = pixel.to_le_bytes();
        if p[3] == 0 {
            continue;
        }
        let parts = [0, 1, 2].map(|i| table[p[i] as usize] as u32);
        let average = thirds(parts[2] + parts[0] + parts[1]) as i32;
        for (i, &part) in parts.iter().enumerate() {
            let v = ((part as i32 - average).wrapping_mul(saturation) >> 8).wrapping_add(average);
            p[i] = v.clamp(0, 0xFF) as u8;
        }
        let whole = u32::from_le_bytes(p);
        *pixel = if whole & 0xF8_F8F8 == 0 {
            whole & 0xFF00_0000 | key
        } else {
            whole
        };
    }
}

/// Whether `halve` can do it: every pass at least one pixel wide and high.
pub fn halvable(width: u32, height: u32, times: u32) -> bool {
    times <= 32 && (width >> times) > 0 && (height >> times) > 0
}

/// A texture made half as wide and high in place, `times` times: each
/// pixel the average of four, the colour counting only those of an alpha above 0. The
/// size after the last pass; `None` if the pixels are fewer than the size says.
pub fn halve(pixels: &mut [u32], width: u32, height: u32, times: u32) -> Option<(u32, u32)> {
    let (mut width, mut height) = (width, height);
    for _ in 0..times {
        let row = width as usize;
        width >>= 1;
        height >>= 1;
        let (mut from, mut to) = (0usize, 0usize);
        for _ in 0..height {
            for _ in 0..width {
                let four = [
                    *pixels.get(from)?,
                    *pixels.get(from + 1)?,
                    *pixels.get(from + row)?,
                    *pixels.get(from + row + 1)?,
                ];
                let (mut alpha, mut sums, mut counted) = (0u32, [0u32; 3], 0u32);
                for p in four {
                    if p >> 24 != 0 {
                        alpha += p >> 24;
                        for (i, sum) in sums.iter_mut().enumerate() {
                            *sum += (p >> (i * 8)) & 0xFF;
                        }
                        counted += 1;
                    }
                }
                let alpha = (alpha >> 2) & 0xFF;
                pixels[to] = if alpha == 0 {
                    0
                } else {
                    let sums = sums.map(|s| match counted {
                        1 => s,
                        2 => s >> 1,
                        3 => thirds(s),
                        _ => s >> 2,
                    });
                    alpha << 24 | sums[2] << 16 | sums[1] << 8 | sums[0]
                };
                from += 2;
                to += 1;
            }
            from += row;
        }
    }
    Some((width, height))
}

/// The masks of the components format (bytes 0 .. 3).
pub const COMPONENT_MASKS: [u32; 4] = [0xFF, 0xFF00, 0xFF_0000, 0xFF00_0000];

/// The table of n / 3 for n = 0 .. 0x2FF.
pub fn thirds_table() -> [u8; 0x300] {
    std::array::from_fn(|n| (n / 3) as u8)
}

/// `count` pixels of three bytes turned into dwords in place, from the last down (the first byte
/// up 16, the second up 8, the top byte 0). `None` if the buffer is shorter than the dwords.
pub fn rgb_to_dwords(buffer: &mut [u8], count: usize) -> Option<()> {
    if buffer.len() < count.checked_mul(4)? {
        return None;
    }
    for n in (0..count).rev() {
        let dword = palette_entry([buffer[n * 3], buffer[n * 3 + 1], buffer[n * 3 + 2]]);
        buffer[n * 4..n * 4 + 4].copy_from_slice(&dword.to_le_bytes());
    }
    Some(())
}

/// `count` palette numbers (a byte a pixel) turned into the palette's dwords in place, from the
/// last down. `None` if the buffer is shorter than the dwords.
pub fn palette_to_dwords(buffer: &mut [u8], count: usize, palette: &[u32; 256]) -> Option<()> {
    if buffer.len() < count.checked_mul(4)? {
        return None;
    }
    for n in (0..count).rev() {
        let dword = palette[buffer[n] as usize];
        buffer[n * 4..n * 4 + 4].copy_from_slice(&dword.to_le_bytes());
    }
    Some(())
}

#[cfg(test)]
/// A pixel of four eight-bit components as format 4's two bytes: the top four bits of the first
/// and second component in the first byte (low half, high half), of the third and fourth in the
/// second.
pub(crate) fn to_4444(c: [u8; 4]) -> [u8; 2] {
    [c[0] >> 4 | c[1] & 0xF0, c[2] >> 4 | c[3] & 0xF0]
}

/// A row of pixels of four components packed into the surface's format: `from` holds the row's
/// components, `to` gets words (`words`, two bytes a pixel) or dwords. Gives the last scratch the
/// packing left. Refused: a row longer than either side.
pub fn pack_row(
    from: &[u8],
    to: &mut [u8],
    pixels: usize,
    format: &PixelFormat,
    words: bool,
) -> Result<Option<[u8; 4]>, crate::Invalid> {
    let size = if words { 2 } else { 4 };
    if from.len() < pixels * 4 || to.len() < pixels * size {
        return Err(crate::Invalid(
            "a row longer than its picture or its surface",
        ));
    }
    let mut kept = None;
    let (from, _) = from.as_chunks::<4>();
    let from = &from[..pixels];
    if let Some(rotations) = format.rotations() {
        // Rotation counts and masks read once for the row.
        let [r0, r1, r2, r3] = rotations;
        let [m0, m1, m2, m3] = format.masks;
        let pack = |c: &[u8; 4]| {
            let c = u32::from_le_bytes(*c);
            c.rotate_left(r0) & m0
                | c.rotate_left(r1) & m1
                | c.rotate_left(r2) & m2
                | c.rotate_left(r3) & m3
        };
        if words {
            let (to, _) = to.as_chunks_mut::<2>();
            for (c, out) in from.iter().zip(to) {
                *out = (pack(c) as u16).to_le_bytes();
            }
        } else {
            let (to, _) = to.as_chunks_mut::<4>();
            for (c, out) in from.iter().zip(to) {
                *out = pack(c).to_le_bytes();
            }
        }
        return Ok(None);
    }
    for (n, c) in from.iter().enumerate() {
        let (pixel, scratch) = pack(format, u32::from_le_bytes(*c));
        kept = scratch.or(kept);
        to[n * size..n * size + size].copy_from_slice(&pixel.to_le_bytes()[..size]);
    }
    Ok(kept)
}

/// The row table: `rows` addresses, `step` apart from `first` (wrapping).
pub fn row_table(first: u32, step: u32, rows: usize) -> Vec<u32> {
    (0..rows as u32)
        .map(|n| first.wrapping_add(n.wrapping_mul(step)))
        .collect()
}

/// The game fonts' second palette: a palette entry made brighter, its first component + 0x40,
/// the second and third + 0x20, each stopping at 0xFF; the alpha kept.
pub fn brighter(entry: u32) -> u32 {
    let [a, b, c, d] = entry.to_le_bytes();
    u32::from_le_bytes([
        a.saturating_add(0x40),
        b.saturating_add(0x20),
        c.saturating_add(0x20),
        d,
    ])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn brighter_entries() {
        assert_eq!(brighter(0x8010_E0D0), 0x8030_FFFF);
        assert_eq!(brighter(0), 0x0020_2040);
    }

    const RGB565: PixelFormat = PixelFormat {
        layout: 0x9810_081D,
        masks: [0xF800, 0x07E0, 0x001F, 0x8000_0000],
    };

    #[test]
    fn formats() {
        // Top bits 15, 10, 4, 31 less 7, 15, 23, 31: 8, -5, -19, 0.
        let (layout, number) = describe(RGB565.masks, &[(3, 0x80ED_FB08)]);
        assert_eq!(layout, 0x80ED_FB08);
        assert_eq!(number, 0); // the masks together are not one run (bit 31)
        let rgb565 = PixelFormat { layout, ..RGB565 };
        let (_, number) = describe([0xF800, 0x07E0, 0x001F, 0], &[(4, 0x8710_081D)]);
        assert_eq!(number, 0);
        assert_eq!(describe([0xF0F0, 0, 0, 0], &[]), (15, 0));
        // Rotation formats round-trip the bits they keep.
        let pixel = 0xA5C3;
        assert_eq!(pack(&rgb565, unpack(&rgb565, pixel)).0, pixel);
        // Bit-by-bit formats give the same as the rotated ones for plain layouts.
        let bitwise = PixelFormat {
            layout: 15,
            masks: [0xF800, 0x07E0, 0x001F, 0],
        };
        let plain = PixelFormat {
            layout: describe(bitwise.masks, &[]).0,
            masks: bitwise.masks,
        };
        assert_eq!(unpack(&bitwise, pixel), unpack(&plain, pixel) & 0xFF_FFFF);
        assert_eq!(pack(&bitwise, 0x00F8_FCF8).0, 0xFFFF);
        assert_eq!(least_colour(&plain), 0x0008_0408);
    }

    #[test]
    fn colours() {
        assert_eq!(fade_step(0, 0xFF, 0x80).0, 0x7F);
        assert_eq!(fade_step(0x80, 0, 0x100).0, 0);
        assert_eq!(fade_step(0x10, 0xFF, 0x200).0, 0xFF);
        assert_eq!(palette_entry([1, 2, 3]), 0x01_0203);
        let mut rgb = [1u8, 2, 3, 4, 5, 6, 0, 0];
        rgb_to_dwords(&mut rgb, 2).unwrap();
        assert_eq!(rgb, [3, 2, 1, 0, 6, 5, 4, 0]);
        assert_eq!(rgb_to_dwords(&mut [0u8; 7], 2), None);
        assert_eq!(thirds_table()[0x2FF], 0xFF);
        let format = PixelFormat {
            layout: 0x8000_0000,
            masks: [0xFF, 0xFF00, 0xFF_0000, 0],
        };
        assert_eq!(
            blend_entry(&format, 0x00_0000, 0xFF_FFFF, 0, 0x80).0,
            0x7F_7F7F
        );
    }

    #[test]
    fn textures() {
        let mut pixels = [0x0000_0001, 0x00F8_00F8, 0x0012_3456];
        key_colours(&mut pixels, 0x0808);
        assert_eq!(pixels, [0xFF00_0808, 0, 0xFF12_3456]);
        let mut big = vec![0xFF00_0000u32 | 0x40, 0xFF00_0000 | 0x80, 0, 0xFF00_0000];
        assert_eq!(halve(&mut big, 2, 2, 1), Some((1, 1)));
        assert_eq!(big[0] & 0xFF, (0x40 + 0x80) / 3); // three counted
        assert_eq!(big[0] >> 24, (0xFF * 3) >> 2);
        assert_eq!(halve(&mut [0u32; 3], 2, 2, 1), None);
        assert!(!halvable(4, 1, 1));
        let table = grades(0x100);
        assert_eq!(table[0x80], 0x80);
        let mut pixels = [0xFF10_2030u32];
        grade(&mut pixels, 0, 0x100, &table);
        assert_eq!(pixels, [0xFF10_2030]);
        let mut buffer = vec![0u8; KEPT + 8];
        buffer[KEPT] = 0x0F;
        alpha_from_kept(&mut buffer, 2).unwrap();
        assert_eq!(buffer[3], 0xF0);
        assert_eq!(buffer[7], 0xFF);
    }

    #[test]
    fn palette_and_4444() {
        let mut palette = [0u32; 256];
        palette[2] = 0x0011_2233;
        palette[7] = 0xAABB_CCDD;
        let mut buffer = [2u8, 7, 0, 0, 0, 0, 0, 0];
        palette_to_dwords(&mut buffer, 2, &palette).unwrap();
        assert_eq!(buffer, [0x33, 0x22, 0x11, 0, 0xDD, 0xCC, 0xBB, 0xAA]);
        assert!(palette_to_dwords(&mut buffer, 3, &palette).is_none());
        assert_eq!(to_4444([0x12, 0x34, 0x56, 0x78]), [0x31, 0x75]);
    }

    /// A deterministic spread of components, zero ones included.
    fn components(n: usize) -> Vec<u8> {
        (0..n * 4)
            .map(|i| {
                let v = (i as u32).wrapping_mul(2_654_435_761) >> 13;
                if v.is_multiple_of(7) { 0 } else { v as u8 }
            })
            .collect()
    }

    /// `pack_row` is `pack` pixel by pixel, for both layouts and both pixel sizes, and keeps the
    /// scratch of the last pixel that had one.
    #[test]
    fn rows_pack_like_single_pixels() {
        let bitwise = PixelFormat {
            layout: 15,
            masks: [0xF800, 0x07E0, 0x001F, 0],
        };
        for format in [
            RGB565,
            PixelFormat {
                layout: 0x8710_081D,
                ..RGB565
            },
            bitwise,
        ] {
            for words in [false, true] {
                let size = if words { 2 } else { 4 };
                let from = components(37);
                let mut row = vec![0xAA; 37 * size];
                let kept = pack_row(&from, &mut row, 37, &format, words).unwrap();
                let mut want = Vec::new();
                let mut want_kept = None;
                for c in from.as_chunks::<4>().0 {
                    let (pixel, scratch) = pack(&format, u32::from_le_bytes(*c));
                    want.extend_from_slice(&pixel.to_le_bytes()[..size]);
                    want_kept = scratch.or(want_kept);
                }
                assert_eq!(row, want);
                assert_eq!(kept, want_kept);
            }
        }
        assert!(pack_row(&[0; 8], &mut [0; 8], 3, &RGB565, false).is_err());
    }

    /// `key_colours_in_bytes` is `key_colours` on the dwords.
    #[test]
    fn keyed_bytes_are_keyed_dwords() {
        let bytes = components(64);
        let mut dwords: Vec<u32> = bytes
            .as_chunks::<4>()
            .0
            .iter()
            .map(|c| u32::from_le_bytes(*c))
            .collect();
        dwords.extend([0, 0xF8_00F8, 0x0100_0000]);
        let mut bytes = bytes;
        bytes.extend(dwords[64..].iter().flat_map(|d| d.to_le_bytes()));
        key_colours(&mut dwords, 0x0012_3456);
        key_colours_in_bytes(&mut bytes, 0x0012_3456);
        let want: Vec<u8> = dwords.iter().flat_map(|d| d.to_le_bytes()).collect();
        assert_eq!(bytes, want);
    }
}
