//! `gfx::decode_png` (0x4D3040): decodes a PNG file held in memory.
//!
//! In (on the stack, a C function): the file, where the pixels go, the `PngInfo` to fill
//! Out: eax = 0, or 3 if the info block lies inside the pixels, 1 if an argument is null, 2
//!      if the file does not begin as a PNG; ecx, edx whatever; the rest preserved
//! The original walks the chunks to IEND, trusting every length. IHDR gives the width, the
//! height and the colour type; an IDAT chunk is inflated (zlib 1.1.3) straight into the
//! pixel area, `(bytes a pixel * width + 1) * height` bytes at most, and its rows are then
//! unfiltered in place and closed up, as three bytes a pixel for colour type 2 and as one
//! byte a pixel for every other type; a PLTE chunk is remembered. At IEND the info block
//! gets 0 or 1 in its first byte, the width and the height, and with a palette 0x300 bytes
//! from the PLTE chunk. Check sums are not looked at, nor is what the inflating returns.
//! For the first row the "row above" of filters 2, 3 and 4 is the memory before the pixel
//! area, not zeros as the PNG standard has it; the Rust version reads the same bytes.
//! The Rust version inflates with the `miniz_oxide` crate and does the rest as the original
//! does. What it does not do: use the info block and what follows it as zlib's work area
//! (the original's zlib takes its memory from there), so that memory is left alone; and
//! keep zlib's own variables (0xA8C1A4 on).
//! One caller, `gfx::read_png`. Every PNG file of the game is of colour type 2 with 8 bits
//! and one IDAT chunk.
//! Refuses, and leaves to the original: a file whose length is not known (the function is
//! not told it; `read_png` keeps it in `dword_5BEC34`); a chunk that runs past the end;
//! more than one IDAT chunk, or one before IHDR; a bit depth other than 8; colour types 4
//! and 6 (the original takes them for one byte a pixel); a width under 2 or a height of 0
//! (the original counts through all 32 bits); a row filter above 4; data that does not
//! inflate cleanly; a picture too big for the picture buffer. The name is a reading.

use crate::errors::BadInput;
use crate::framework::{RawHook, Regs, SelfTest, game_read, game_write, hook_asm, rva, shadow};
use crate::gfx::picture::PICTURE_DWORDS;
use crate::gfx::png::{PngInfo, SIGNATURE, test_png};
use miniz_oxide::inflate::TINFLStatus;
use miniz_oxide::inflate::core::{DecompressorOxide, decompress, inflate_flags};
use std::sync::atomic::{AtomicUsize, Ordering};

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("gfx::decode_png", rva(0x4D3040), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// A C function takes its arguments on the stack, so its original is called as one (see
// `dispatch`) and not through the thunk's companion, which only loads registers.
const _: unsafe extern "C" fn(*mut Regs) = call_original;

// ---- game data ----

const WIDTH_RVA: usize = rva(0xA8C190); // dword_A8C190
const HEIGHT_RVA: usize = rva(0xA8C194); // dword_A8C194
const COMPRESSION_RVA: usize = rva(0xA8C19A); // byte_A8C19A: the header's byte after the colour type
const COLOUR_TYPE_RVA: usize = rva(0xA8C1A0); // dword_A8C1A0
/// dword_5BEC30, dword_5BEC34: the file `read_png` holds in memory, and its length.
pub const FILE_RVA: usize = rva(0x5BEC30);
pub const FILE_LEN_RVA: usize = rva(0x5BEC34);

/// dword_4DEE88: bytes a pixel by colour type.
const PIXEL_BYTES: [u32; 8] = [1, 0, 3, 1, 2, 0, 4, 0];

// ---- Rust version ----

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

/// Takes the filter bytes out of `rows` rows at `area` and undoes the filters, in place.
/// `step` is the bytes of one pixel as the filters see it. The row above the first is the
/// memory before `area`.
unsafe fn unfilter(
    area: *mut u8,
    row_bytes: usize,
    rows: u32,
    step: usize,
) -> Result<(), BadInput> {
    unsafe {
        let (mut from, mut to) = (area as *const u8, area);
        for _ in 0..rows {
            let filter = from.read();
            from = from.add(1);
            let above = to.wrapping_sub(row_bytes) as *const u8;
            match filter {
                0 => std::ptr::copy(from, to, row_bytes),
                1 => {
                    for i in 0..row_bytes {
                        let left = if i >= step {
                            to.add(i - step).read()
                        } else {
                            0
                        };
                        to.add(i).write(from.add(i).read().wrapping_add(left));
                    }
                }
                2 => {
                    for i in 0..row_bytes {
                        to.add(i)
                            .write(from.add(i).read().wrapping_add(above.add(i).read()));
                    }
                }
                3 => {
                    for i in 0..row_bytes {
                        let up = above.add(i).read();
                        let predicted = if i >= step {
                            ((up as u32 + to.add(i - step).read() as u32) >> 1) as u8
                        } else {
                            up >> 1
                        };
                        to.add(i).write(from.add(i).read().wrapping_add(predicted));
                    }
                }
                4 => {
                    for i in 0..row_bytes {
                        let up = above.add(i).read();
                        let predicted = if i >= step {
                            paeth(to.add(i - step).read(), up, above.add(i - step).read())
                        } else {
                            paeth(0, up, 0)
                        };
                        to.add(i).write(from.add(i).read().wrapping_add(predicted));
                    }
                }
                _ => return Err(BadInput("a PNG row filter above 4")),
            }
            from = from.add(row_bytes);
            to = to.add(row_bytes);
        }
        Ok(())
    }
}

fn be(bytes: &[u8]) -> u32 {
    u32::from_be_bytes(bytes[..4].try_into().unwrap())
}

/// Returns what the original returns in eax.
pub unsafe fn decode_png(
    file: &[u8],
    pixels: *mut u8,
    info: *mut PngInfo,
) -> Result<u32, BadInput> {
    if pixels.is_null() || info.is_null() {
        return Ok(1);
    }
    if file.len() < 8 {
        return Err(BadInput("a PNG file shorter than its signature"));
    }
    if file[..8] != SIGNATURE {
        return Ok(2);
    }
    // First the whole walk without writing anything: what is refused here leaves no trace.
    // (Data that does not inflate, and a row filter above 4, are found out later, with the
    // pixel area written to; the file is untouched, and the original starts from it.)
    let mut header: Option<(u32, u32, u32, u8)> = None;
    let mut palette: Option<usize> = None;
    let mut data: Option<(usize, usize)> = None;
    let mut at = 8usize;
    loop {
        if at + 8 > file.len() {
            return Err(BadInput("a PNG chunk past the end of the file"));
        }
        let length = be(&file[at..]) as usize;
        let kind = &file[at + 4..at + 8];
        let start = at + 8;
        if start.checked_add(length).is_none_or(|end| end > file.len()) {
            return Err(BadInput("a PNG chunk past the end of the file"));
        }
        match kind {
            b"IHDR" => {
                if length < 11 || data.is_some() {
                    return Err(BadInput("a PNG header out of place"));
                }
                if file[start + 8] != 8 {
                    return Err(BadInput("a PNG bit depth other than 8"));
                }
                header = Some((
                    be(&file[start..]),
                    be(&file[start + 4..]),
                    file[start + 9] as u32 & 7,
                    file[start + 10],
                ));
            }
            b"PLTE" => palette = Some(start),
            b"IDAT" => {
                if data.is_some() || header.is_none() {
                    return Err(BadInput(
                        "more than one PNG data chunk, or one before the header",
                    ));
                }
                data = Some((start, length));
            }
            _ => {}
        }
        if kind == b"IEND" {
            break;
        }
        at = start + length + 4;
    }
    // Without a header in the file the variables keep what the last file left in them.
    let (width, height, colour_type) = match header {
        Some((width, height, colour_type, _)) => (width, height, colour_type),
        None => unsafe {
            (
                game_read(WIDTH_RVA),
                game_read(HEIGHT_RVA),
                game_read(COLOUR_TYPE_RVA),
            )
        },
    };
    if palette.is_some_and(|start| start + 0x300 > file.len()) {
        return Err(BadInput("a PNG palette past the end of the file"));
    }
    let pixel_bytes = PIXEL_BYTES[colour_type as usize & 7];
    if data.is_some() {
        if !matches!(colour_type, 0 | 2 | 3) {
            return Err(BadInput(
                "a PNG colour type the original takes for one byte a pixel",
            ));
        }
        if width < 2 || height == 0 {
            return Err(BadInput("a PNG picture without width or height"));
        }
        if (pixel_bytes as u64 * width as u64 + 1) * height as u64 > PICTURE_DWORDS as u64 * 4 {
            return Err(BadInput("a PNG picture too big for the picture buffer"));
        }
    }

    unsafe {
        if let Some((_, _, _, compression)) = header {
            game_write(WIDTH_RVA, width);
            game_write(HEIGHT_RVA, height);
            game_write(COMPRESSION_RVA, compression);
            game_write(COLOUR_TYPE_RVA, colour_type);
        }
        if let Some((start, length)) = data {
            let room = ((pixel_bytes * width + 1) * height) as usize;
            let out = std::slice::from_raw_parts_mut(pixels, room);
            let mut inflater = DecompressorOxide::new();
            let flags = inflate_flags::TINFL_FLAG_PARSE_ZLIB_HEADER
                | inflate_flags::TINFL_FLAG_USING_NON_WRAPPING_OUTPUT_BUF;
            let (status, _, _) =
                decompress(&mut inflater, &file[start..start + length], out, 0, flags);
            // Done, the area full, or all there but for zlib's check sum: in each the
            // original has written the same bytes.
            if !matches!(
                status,
                TINFLStatus::Done | TINFLStatus::HasMoreOutput | TINFLStatus::Adler32Mismatch
            ) {
                return Err(BadInput("PNG data that does not inflate cleanly"));
            }
            let (row_bytes, step) = if colour_type == 2 {
                (width as usize * 3, 3)
            } else {
                (width as usize, 1)
            };
            unfilter(pixels, row_bytes, height, step)?;
        }
        (&raw mut (*info).has_palette).write(0);
        (&raw mut (*info).width).write_unaligned(width);
        (&raw mut (*info).height).write_unaligned(height);
        if let Some(start) = palette {
            (&raw mut (*info).has_palette).write(1);
            std::ptr::copy(
                file[start..].as_ptr(),
                (&raw mut (*info).palette).cast(),
                0x300,
            );
        }
    }
    let (info, pixels) = (info as u32, pixels as u32);
    let end = pixel_bytes
        .wrapping_mul(width)
        .wrapping_mul(height)
        .wrapping_add(pixels);
    Ok(if info >= pixels && info < end { 3 } else { 0 })
}

// ---- hook ----

type Original = unsafe extern "C" fn(*const u8, *mut u8, *mut PngInfo) -> u32;

/// The part of the info block and of the pixel area that a decoding defines.
unsafe fn defined(pixels: *const u8, info: *const PngInfo) -> Vec<u8> {
    unsafe {
        let (width, height) = ((*info).width as usize, (*info).height as usize);
        let mut bytes = vec![(*info).has_palette];
        bytes.extend((width as u32).to_le_bytes());
        bytes.extend((height as u32).to_le_bytes());
        if (*info).has_palette == 1 {
            bytes.extend((*info).palette);
        }
        let step = if game_read::<u32>(COLOUR_TYPE_RVA) == 2 {
            3
        } else {
            1
        };
        let length = width.saturating_mul(height).saturating_mul(step);
        if length <= PICTURE_DWORDS * 4 {
            bytes.extend(std::slice::from_raw_parts(pixels, length));
        }
        bytes
    }
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    // A C function: its arguments are above the return address, which is above the flags
    // the thunk pushed before the registers.
    let args = (r.esp as usize + 8) as *const u32;
    let (file, pixels, info) = unsafe {
        (
            args.read() as *const u8,
            args.add(1).read() as *mut u8,
            args.add(2).read() as *mut PngInfo,
        )
    };
    let original: Original = unsafe { std::mem::transmute(ORIGINAL.load(Ordering::SeqCst)) };
    // The length is only known for the file `read_png` holds.
    let known = !file.is_null() && unsafe { game_read::<u32>(FILE_RVA) } == file as u32;
    let mine = if known {
        let length = unsafe { game_read::<u32>(FILE_LEN_RVA) } as usize;
        unsafe { decode_png(std::slice::from_raw_parts(file, length), pixels, info) }
    } else {
        Err(BadInput("a PNG file of unknown length"))
    };
    if !shadow() {
        r.eax = match mine {
            Ok(result) => result,
            Err(_) => unsafe { original(file, pixels, info) },
        };
        return;
    }
    let state = mine
        .is_ok_and(|result| result == 0)
        .then(|| unsafe { defined(pixels, info) });
    let result = unsafe { original(file, pixels, info) };
    if let Ok(my_result) = mine
        && (my_result != result
            || state.is_some_and(|state| state != unsafe { defined(pixels, info) }))
    {
        HOOK.mismatch(|| {
            format!(
                "a file of {} bytes: original returned {result}, rust {my_result}; the pixels or the info differ: {}",
                unsafe { game_read::<u32>(FILE_LEN_RVA) },
                my_result == result
            )
        });
    }
    r.eax = result;
}

// ---- self-test: PNG files made for the purpose, decoded into buffers of the test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let original: Original = unsafe { std::mem::transmute(ORIGINAL.load(Ordering::SeqCst)) };
    let saved: [u32; 5] = unsafe { game_read(WIDTH_RVA) };
    let mut cases: Vec<(String, Vec<u8>, Option<Vec<u8>>)> = Vec::new();
    for (width, height, bpp) in [
        (2u32, 1u32, 3usize),
        (5, 4, 3),
        (17, 9, 3),
        (64, 40, 3),
        (6, 5, 1),
        (33, 7, 1),
    ] {
        for filters in [
            &[0u8][..],
            &[1],
            &[0, 2],
            &[1, 3],
            &[0, 4],
            &[1, 2, 3, 4, 0],
            &[2],
            &[3],
            &[4],
        ] {
            let (file, pixels) = test_png(width, height, bpp, filters, width * 31 + height);
            // With a filter on the first row that looks at the row above, the pixels depend
            // on what lies before the area, and are not the picture's.
            let expected = (filters[0] < 2).then_some(pixels);
            cases.push((
                format!("{width}x{height} bpp {bpp} filters {filters:?}"),
                file,
                expected,
            ));
        }
    }
    cases.push(("not a PNG".into(), vec![7u8; 64], None));
    for (what, file, expected) in &cases {
        // Room before the area for a "row above" the first, with something in it.
        let run = |rust: bool| {
            let mut area = vec![0x5Au8; 0x1000 + 0x8000];
            let mut info = vec![0xAAu8; std::mem::size_of::<PngInfo>()];
            let pixels = unsafe { area.as_mut_ptr().add(0x1000) };
            let result = if rust {
                unsafe { decode_png(file, pixels, info.as_mut_ptr().cast()) }
            } else {
                // zlib's work area lies over the info block and on: give it room.
                let mut work = vec![0xAAu8; 0x20000];
                let result = unsafe { original(file.as_ptr(), pixels, work.as_mut_ptr().cast()) };
                let length = info.len();
                info.copy_from_slice(&work[..length]);
                Ok(result)
            };
            let seen = if result == Ok(0) {
                unsafe { defined(pixels, info.as_ptr().cast()) }
            } else {
                Vec::new()
            };
            (result, seen, area)
        };
        let theirs = run(false);
        let mine = run(true);
        let as_made = expected
            .as_ref()
            .is_none_or(|pixels| mine.1.ends_with(pixels));
        t.check(theirs.0 == mine.0 && theirs.1 == mine.1 && as_made, || {
            format!(
                "{what}: original {:?} rust {:?}; same info and pixels={} the picture as made={as_made} same area={}",
                theirs.0,
                mine.0,
                theirs.1 == mine.1,
                theirs.2 == mine.2
            )
        });
    }
    unsafe { game_write(WIDTH_RVA, saved) };
    t.finish()
}
