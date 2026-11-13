//! PNG files as the game reads them.
//!
//! The chunks are walked to IEND. IHDR gives the size and the colour type; the IDAT chunk is
//! inflated (`miniz_oxide`) straight into the pixel area, `(bytes a pixel * width + 1) * height`
//! bytes at most, and its rows are unfiltered in place and closed up, as three bytes a pixel for
//! colour type 2 and one byte a pixel for every other type; a PLTE chunk is remembered. Check sums
//! are not looked at. For the first row the "row above" of filters 2, 3 and 4 comes from the
//! caller, which gives zeros as the PNG standard has it (fixed: "A PNG picture's first row read
//! against stray memory" in wiki/Fixed-bugs.md).

use crate::Invalid;
use miniz_oxide::inflate::TINFLStatus;
use miniz_oxide::inflate::core::{DecompressorOxide, decompress, inflate_flags};

pub const SIGNATURE: [u8; 8] = [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];

/// Bytes a pixel by colour type.
pub const PIXEL_BYTES: [u32; 8] = [1, 0, 3, 1, 2, 0, 4, 0];

/// What IHDR says.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Header {
    pub width: u32,
    pub height: u32,
    /// The low three bits of the colour type.
    pub colour_type: u32,
    /// The byte after the colour type.
    pub compression: u8,
}

/// Where the chunks the game uses are: the header, the palette's start and the data's
/// start and length.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct Chunks {
    pub header: Option<Header>,
    pub palette: Option<usize>,
    pub data: Option<(usize, usize)>,
}

fn be(file: &[u8], at: usize) -> u32 {
    u32::from_be_bytes([file[at], file[at + 1], file[at + 2], file[at + 3]])
}

/// The bytes of a file's head looked at before it is read.
pub const HEAD_LEN: usize = 0x80;

/// Whether a file's head is a PNG the game takes: the signature, an IHDR chunk of 13 bytes,
/// at most 800 x 600, not interlaced.
pub fn head_taken(head: &[u8; HEAD_LEN]) -> bool {
    let be = |at: usize| u32::from_be_bytes(head[at..at + 4].try_into().unwrap());
    head[..8] == SIGNATURE
        && be(8) == 13
        && &head[12..16] == b"IHDR"
        && be(0x10) <= 800
        && be(0x14) <= 600
        && head[0x1C] == 0
}

/// The chunks of a file, walked to IEND. `Ok(None)` if the file does not begin as a PNG. Refused:
/// a file shorter than the signature, a chunk past the end, a header out of place, a bit depth
/// other than 8, more than one data chunk or one before the header, a palette past the end.
pub fn chunks(file: &[u8]) -> Result<Option<Chunks>, Invalid> {
    if file.len() < 8 {
        return Err(Invalid("a PNG file shorter than its signature"));
    }
    if file[..8] != SIGNATURE {
        return Ok(None);
    }
    let mut found = Chunks::default();
    let mut at = 8usize;
    loop {
        if at + 8 > file.len() {
            return Err(Invalid("a PNG chunk past the end of the file"));
        }
        let length = be(file, at) as usize;
        let kind = &file[at + 4..at + 8];
        let start = at + 8;
        if start.checked_add(length).is_none_or(|end| end > file.len()) {
            return Err(Invalid("a PNG chunk past the end of the file"));
        }
        match kind {
            b"IHDR" => {
                if length < 11 || found.data.is_some() {
                    return Err(Invalid("a PNG header out of place"));
                }
                if file[start + 8] != 8 {
                    return Err(Invalid("a PNG bit depth other than 8"));
                }
                found.header = Some(Header {
                    width: be(file, start),
                    height: be(file, start + 4),
                    colour_type: file[start + 9] as u32 & 7,
                    compression: file[start + 10],
                });
            }
            b"PLTE" => found.palette = Some(start),
            b"IDAT" => {
                if found.data.is_some() || found.header.is_none() {
                    return Err(Invalid(
                        "more than one PNG data chunk, or one before the header",
                    ));
                }
                found.data = Some((start, length));
            }
            _ => {}
        }
        if kind == b"IEND" {
            break;
        }
        at = start + length + 4;
    }
    if found
        .palette
        .is_some_and(|start| start + 0x300 > file.len())
    {
        return Err(Invalid("a PNG palette past the end of the file"));
    }
    Ok(Some(found))
}

/// The bytes the pixel area must hold for a picture with data. Refused: colour types 4 and 6
/// (they would be taken as one byte a pixel), a width under 2 or a height of 0 (the counts would
/// wrap), more than `room`.
pub fn area_for(header: &Header, room: usize) -> Result<usize, Invalid> {
    if !matches!(header.colour_type, 0 | 2 | 3) {
        return Err(Invalid(
            "a PNG colour type the original takes for one byte a pixel",
        ));
    }
    if header.width < 2 || header.height == 0 {
        return Err(Invalid("a PNG picture without width or height"));
    }
    let pixel_bytes = PIXEL_BYTES[header.colour_type as usize] as u64;
    let bytes = (pixel_bytes * header.width as u64 + 1) * header.height as u64;
    if bytes > room as u64 {
        return Err(Invalid("a PNG picture too big for the picture buffer"));
    }
    Ok(bytes as usize)
}

/// The data inflated into `out` (its length the area's). Taken: done, the area full, or all there
/// but for zlib's check sum; refused otherwise.
pub fn inflate(data: &[u8], out: &mut [u8]) -> Result<(), Invalid> {
    let mut inflater = DecompressorOxide::new();
    let flags = inflate_flags::TINFL_FLAG_PARSE_ZLIB_HEADER
        | inflate_flags::TINFL_FLAG_USING_NON_WRAPPING_OUTPUT_BUF;
    let (status, _, _) = decompress(&mut inflater, data, out, 0, flags);
    if matches!(
        status,
        TINFLStatus::Done | TINFLStatus::HasMoreOutput | TINFLStatus::Adler32Mismatch
    ) {
        Ok(())
    } else {
        Err(Invalid("PNG data that does not inflate cleanly"))
    }
}

/// A row's bytes and a pixel's step for the filters: three bytes a pixel for colour type
/// 2, else one.
pub fn row_layout(header: &Header) -> (usize, usize) {
    if header.colour_type == 2 {
        (header.width as usize * 3, 3)
    } else {
        (header.width as usize, 1)
    }
}

fn paeth(a: u8, b: u8, c: u8) -> u8 {
    let p = a as i32 + b as i32 - c as i32;
    let (pa, pb, pc) = (
        (p - a as i32).abs(),
        (p - b as i32).abs(),
        (p - c as i32).abs(),
    );
    if pa <= pb && pa <= pc {
        a
    } else if pb <= pc {
        b
    } else {
        c
    }
}

/// The filter bytes taken out of `rows` rows of `area` and the filters undone, in place. The row
/// above the first is `first_above(i)` for its byte `i` (zeros for a standard PNG), asked for
/// only by the filters that read it (2, 3, 4). Refused: a filter above 4, and rows past the area.
pub fn unfilter(
    area: &mut [u8],
    row_bytes: usize,
    rows: u32,
    step: usize,
    first_above: &dyn Fn(usize) -> u8,
) -> Result<(), Invalid> {
    if (row_bytes + 1) * rows as usize > area.len() {
        return Err(Invalid("PNG rows past their buffer"));
    }
    let (mut from, mut to) = (0usize, 0usize);
    for row in 0..rows {
        let filter = area[from];
        from += 1;
        if filter > 4 {
            return Err(Invalid("a PNG row filter above 4"));
        }
        for i in 0..row_bytes {
            let raw = area[from + i];
            let up = |area: &[u8], i: usize| {
                if row == 0 {
                    first_above(i)
                } else {
                    area[to - row_bytes + i]
                }
            };
            let value = match filter {
                0 => raw,
                1 => raw.wrapping_add(if i >= step { area[to + i - step] } else { 0 }),
                2 => raw.wrapping_add(up(area, i)),
                3 => {
                    let up = up(area, i);
                    let predicted = if i >= step {
                        ((up as u32 + area[to + i - step] as u32) >> 1) as u8
                    } else {
                        up >> 1
                    };
                    raw.wrapping_add(predicted)
                }
                _ => {
                    let up_here = up(area, i);
                    let predicted = if i >= step {
                        paeth(area[to + i - step], up_here, up(area, i - step))
                    } else {
                        paeth(0, up_here, 0)
                    };
                    raw.wrapping_add(predicted)
                }
            };
            area[to + i] = value;
        }
        from += row_bytes;
        to += row_bytes;
    }
    Ok(())
}

/// The layout variables for a decoded PNG picture: rows without a gap, 8
/// bits a pixel when bit 8 of the picture flags is set, else 32. (width, height,
/// pixel_bits, pixel_bytes, row_bytes, row_dwords, stride, row_gap.)
pub fn layout(width: u32, height: u32, flags: u32) -> [u32; 8] {
    let bits: u32 = if flags & 0x100 != 0 { 8 } else { 32 };
    let row_bytes = (bits >> 3).wrapping_mul(width);
    [
        width,
        height,
        bits,
        bits >> 3,
        row_bytes,
        row_bytes >> 2,
        row_bytes,
        0,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn chunk(kind: &[u8; 4], data: &[u8]) -> Vec<u8> {
        let mut c = (data.len() as u32).to_be_bytes().to_vec();
        c.extend(kind);
        c.extend(data);
        c.extend([0; 4]); // the check sum, not looked at
        c
    }

    #[test]
    fn decodes() {
        // 2 x 2, colour type 2: row 1 filter 0, row 2 filter 2 (up) adding 1 to each byte.
        let raw = [0u8, 1, 2, 3, 4, 5, 6, 2, 1, 1, 1, 1, 1, 1];
        let mut ihdr = vec![];
        ihdr.extend(2u32.to_be_bytes());
        ihdr.extend(2u32.to_be_bytes());
        ihdr.extend([8, 2, 0, 0, 0]);
        let mut file = SIGNATURE.to_vec();
        file.extend(chunk(b"IHDR", &ihdr));
        file.extend(chunk(
            b"IDAT",
            &miniz_oxide::deflate::compress_to_vec_zlib(&raw, 6),
        ));
        file.extend(chunk(b"IEND", &[]));
        let found = chunks(&file).unwrap().unwrap();
        let header = found.header.unwrap();
        assert_eq!((header.width, header.height, header.colour_type), (2, 2, 2));
        let area = area_for(&header, 1000).unwrap();
        assert_eq!(area, 14);
        let (row_bytes, step) = row_layout(&header);
        let mut buffer = vec![0u8; area];
        let (start, length) = found.data.unwrap();
        inflate(&file[start..start + length], &mut buffer).unwrap();
        // Filter 0 first: the row above it is never asked for.
        unfilter(&mut buffer, row_bytes, 2, step, &|_| unreachable!()).unwrap();
        assert_eq!(&buffer[..12], &[1, 2, 3, 4, 5, 6, 2, 3, 4, 5, 6, 7]);
        // Filter 2 first: the row before the area.
        let mut up = [2u8, 0, 1, 2, 3, 4, 5];
        unfilter(&mut up, 6, 1, 3, &|i| i as u8 * 10).unwrap();
        assert_eq!(&up[..6], &[0, 11, 22, 33, 44, 55]);
        assert_eq!(chunks(b"not a png file"), Ok(None));
        assert!(area_for(&Header { width: 1, ..header }, 1000).is_err());
        assert_eq!(layout(4, 3, 0), [4, 3, 32, 4, 16, 4, 16, 0]);
    }
}
