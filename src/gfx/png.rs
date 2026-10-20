//! What the PNG reader's parts share: the block the decoder describes a picture in, and PNG
//! files for their self-tests.

use crate::framework::rva;

/// `buf` (0x93E8B0): first the file's first 0x80 bytes, then what the decoder says about
/// the picture, a `PngInfo`.
pub const INFO_RVA: usize = rva(0x93E8B0);

/// What `decode_png` leaves behind. Only the first byte of `has_palette` is written, and the
/// palette only when the file has one: the rest is whatever was there (in the original, the
/// work area of zlib, which it lays over this block).
#[repr(C)]
pub struct PngInfo {
    /// 1 if the file has a palette (a PLTE chunk), else 0.
    pub has_palette: u8,
    pub unset: [u8; 3],
    pub width: u32,
    pub height: u32,
    /// Three bytes an entry.
    pub palette: [u8; 0x300],
}

pub use rc2k_core::png::SIGNATURE;

// ---- for the self-tests ----

pub const TEST_FILE: &str = "var\\rc2k_hook_test.png";
pub const TEST_PATH: &[u8] = b"var\\rc2k_hook_test.png\0";
pub const MISSING_PATH: &[u8] = b"var\\rc2k_hook_no_such.png\0";

fn chunk(kind: &[u8; 4], data: &[u8]) -> Vec<u8> {
    let mut out = (data.len() as u32).to_be_bytes().to_vec();
    out.extend(kind);
    out.extend(data);
    // The check sum, which the game does not look at.
    out.extend([0u8; 4]);
    out
}

/// The paeth predictor, as the standard has it.
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

/// A PNG file of `width` x `height` pixels of `bpp` bytes (3 = colour type 2, 1 = colour
/// type 3 with a palette), whose row n is coded with filter `filters[n % filters.len()]`.
/// Returns the file and the pixel bytes it stands for.
pub fn test_png(
    width: u32,
    height: u32,
    bpp: usize,
    filters: &[u8],
    seed: u32,
) -> (Vec<u8>, Vec<u8>) {
    let row_bytes = width as usize * bpp;
    let mut value = seed;
    let pixels: Vec<u8> = (0..row_bytes * height as usize)
        .map(|n| {
            value = value.wrapping_mul(0x0019_660D).wrapping_add(0x3C6E_F35F);
            // Smooth here and there, so that the filters have something to do.
            if n % 11 < 5 {
                (n / 3) as u8
            } else {
                (value >> 24) as u8
            }
        })
        .collect();
    let mut raw = Vec::new();
    for row in 0..height as usize {
        let filter = filters[row % filters.len()];
        raw.push(filter);
        let at = |r: usize, i: usize| pixels[r * row_bytes + i];
        for i in 0..row_bytes {
            let x = at(row, i);
            let a = if i >= bpp { at(row, i - bpp) } else { 0 };
            let b = if row > 0 { at(row - 1, i) } else { 0 };
            let c = if row > 0 && i >= bpp {
                at(row - 1, i - bpp)
            } else {
                0
            };
            raw.push(match filter {
                1 => x.wrapping_sub(a),
                2 => x.wrapping_sub(b),
                3 => x.wrapping_sub(((a as u32 + b as u32) / 2) as u8),
                4 => x.wrapping_sub(paeth(a, b, c)),
                _ => x,
            });
        }
    }
    let mut header = Vec::new();
    header.extend(width.to_be_bytes());
    header.extend(height.to_be_bytes());
    header.extend([8, if bpp == 3 { 2 } else { 3 }, 0, 0, 0]);
    let mut file = SIGNATURE.to_vec();
    file.extend(chunk(b"IHDR", &header));
    // A chunk the game skips, long enough that the file's first 0x80 bytes are all there.
    file.extend(chunk(b"tEXt", &[b'x'; 0x60]));
    if bpp == 1 {
        let palette: Vec<u8> = (0..0x300u32).map(|n| (n * 7 + 3) as u8).collect();
        file.extend(chunk(b"PLTE", &palette));
    }
    file.extend(chunk(
        b"IDAT",
        &miniz_oxide::deflate::compress_to_vec_zlib(&raw, 6),
    ));
    file.extend(chunk(b"IEND", &[]));
    (file, pixels)
}

pub fn write_test_file(content: &[u8]) -> bool {
    std::fs::write(TEST_FILE, content).is_ok()
}

pub fn remove_test_file() {
    let _ = std::fs::remove_file(TEST_FILE);
}
