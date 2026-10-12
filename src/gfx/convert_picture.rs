//! `gfx::convert_picture` (0x406126): turns the picture in the picture buffer, 32-bit pixels
//! of four components, into pixels of the surface's format, in place.
//!
//! In:  nothing
//! Out: every register preserved
//! The surface's format (`g_surfaceFormat`, 0x4F496C) has a number at +0x14. If the table
//! `dword_4F5288` holds a converter for that number, it is called (ebx = the format, esi =
//! edi = the buffer, eax = the number, ecx = width * height, edx = 1, ebp = 0). Otherwise
//! every pixel goes through `components_to_pixel`: with a 16-bit screen the results are
//! written as words from the buffer's start, with any other as dwords. With a 16-bit screen,
//! whichever way the pixels were made, bits 8 .. 11 of the picture's flags become 2 and its
//! row length is halved.
//! 14 callers. Of the converters (the table is filled by `gfx::install_converters`) two are
//! ported and called as Rust, `copy_pixel_rows` (the owner's 32-bit screen: nothing to do)
//! and `components_to_4444_rows`; the eight MMX and 3DNow ones are not: the Rust version
//! calls the one the table names, and shadow mode then runs the original alone.
//! Refuses a null buffer and a width or height of 0 (the original's loops count through all
//! 32 bits). No limit on width * height, as in the original. The name is a reading.

use crate::errors::{BadInput, Refused};
use crate::framework::{
    RawHook, Regs, SelfTest, call_game, game_addr, game_read, game_write, hook_asm, log, rva,
    shadow,
};
use crate::gfx::components_to_4444_rows::{COMPONENTS_TO_4444_ROWS, components_to_4444_rows};
use crate::gfx::components_to_pixel::components_to_pixel;
use crate::gfx::copy_pixel_rows::{COPY_PIXEL_ROWS, copy_pixel_rows};
use crate::gfx::harness::dispatch_pixels;
use crate::gfx::install_converters::CONVERTERS_RVA;
use crate::gfx::picture::{
    FLAGS_RVA, HEIGHT_RVA, ROW_BYTES_RVA, SCREEN_BITS_RVA, WIDTH_RVA, picture, picture_region,
};
use crate::gfx::pixel_format::{PixelFormat, SCRATCH_RVA, SURFACE_FORMAT_RVA};
use crate::redirect::RedirectedCopy;
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, restore};
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicUsize, Ordering};

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("gfx::convert_picture", rva(0x406126), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

const FORMAT_NUMBER: usize = 0x14; // in the surface's format

/// The surface format's number.
fn format_number() -> u32 {
    unsafe { game_read(SURFACE_FORMAT_RVA + FORMAT_NUMBER) }
}

/// The converter the game has for the surface's format, or 0.
fn converter() -> u32 {
    unsafe { game_read(CONVERTERS_RVA + format_number() as usize * 4) }
}

/// Whether the Rust version handles this table entry itself.
fn ported(converter: u32) -> bool {
    [
        0,
        game_addr(COPY_PIXEL_ROWS),
        game_addr(COMPONENTS_TO_4444_ROWS),
    ]
    .contains(&(converter as usize))
}

// ---- Rust version ----

/// `screen_bits` is `dword_4F4954`; `converter` the table's entry for the surface's format.
pub unsafe fn convert_picture(screen_bits: u32, converter: u32) -> Result<(), BadInput> {
    unsafe {
        let buffer = picture();
        let (width, height): (u32, u32) = (game_read(WIDTH_RVA), game_read(HEIGHT_RVA));
        if buffer.is_null() {
            return Err(BadInput("no picture buffer"));
        }
        if converter == 0 && (width == 0 || height == 0) {
            return Err(BadInput("a picture without a width or a height"));
        }
        if converter as usize == game_addr(COPY_PIXEL_ROWS) {
            // The picture is already what the surface wants: from and to the same place,
            // `copy_pixel_rows` does nothing.
            copy_pixel_rows(
                buffer as u32,
                buffer as u32,
                width.wrapping_mul(height),
                1,
                0,
            )?;
            if screen_bits != 16 {
                return Ok(());
            }
        } else if converter as usize == game_addr(COMPONENTS_TO_4444_ROWS) {
            if width.wrapping_mul(height) == 0 {
                return Err(BadInput("a picture without a width or a height"));
            }
            components_to_4444_rows(
                buffer as u32,
                buffer as u32,
                width.wrapping_mul(height),
                1,
                0,
            )?;
            if screen_bits != 16 {
                return Ok(());
            }
        } else if converter != 0 {
            let mut g = Regs {
                eax: format_number(),
                ebx: game_addr(SURFACE_FORMAT_RVA) as u32,
                ecx: width.wrapping_mul(height),
                edx: 1,
                esi: buffer as u32,
                edi: buffer as u32,
                ebp: 0,
                ..Regs::default()
            };
            call_game(&mut g, converter as usize);
            if screen_bits != 16 {
                return Ok(());
            }
        } else {
            let format: PixelFormat = game_read(SURFACE_FORMAT_RVA);
            let pixels = width as usize * height as usize;
            for n in 0..pixels {
                let components = (buffer.add(n * 4) as *const u32).read_unaligned();
                let pixel = components_to_pixel(&format, components);
                if screen_bits == 16 {
                    (buffer.add(n * 2) as *mut u16).write_unaligned(pixel as u16);
                } else {
                    (buffer.add(n * 4) as *mut u32).write_unaligned(pixel);
                }
            }
            if screen_bits != 16 {
                return Ok(());
            }
        }
        game_write(FLAGS_RVA, game_read::<u32>(FLAGS_RVA) & 0xFFFF_F0FF | 0x200);
        game_write(ROW_BYTES_RVA, game_read::<u32>(ROW_BYTES_RVA) >> 1);
        Ok(())
    }
}

/// With the screen's bits and the surface format's converter as they are now.
pub unsafe fn convert_picture_now() -> Result<(), BadInput> {
    unsafe { convert_picture(game_read(SCREEN_BITS_RVA), converter()) }
}

// ---- hook ----

/// What the function writes, for a picture of the size the variables say now.
fn regions() -> Vec<Region> {
    let pixels =
        unsafe { game_read::<u32>(WIDTH_RVA) as usize * game_read::<u32>(HEIGHT_RVA) as usize };
    vec![
        picture_region(pixels * 4),
        at(FLAGS_RVA),
        at(ROW_BYTES_RVA),
        at(SCRATCH_RVA),
    ]
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { convert_picture(game_read(SCREEN_BITS_RVA), converter()) }?;
    Ok(Left::nothing())
}

static SAID_WHICH_WAY: AtomicBool = AtomicBool::new(false);

extern "C" fn dispatch(r: &mut Regs) {
    // Once a run, for hook.log: whether this screen mode's pictures go through a converter
    // of the game's (then shadow mode compares nothing here) or through the pixel loop.
    if !SAID_WHICH_WAY.swap(true, Ordering::SeqCst) {
        log(&format!(
            "gfx::convert_picture: surface format number {}, converter {:#x}, screen bits {}",
            format_number(),
            converter(),
            unsafe { game_read::<u32>(SCREEN_BITS_RVA) }
        ));
    }
    // A converter that is still the game's code is not run twice.
    let unsafe_twice = picture().is_null() || !ported(converter());
    if shadow() && unsafe_twice {
        HOOK.count_call();
        unsafe { call_original(r) };
        return;
    }
    if picture().is_null() {
        HOOK.count_call();
        let e = BadInput("no picture buffer");
        unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) };
        return;
    }
    dispatch_pixels(&HOOK, r, call_original, &regions(), &rust);
}

// ---- self-test: a copy of the original that reads the screen's bits and the converter table
// from test variables, on a small picture in the game's own buffer ----

static TEST_BITS: AtomicU32 = AtomicU32::new(0);
static NO_CONVERTERS: [AtomicU32; 64] = [const { AtomicU32::new(0) }; 64];

const LEN: usize = 0xAC;
const ENTRY_BYTES: [u8; 8] = [0x60, 0x8D, 0x1D, 0x6C, 0x49, 0x4F, 0x00, 0x8B];
/// Both go to `components_to_pixel`, and stay so.
const CALLS: [usize; 2] = [0x5C, 0x9A];
/// Where the copy names `dword_4F4954` and the converter table.
const BITS_OPERANDS: [usize; 2] = [0x39, 0x44];
const TABLE_OPERANDS: [usize; 2] = [0x15, 0x33];

const TEST_WIDTH: u32 = 16;
const TEST_HEIGHT: u32 = 5;

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    if picture().is_null() {
        return t.skip("the picture buffer is not allocated");
    }
    if format_number() as usize >= NO_CONVERTERS.len() {
        return t.skip("the surface format's number is past the test's table");
    }
    let copy = match unsafe {
        RedirectedCopy::new(game_addr(rva(0x406126)), LEN, &ENTRY_BYTES, &CALLS, &|_| {
            None
        })
    } {
        Ok(copy) => copy,
        Err(why) => return t.skip(why),
    };
    for operand in BITS_OPERANDS {
        unsafe { copy.patch_u32(operand, &TEST_BITS as *const AtomicU32 as u32) };
    }
    for operand in TABLE_OPERANDS {
        unsafe { copy.patch_u32(operand, NO_CONVERTERS.as_ptr() as u32) };
    }

    let bytes = (TEST_WIDTH * TEST_HEIGHT * 4) as usize;
    let watched = [
        picture_region(bytes),
        at(FLAGS_RVA),
        at(ROW_BYTES_RVA),
        at(SCRATCH_RVA),
    ];
    let inputs = [at(WIDTH_RVA), at(HEIGHT_RVA)];
    let saved = (capture(&watched), capture(&inputs));

    for bits in [16u32, 32, 8] {
        for (flags, seed) in [(0x802u32, 7u32), (0xF00, 0x1234_5678), (0, 0)] {
            let prepare = || unsafe {
                game_write(WIDTH_RVA, TEST_WIDTH);
                game_write(HEIGHT_RVA, TEST_HEIGHT);
                game_write(FLAGS_RVA, flags);
                game_write(ROW_BYTES_RVA, TEST_WIDTH * 4);
                game_write(SCRATCH_RVA, 0xAAAA_AAAAu32);
                let mut value = seed;
                for n in 0..bytes / 4 {
                    // A pixel of zeros now and then: `components_to_pixel` treats it apart.
                    value = value.wrapping_mul(0x0019_660D).wrapping_add(0x3C6E_F35F);
                    let pixel = if n % 7 == 3 || seed == 0 { 0 } else { value };
                    (picture().add(n * 4) as *mut u32).write_unaligned(pixel);
                }
                TEST_BITS.store(bits, Ordering::SeqCst);
            };
            let before = Regs::SENTINEL;
            prepare();
            let mut ro = before;
            unsafe { call_game(&mut ro, copy.addr()) };
            let theirs = capture(&watched);
            prepare();
            let mine = unsafe { convert_picture(bits, 0) };
            let state = capture(&watched);
            let preserved = ro.preserved_except(&before, 0);
            t.check(mine.is_ok() && theirs == state && preserved, || {
                let first = theirs.iter().zip(&state).position(|(a, b)| a != b);
                format!(
                    "bits={bits} flags={flags:#x} seed={seed:#x}: preserved={preserved} rust {mine:?} first different byte {first:?} of {}",
                    theirs.len()
                )
            });
        }
    }
    restore(&watched, &saved.0);
    restore(&inputs, &saved.1);
    t.finish()
}
