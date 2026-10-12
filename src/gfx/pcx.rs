//! What the PCX readers share: the header and palette they read, the layout they work out
//! from it, and PCX files for their self-tests.
//!
//! A PCX file here: a header of 0x80 bytes (+4 .. +0xA the picture's left, top, right and
//! bottom as words; +0x41 the number of planes, 1 or 3; +0x42 the bytes of one plane of one
//! row as a word), then the rows, run-length coded: a byte of 0xC0 or more is a count (less
//! 0xC0) for the byte after it, any other byte stands for itself. With one plane a byte is
//! a palette number and the palette, 0x300 bytes, is the end of the file.

use crate::files::stream::Stream;
use crate::framework::{game_addr, game_ptr, game_read, rva};
use crate::gfx::picture::{FLAGS_RVA, HEIGHT_RVA, PICTURE_RVA, ROW_BYTES_RVA, WIDTH_RVA};
use crate::service::harness::{Region, at};

/// dword_51B5F4: the header as read from the file, 0x80 bytes.
pub const HEADER_RVA: usize = rva(0x51B5F4);
pub const HEADER_LEN: usize = 0x80;
pub const HEADER_LEFT: usize = 4;
pub const HEADER_TOP: usize = 6;
pub const HEADER_RIGHT: usize = 8;
pub const HEADER_BOTTOM: usize = 0xA;
pub const HEADER_PLANES: usize = 0x41;
pub const HEADER_LINE_BYTES: usize = 0x42;
/// unk_51B674: the palette as read from the file, three bytes an entry.
pub const FILE_PALETTE_RVA: usize = rva(0x51B674);
pub const FILE_PALETTE_LEN: usize = 0x300;
/// dword_5195E4: the stream the readers open the file as.
pub const STREAM_RVA: usize = rva(0x5195E4);

// The layout: what a decoded picture is, for the readers and for what comes after them.
pub const PLANES_RVA: usize = rva(0x5195E0); // dword_5195E0
pub const LINE_BYTES_RVA: usize = rva(0x5195DC); // dword_5195DC
pub const FILE_WIDTH_RVA: usize = rva(0x5195D4); // dword_5195D4: as the header says
pub const FILE_HEIGHT_RVA: usize = rva(0x5195D8); // dword_5195D8
pub const DECODED_WIDTH_RVA: usize = rva(0x519124); // dword_519124: cut to what fits
pub const DECODED_HEIGHT_RVA: usize = rva(0x519128); // dword_519128
pub const PIXEL_BITS_RVA: usize = rva(0x51912C); // dword_51912C: 8 or 32
pub const PIXEL_BYTES_RVA: usize = rva(0x519130); // dword_519130
pub const DECODED_ROW_BYTES_RVA: usize = rva(0x519134); // dword_519134
pub const DECODED_ROW_DWORDS_RVA: usize = rva(0x519138); // dword_519138
pub const STRIDE_RVA: usize = rva(0x51913C); // dword_51913C: from one row to the next
pub const PIXELS_RVA: usize = rva(0x519140); // dword_519140: where the rows go
pub const ROW_GAP_RVA: usize = rva(0x519144); // dword_519144: the stride less a row's bytes
pub const PALETTE_RVA: usize = rva(0x519154); // dword_519154: 256 dwords
/// dword_51B974, dword_51B978: the widest and highest picture that is taken.
pub const WIDTH_LIMIT_RVA: usize = rva(0x51B974);
pub const HEIGHT_LIMIT_RVA: usize = rva(0x51B978);

pub fn stream() -> *mut Stream {
    game_ptr(STREAM_RVA)
}

/// Everything the layout step writes.
pub fn layout_regions() -> Vec<Region> {
    let mut regions: Vec<Region> = [
        PLANES_RVA,
        LINE_BYTES_RVA,
        FILE_WIDTH_RVA,
        FILE_HEIGHT_RVA,
        DECODED_WIDTH_RVA,
        DECODED_HEIGHT_RVA,
        PIXEL_BITS_RVA,
        PIXEL_BYTES_RVA,
        DECODED_ROW_BYTES_RVA,
        DECODED_ROW_DWORDS_RVA,
        STRIDE_RVA,
        PIXELS_RVA,
        ROW_GAP_RVA,
    ]
    .map(at)
    .to_vec();
    regions.push((game_addr(PALETTE_RVA) as u32, 0x400));
    regions
}

/// Everything a reader writes besides the pixels: the layout, the header, the file's
/// palette, the picture's flags and the head of the stream.
pub fn reader_regions() -> Vec<Region> {
    let mut regions = layout_regions();
    regions.extend([
        (game_addr(HEADER_RVA) as u32, HEADER_LEN),
        (game_addr(FILE_PALETTE_RVA) as u32, FILE_PALETTE_LEN),
        at(FLAGS_RVA),
        (game_addr(STREAM_RVA) as u32, 16),
    ]);
    regions
}

/// What the readers and the layout step read without writing it: the picture's size and
/// row length, the buffer, the two limits.
pub fn reader_inputs() -> [Region; 6] {
    [
        WIDTH_RVA,
        HEIGHT_RVA,
        ROW_BYTES_RVA,
        PICTURE_RVA,
        WIDTH_LIMIT_RVA,
        HEIGHT_LIMIT_RVA,
    ]
    .map(at)
}

/// The decoded rows as they lie now: from the first row's start over `height` strides.
/// Empty if the numbers make no block of memory.
pub fn decoded_region() -> Vec<Region> {
    let (pixels, stride, height): (u32, u32, u32) = unsafe {
        (
            game_read(PIXELS_RVA),
            game_read(STRIDE_RVA),
            game_read(DECODED_HEIGHT_RVA),
        )
    };
    let bytes = stride as u64 * height as u64;
    if pixels == 0 || bytes == 0 || bytes > 0x100_0000 {
        return Vec::new();
    }
    vec![(pixels, bytes as usize)]
}

// ---- for the self-tests: PCX files of the test's own in the game's `var` folder ----

pub const TEST_FILE: &str = "var\\rc2k_hook_test.pcx";
pub const TEST_PATH: &[u8] = b"var\\rc2k_hook_test.pcx\0";
pub const MISSING_PATH: &[u8] = b"var\\rc2k_hook_no_such.pcx\0";

/// A header for a picture `width` x `height` at (left, top).
pub fn test_header(
    left: u16,
    top: u16,
    width: u16,
    height: u16,
    planes: u8,
    line_bytes: u16,
) -> Vec<u8> {
    let mut header = vec![0u8; HEADER_LEN];
    header[0] = 0x0A;
    header[1] = 5;
    header[2] = 1;
    header[3] = 8;
    header[HEADER_LEFT..][..2].copy_from_slice(&left.to_le_bytes());
    header[HEADER_TOP..][..2].copy_from_slice(&top.to_le_bytes());
    header[HEADER_RIGHT..][..2]
        .copy_from_slice(&left.wrapping_add(width).wrapping_sub(1).to_le_bytes());
    header[HEADER_BOTTOM..][..2]
        .copy_from_slice(&top.wrapping_add(height).wrapping_sub(1).to_le_bytes());
    header[HEADER_PLANES] = planes;
    header[HEADER_LINE_BYTES..][..2].copy_from_slice(&line_bytes.to_le_bytes());
    header
}

/// Run-length codes `bytes` the way PCX files are written: runs of up to 63, and a count
/// before every byte of 0xC0 or more. Runs go on across the ends of rows, which the readers
/// have to cope with.
pub fn test_runs(bytes: &[u8]) -> Vec<u8> {
    let mut coded = Vec::new();
    let mut n = 0;
    while n < bytes.len() {
        let byte = bytes[n];
        let run = bytes[n..]
            .iter()
            .take(63)
            .take_while(|&&b| b == byte)
            .count();
        if run > 1 || byte >= 0xC0 {
            coded.push(0xC0 + run as u8);
        }
        coded.push(byte);
        n += run;
    }
    coded
}

/// 256 entries of three bytes.
pub fn test_palette() -> Vec<u8> {
    (0..FILE_PALETTE_LEN as u32)
        .map(|n| (n.wrapping_mul(0x53) >> 2) as u8 ^ (n >> 3) as u8)
        .collect()
}

/// Plane bytes for a test picture: runs of several lengths and bytes on both sides of 0xC0.
pub fn test_plane_bytes(count: usize, seed: u32) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(count);
    let mut value = seed;
    while bytes.len() < count {
        value = value.wrapping_mul(0x0019_660D).wrapping_add(0x3C6E_F35F);
        let byte = (value >> 24) as u8;
        let run = 1 + (value >> 8) as usize % 5 * ((value >> 16) as usize % 3);
        for _ in 0..run.min(count - bytes.len()) {
            bytes.push(byte);
        }
    }
    bytes
}

pub fn write_test_file(content: &[u8]) -> bool {
    std::fs::write(TEST_FILE, content).is_ok()
}

pub fn remove_test_file() {
    let _ = std::fs::remove_file(TEST_FILE);
}
