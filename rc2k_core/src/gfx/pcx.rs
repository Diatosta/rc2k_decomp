//! PCX pictures.
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

/// What a decoding asks of its caller.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Target {
    /// The picture buffer's size.
    pub width: u32,
    pub height: u32,
    /// The widest and highest picture taken.
    pub width_limit: u32,
    pub height_limit: u32,
    /// The picture flags: bit 8 for 8-bit pixels.
    pub flags: u32,
    /// The picture's row bytes: 0xFFFFFFFF a screen's width of pixels, 0xFFFFFFFE the buffer's
    /// width, 0xFFFFFFFD the decoded row itself, else that many bytes.
    pub row_bytes: u32,
}

/// How a picture is decoded.
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

/// How a PCX file is decoded, by its header's planes (+0x41): 1 with a palette, 3 as three
/// planes, else not at all.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Palette,
    Planes,
}

pub fn kind(header: &[u8; HEADER_LEN]) -> Option<Kind> {
    match header[0x41] {
        1 => Some(Kind::Palette),
        3 => Some(Kind::Planes),
        _ => None,
    }
}

/// The layout a header makes for `target`.
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

/// The rows of a one-plane picture through `palette`. `next` reads the file's
/// next byte as the stream does (`ByteRead::value`: 0 at the end of the file). `put` takes a
/// pixel at a byte offset from the first row's start (wrapping). A run goes on across the end of a
/// row; a count of 0 is a run of 2^32. `None` for a picture without rows (its count would wrap).
pub fn decode_8bit<E>(
    layout: &Layout,
    palette: &[u32; 256],
    mut next: impl FnMut() -> Result<u8, E>,
    mut put: impl FnMut(u32, u32),
) -> Result<Option<()>, E> {
    if layout.height == 0 {
        return Ok(None);
    }
    let mut to = 0u32;
    // The pixel a run repeats, and how much of the run is left.
    let (mut pixel, mut run) = (0u32, 0u32);
    for _ in 0..layout.height {
        let mut column = 0u32;
        loop {
            if run == 0 {
                let mut byte = next()?;
                run = 1;
                if byte >= 0xC0 {
                    run = byte as u32 - 0xC0;
                    byte = next()?;
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
    Ok(Some(()))
}

/// The rows of a picture of planes, 8 bits each: plane n's bytes go to byte `planes - 1 - n` of
/// the pixels; then every pixel's fourth byte is 0. `put` takes a byte at an offset from the first
/// row's start (wrapping). A run goes on across the ends of planes and rows, but the byte it
/// repeats is 0 from the end of a row on. `None` for a picture without rows or columns (the
/// counts would wrap).
pub fn decode_planes<E>(
    layout: &Layout,
    mut next: impl FnMut() -> Result<u8, E>,
    mut put: impl FnMut(u32, u8),
) -> Result<Option<()>, E> {
    if layout.width == 0 || layout.height == 0 {
        return Ok(None);
    }
    let mut to = 0u32;
    // The byte a run repeats, and what is left of the run, counted down before it is looked
    // at, as a signed number.
    let (mut byte, mut run) = (0u8, 0i32);
    for _ in 0..layout.height {
        to = to.wrapping_add(layout.planes);
        let mut plane = 0u32;
        loop {
            to = to.wrapping_sub(1);
            let mut column = 0u32;
            loop {
                run = run.wrapping_sub(1);
                if run <= 0 {
                    byte = next()?;
                    run = 1;
                    if byte >= 0xC0 {
                        run = (byte - 0xC0) as i32;
                        byte = next()?;
                    }
                }
                if column < layout.width {
                    put(to.wrapping_add(column.wrapping_mul(4)), byte);
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
        // A slip kept: a run going on into the next row repeats 0.
        byte = 0;
        for pixel in (1..=layout.width).rev() {
            put(to.wrapping_add(pixel.wrapping_mul(4)).wrapping_sub(1), 0);
        }
        to = to.wrapping_add(layout.stride);
    }
    Ok(Some(()))
}

/// What `decode_planes_in` refuses: a pixel that would land past the buffer.
pub const PAST_BUFFER: crate::Invalid = crate::Invalid("a decoded pixel past the picture buffer");

/// `decode_planes` on a file held in memory: `file` is everything after the header (the bytes
/// past its end read as 0), `out` the buffer from the first row's start. The same bytes land in
/// `out` as `decode_planes` puts, a run or a stretch of plain bytes at a time.
/// Refused: a pixel past `out` (bytes before it may have been written).
pub fn decode_planes_in(
    layout: &Layout,
    file: &[u8],
    out: &mut [u8],
) -> Result<Option<()>, crate::Invalid> {
    if layout.width == 0 || layout.height == 0 {
        return Ok(None);
    }
    // The last byte the rows reach, when no row's place wraps.
    let end = u64::from(layout.height - 1) * u64::from(layout.stride) + u64::from(layout.width) * 4;
    if layout.planes != 3 || layout.line_bytes == 0 || end > out.len() as u64 {
        return decode_planes_bytewise(layout, file, out);
    }
    let (width, line, stride) = (
        layout.width as usize,
        layout.line_bytes as usize,
        layout.stride as usize,
    );
    // The pixels the planes reach (a row's pixels past its plane bytes get their fourth byte only).
    let stored = width.min(line);
    let mut planes = vec![0u8; 3 * line];
    // The file's bytes one after the other, 0 past its end.
    let mut file = file.iter();
    // The byte being repeated and how many more times it comes before the next is read.
    let (mut byte, mut left) = (0u8, 0usize);
    for row in 0..layout.height as usize {
        for plane in planes.chunks_exact_mut(line) {
            let mut places = plane.iter_mut();
            while let Some(place) = places.next() {
                if left == 0 {
                    byte = file.next().copied().unwrap_or(0);
                    if byte < 0xC0 {
                        *place = byte;
                        continue;
                    }
                    // A count of 0 still gives the byte once.
                    left = usize::from(byte - 0xC0).max(1);
                    byte = file.next().copied().unwrap_or(0);
                }
                *place = byte;
                left -= 1;
                // The rest of the run, as far as the plane goes.
                let rest = places.len().min(left);
                places.by_ref().take(rest).for_each(|place| *place = byte);
                left -= rest;
            }
        }
        // A slip kept: a run going on into the next row repeats 0.
        byte = 0;
        let to = &mut out[row * stride..row * stride + width * 4];
        let (first, rest) = planes.split_at(line);
        let (second, third) = rest.split_at(line);
        let (reached, past) = to.split_at_mut(stored * 4);
        for (((pixel, &r), &g), &b) in reached
            .as_chunks_mut::<4>()
            .0
            .iter_mut()
            .zip(first)
            .zip(second)
            .zip(third)
        {
            *pixel = [b, g, r, 0];
        }
        for pixel in past.as_chunks_mut::<4>().0 {
            pixel[3] = 0;
        }
    }
    Ok(Some(()))
}

/// `decode_planes_in` a byte at a time, each place checked: for the layouts whose rows wrap, run
/// past the buffer or have another number of planes than three.
fn decode_planes_bytewise(
    layout: &Layout,
    file: &[u8],
    out: &mut [u8],
) -> Result<Option<()>, crate::Invalid> {
    let mut file = file.iter();
    let past = core::cell::Cell::new(false);
    let done = decode_planes(
        layout,
        || match past.get() {
            true => Err(PAST_BUFFER),
            false => Ok(file.next().copied().unwrap_or(0)),
        },
        |to, byte| match out.get_mut(to as usize) {
            Some(place) => *place = byte,
            None => past.set(true),
        },
    )?;
    if past.get() {
        return Err(PAST_BUFFER);
    }
    Ok(done)
}

/// The header of a screenshot: run coded, 8 bits, 300 dpi both ways, right and bottom the size
/// less one, the 16-colour palette greys 0 .. 15, three planes of `width` bytes.
pub fn screen_header(width: u32, height: u32) -> [u8; HEADER_LEN] {
    let mut h = [0u8; HEADER_LEN];
    h[..4].copy_from_slice(&0x0801_050Au32.to_le_bytes());
    h[8..10].copy_from_slice(&(width.wrapping_sub(1) as u16).to_le_bytes());
    h[0xA..0xC].copy_from_slice(&(height.wrapping_sub(1) as u16).to_le_bytes());
    h[0xC..0x10].copy_from_slice(&0x012C_012Cu32.to_le_bytes());
    for n in 0..16 {
        h[0x10 + 3 * n..0x13 + 3 * n].fill(n as u8);
    }
    h[0x41] = 3;
    h[0x42..0x44].copy_from_slice(&(width as u16).to_le_bytes());
    h
}

/// A screenshot's row as its three planes, red, green, blue (`width` bytes each). `carry` is
/// a value carried from pixel to pixel and row to row: `components(x, carry)`
/// gives the pixel's components (a 16-bit pixel keeps the carry's upper half, a pixel of
/// another size is not read and the carry stays).
pub fn screen_planes(
    width: u32,
    carry: &mut u32,
    components: &mut dyn FnMut(u32, u32) -> u32,
) -> Vec<u8> {
    let w = width as usize;
    let mut planes = vec![0u8; 3 * w];
    for x in 0..w {
        *carry = components(x as u32, *carry);
        planes[x + 2 * w] = *carry as u8;
        *carry >>= 8;
        planes[w + x] = *carry as u8;
        planes[x] = (*carry >> 8) as u8;
    }
    planes
}

/// One line run coded as the screenshot writer does it: runs up to 63 (the counter is
/// checked as it reaches 0x3F, so a run that long is written and the counting starts again
/// from none), a lone byte below 0xC0 as itself. The runs go over the planes' ends. Then
/// the carry's low half as the coding leaves it (low byte the last run's byte, next byte
/// the line's last byte).
pub fn encode_line(line: &[u8], carry: &mut u32) -> Vec<u8> {
    let mut coded = Vec::new();
    let Some(&first) = line.first() else {
        return coded;
    };
    let put = |coded: &mut Vec<u8>, count: u32, run_byte: u8| {
        if count < 1 {
            return;
        }
        if count > 1 || run_byte >= 0xC0 {
            coded.push((count + 0xC0) as u8);
        }
        coded.push(run_byte);
    };
    let (mut run_byte, mut count) = (first, 1u32);
    for &byte in &line[1..] {
        if run_byte == byte {
            count += 1;
            if count >= 0x3F {
                coded.push((count + 0xC0) as u8);
                coded.push(run_byte);
                count = 0;
            }
        } else {
            put(&mut coded, count, run_byte);
            run_byte = byte;
            count = 1;
        }
    }
    put(&mut coded, count, run_byte);
    let last = if line.len() > 1 {
        line[line.len() - 1]
    } else {
        (*carry >> 8) as u8
    };
    *carry = *carry & 0xFFFF_0000 | (last as u32) << 8 | run_byte as u32;
    coded
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
            || {
                Ok::<_, ()>(match file.get(at) {
                    Some(&b) => {
                        at += 1;
                        b
                    }
                    None => 0,
                })
            },
            |to, p| out[to as usize / 4] = p,
        )
        .unwrap()
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
            || {
                Ok::<_, ()>(match file.get(at) {
                    Some(&b) => {
                        at += 1;
                        b
                    }
                    None => 0,
                })
            },
            |to, b| out[to as usize] = b,
        )
        .unwrap()
        .unwrap();
        assert_eq!(out, [5, 3, 1, 0, 6, 4, 2, 0]);
        assert_eq!(
            decode_planes(&Layout::default(), || Ok::<_, ()>(0), |_, _| ()),
            Ok(None)
        );
        // A reader's error ends the decoding.
        assert_eq!(decode_planes(&l, || Err(7), |_, _| ()), Err(7));
    }

    #[test]
    fn encodes_screens() {
        let mut carry = 0;
        let planes = screen_planes(2, &mut carry, &mut |x, _| 0x0010_2030 + x);
        assert_eq!(planes, [0x10, 0x10, 0x20, 0x20, 0x30, 0x31]);
        let coded = encode_line(&planes, &mut carry);
        assert_eq!(coded, [0xC2, 0x10, 0xC2, 0x20, 0x30, 0x31]);
        assert_eq!(carry & 0xFFFF, 0x3131);
        // A run of 64: 63 written at once, then one more.
        let coded = encode_line(&[7u8; 64], &mut carry);
        assert_eq!(coded, [0xFF, 7, 7]);
        // The screenshot's own reader takes back what is written.
        let header = screen_header(2, 1);
        let layout = lay_out(&header, &TARGET);
        let mut at = 0;
        let mut out = vec![0u8; 8];
        let file = encode_line(&planes, &mut carry);
        decode_planes(
            &layout,
            || {
                Ok::<_, ()>(match file.get(at) {
                    Some(&b) => {
                        at += 1;
                        b
                    }
                    None => 0,
                })
            },
            |to, b| out[to as usize] = b,
        )
        .unwrap()
        .unwrap();
        assert_eq!(out, [0x30, 0x20, 0x10, 0, 0x31, 0x20, 0x10, 0]);
    }

    /// A small generator for the comparison tests.
    struct Numbers(u64);
    impl Numbers {
        fn next(&mut self, below: u32) -> u32 {
            self.0 ^= self.0 << 13;
            self.0 ^= self.0 >> 7;
            self.0 ^= self.0 << 17;
            ((self.0 >> 20) % u64::from(below.max(1))) as u32
        }
    }

    /// `decode_planes` with every place checked, as the game ran it: the buffer after, or the
    /// refusal of a pixel past it.
    fn planes_reference(l: &Layout, file: &[u8], len: usize) -> Result<Option<Vec<u8>>, ()> {
        let mut out = vec![0xAAu8; len];
        let mut at = 0;
        let past = core::cell::Cell::new(false);
        let done = decode_planes(
            l,
            || {
                if past.get() {
                    return Err(());
                }
                let b = file.get(at).copied().unwrap_or(0);
                at += 1;
                Ok(b)
            },
            |to, b| match out.get_mut(to as usize) {
                Some(place) => *place = b,
                None => past.set(true),
            },
        )?;
        if past.get() {
            return Err(());
        }
        Ok(done.map(|()| out))
    }

    /// `decode_planes_in` against `decode_planes` for one layout and file.
    fn same_planes(l: &Layout, file: &[u8], len: usize, what: &str) {
        let want = planes_reference(l, file, len);
        let mut out = vec![0xAAu8; len];
        let got = decode_planes_in(l, file, &mut out);
        match (want, got) {
            (Ok(Some(want)), Ok(Some(()))) => assert!(want == out, "{what}: other pixels, {l:?}"),
            (Ok(None), Ok(None)) | (Err(()), Err(_)) => {}
            (want, got) => panic!(
                "{what}: {:?} against {got:?}, {l:?}",
                want.map(|w| w.is_some())
            ),
        }
    }

    /// The decoder on a held file gives what the byte-by-byte one gives: runs crossing the ends of
    /// planes and rows, counts of 0, odd widths, rows wider or narrower than their plane bytes,
    /// rows that overlap or lie apart, files that end early, pixels past the buffer.
    #[test]
    fn decodes_planes_in_memory_as_byte_by_byte() {
        let mut n = Numbers(0x1234_5678_9ABC_DEF1);
        for case in 0..4000 {
            let planes = if n.next(20) == 0 { 1 + n.next(4) } else { 3 };
            let line_bytes = n.next(40);
            let width = n.next(40);
            let height = n.next(12);
            let stride = match n.next(6) {
                0 => n.next(200),
                1 => 0xFFFF_FFF0 + n.next(16),
                _ => width * 4 + n.next(9),
            };
            let l = Layout {
                planes,
                line_bytes,
                width,
                height,
                stride,
                ..Layout::default()
            };
            // Mostly counts, mostly plain bytes, or anything; sometimes too short a file.
            let kind = n.next(3);
            let full = (planes * line_bytes.max(1) * height) as usize;
            let len = match n.next(4) {
                0 => n.next(full as u32 + 1) as usize,
                _ => full + 8,
            };
            let file: Vec<u8> = (0..len)
                .map(|_| match kind {
                    0 => 0xC0 + n.next(0x40) as u8,
                    1 => n.next(0xC8) as u8,
                    _ => n.next(0x100) as u8,
                })
                .collect();
            let room = match n.next(5) {
                0 => n.next(64) as usize,
                _ => height as usize * (width as usize * 4 + 8) + 16,
            };
            same_planes(&l, &file, room, &format!("case {case}"));
        }
    }

    /// Every PCX file of three planes under the folder `RC2K_PCX_FILES` names (a game's `files`
    /// folder; nothing is checked when it is not set) decodes the same both ways, as a texture, as
    /// a screen and into a buffer of its own rows.
    #[test]
    fn decodes_the_installed_pictures_as_byte_by_byte() {
        let Some(root) = std::env::var_os("RC2K_PCX_FILES") else {
            return;
        };
        let mut folders = vec![std::path::PathBuf::from(root)];
        let mut checked = 0;
        while let Some(folder) = folders.pop() {
            for entry in std::fs::read_dir(&folder).unwrap() {
                let path = entry.unwrap().path();
                if path.is_dir() {
                    folders.push(path);
                    continue;
                }
                let is_pcx = path
                    .extension()
                    .is_some_and(|e| e.eq_ignore_ascii_case("pcx"));
                let bytes = if is_pcx {
                    std::fs::read(&path).unwrap()
                } else {
                    Vec::new()
                };
                let Some((header, rows)) = bytes.split_first_chunk::<HEADER_LEN>() else {
                    continue;
                };
                if kind(header) != Some(Kind::Planes) {
                    continue;
                }
                let targets = [
                    (256, 256, 0xFFFF_FFFE),
                    (640, 480, 0xFFFF_FFFF),
                    (800, 600, 0xFFFF_FFFD),
                    (100, 37, 0xFFFF_FFFE),
                ];
                for (width, height, row_bytes) in targets {
                    let target = Target {
                        width,
                        height,
                        width_limit: 800,
                        height_limit: 600,
                        flags: 0x800,
                        row_bytes,
                    };
                    let l = lay_out(header, &target);
                    let room = (l.stride * l.height.max(1)) as usize;
                    let what = path.display().to_string();
                    same_planes(&l, rows, room, &what);
                    // Cut short: the rest reads as 0.
                    same_planes(&l, &rows[..rows.len() / 3], room, &what);
                }
                checked += 1;
            }
        }
        println!("{checked} pictures of three planes checked");
        assert!(
            checked > 0,
            "no PCX file of three planes under RC2K_PCX_FILES"
        );
    }
}
