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
use crate::gfx::png::{PngInfo, test_png};
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

// ---- Rust version ----
// Converted: `rc2k_core::gfx::png`. Here the game's variables for the header, the pixel area
// (with the row before it, which the first row's filters read) and the info block.

/// Returns what the original returns in eax.
pub unsafe fn decode_png(
    file: &[u8],
    pixels: *mut u8,
    info: *mut PngInfo,
) -> Result<u32, BadInput> {
    use rc2k_core::gfx::png;
    if pixels.is_null() || info.is_null() {
        return Ok(1);
    }
    // First the whole walk and the checks without writing anything: what is refused here
    // leaves no trace. (Data that does not inflate, and a row filter above 4, are found
    // out later, with the pixel area written to; the file is untouched, and the original
    // starts from it.)
    let Some(found) = png::chunks(file)? else {
        return Ok(2);
    };
    // Without a header in the file the variables keep what the last file left in them.
    let header = found.header.unwrap_or_else(|| unsafe {
        png::Header {
            width: game_read(WIDTH_RVA),
            height: game_read(HEIGHT_RVA),
            colour_type: game_read(COLOUR_TYPE_RVA),
            compression: 0,
        }
    });
    let pixel_bytes = png::PIXEL_BYTES[header.colour_type as usize & 7];
    let area = match found.data {
        Some(_) => png::area_for(&header, PICTURE_DWORDS * 4)?,
        None => 0,
    };
    unsafe {
        if let Some(h) = found.header {
            game_write(WIDTH_RVA, h.width);
            game_write(HEIGHT_RVA, h.height);
            game_write(COMPRESSION_RVA, h.compression);
            game_write(COLOUR_TYPE_RVA, h.colour_type);
        }
        if let Some((start, length)) = found.data {
            let (row_bytes, step) = png::row_layout(&header);
            let buffer = std::slice::from_raw_parts_mut(pixels, area);
            png::inflate(&file[start..start + length], buffer)?;
            // The row above the first is the memory before the area, read only where the
            // original reads it.
            let before = pixels.wrapping_sub(row_bytes) as *const u8;
            png::unfilter(buffer, row_bytes, header.height, step, &|i| {
                before.wrapping_add(i).read()
            })?;
        }
        (&raw mut (*info).has_palette).write(0);
        (&raw mut (*info).width).write_unaligned(header.width);
        (&raw mut (*info).height).write_unaligned(header.height);
        if let Some(start) = found.palette {
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
        .wrapping_mul(header.width)
        .wrapping_mul(header.height)
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

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
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
    r.eax = match mine {
        Ok(result) => result,
        Err(_) => unsafe { original(file, pixels, info) },
    };
    Ok(())
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
