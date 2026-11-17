//! PNG files as the screenshots are written: 8-bit colour (type 2), no interlacing, every row
//! unfiltered, the pixel data deflated (`miniz_oxide`).

use miniz_oxide::deflate::compress_to_vec_zlib;

const SIGNATURE: [u8; 8] = [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];

/// The CRC-32 the chunks carry (over a chunk's type and data).
fn crc32(parts: &[&[u8]]) -> u32 {
    let mut crc = 0xFFFF_FFFFu32;
    for &byte in parts.iter().flat_map(|part| part.iter()) {
        crc ^= u32::from(byte);
        for _ in 0..8 {
            crc = if crc & 1 != 0 {
                (crc >> 1) ^ 0xEDB8_8320
            } else {
                crc >> 1
            };
        }
    }
    !crc
}

fn chunk(out: &mut Vec<u8>, kind: &[u8; 4], data: &[u8]) {
    out.extend_from_slice(&(data.len() as u32).to_be_bytes());
    out.extend_from_slice(kind);
    out.extend_from_slice(data);
    out.extend_from_slice(&crc32(&[kind, data]).to_be_bytes());
}

/// A PNG of `width` x `height` pixels from `rgb`, three bytes (red, green, blue) a pixel, rows
/// from the top without padding. `rgb` must hold exactly `3 * width * height` bytes.
pub fn encode_rgb(width: u32, height: u32, rgb: &[u8]) -> Vec<u8> {
    let row = 3 * width as usize;
    assert_eq!(rgb.len(), row * height as usize, "the pixels' size");
    let mut raw = Vec::with_capacity((row + 1) * height as usize);
    for line in rgb.chunks(row.max(1)).take(height as usize) {
        raw.push(0);
        raw.extend_from_slice(line);
    }
    let mut head = Vec::with_capacity(13);
    head.extend_from_slice(&width.to_be_bytes());
    head.extend_from_slice(&height.to_be_bytes());
    head.extend_from_slice(&[8, 2, 0, 0, 0]);
    let mut out = SIGNATURE.to_vec();
    chunk(&mut out, b"IHDR", &head);
    chunk(&mut out, b"IDAT", &compress_to_vec_zlib(&raw, 6));
    chunk(&mut out, b"IEND", &[]);
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use miniz_oxide::inflate::decompress_to_vec_zlib;

    #[test]
    fn the_check_sum_is_the_standard_one() {
        assert_eq!(crc32(&[b"123456789"]), 0xCBF4_3926);
        assert_eq!(crc32(&[b"IEND"]), 0xAE42_6082);
    }

    #[test]
    fn a_picture_reads_back() {
        let rgb: Vec<u8> = (0..3 * 5 * 3).map(|n: u8| n.wrapping_mul(7)).collect();
        let png = encode_rgb(5, 3, &rgb);
        assert_eq!(&png[..8], &SIGNATURE);
        assert_eq!(&png[12..16], b"IHDR");
        assert_eq!(u32::from_be_bytes(png[16..20].try_into().unwrap()), 5);
        assert_eq!(u32::from_be_bytes(png[20..24].try_into().unwrap()), 3);
        let idat = 33;
        let len = u32::from_be_bytes(png[idat..idat + 4].try_into().unwrap()) as usize;
        assert_eq!(&png[idat + 4..idat + 8], b"IDAT");
        let raw = decompress_to_vec_zlib(&png[idat + 8..idat + 8 + len]).unwrap();
        assert_eq!(raw.len(), 3 * 16);
        for (y, line) in raw.chunks(16).enumerate() {
            assert_eq!(line[0], 0);
            assert_eq!(&line[1..], &rgb[15 * y..15 * y + 15]);
        }
        assert_eq!(&png[png.len() - 12..png.len() - 8], &[0, 0, 0, 0]);
        assert_eq!(&png[png.len() - 8..png.len() - 4], b"IEND");
    }
}
