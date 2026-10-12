//! `gfx::read_pcx_24bit` (0x411D57): decodes a PCX file of three planes, 8 bits each, into
//! the picture buffer as 32-bit pixels.
//!
//! In:  eax = path
//! Out: carry = failed; every register preserved
//! Opens the file as the stream `dword_5195E4`, reads the header through it, sets
//! `g_pictureFlags` to 0x800 and calls `lay_out_pcx`. Then the rows. A row in the file is
//! one plane after the other, each the header's count of bytes; plane n's bytes go to byte
//! `planes - 1 - n` of the pixels (so the first plane is the third byte), those within the
//! decoded width. Then the fourth byte of every pixel of the row is set to 0, and the
//! destination moves on by the stride. A run goes on across the ends of planes and rows,
//! but the byte it repeats is 0 from the end of a row on (the zeroing leaves al 0). What
//! `read_stream_byte` says about the end of the file is not looked at.
//! It fails if the open, the header's read or the close fails.
//! Two callers, `gfx::read_pcx` and `sub_4119AF`. Shadow mode decodes the file twice and
//! compares.
//! Refuses a decoded width or height of 0 (the original counts through all 32 bits) and
//! what `open_stream` refuses. The name is a reading.

use crate::errors::BadInput;
use crate::files::close_stream::close_stream;
use crate::files::open_stream::open_stream;
use crate::files::read_stream_block::read_stream_block;
use crate::files::read_stream_byte::read_stream_byte;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva};
use crate::gfx::lay_out_pcx::lay_out_pcx;
use crate::gfx::pcx::{
    DECODED_HEIGHT_RVA, DECODED_WIDTH_RVA, HEADER_LEN, HEADER_RVA, LINE_BYTES_RVA, MISSING_PATH,
    PIXELS_RVA, PLANES_RVA, STRIDE_RVA, TEST_PATH, remove_test_file, stream, test_header,
    test_plane_bytes, test_runs, write_test_file,
};
use crate::gfx::pcx_reader::{ReaderTest, dispatch_reader};
use crate::gfx::picture::FLAGS_RVA;
use std::sync::atomic::{AtomicUsize, Ordering};

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("gfx::read_pcx_24bit", rva(0x411D57), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- Rust version ----

/// The part between the open and the close: whether it worked.
unsafe fn read_rows() -> Result<bool, BadInput> {
    unsafe {
        let stream = stream();
        let header = game_addr(HEADER_RVA) as *mut u8;
        if read_stream_block(stream, header, HEADER_LEN as u32).1 {
            return Ok(false);
        }
        game_write(FLAGS_RVA, 0x800u32);
        lay_out_pcx();

        let (width, height, line_bytes, planes): (u32, u32, u32, u32) = (
            game_read(DECODED_WIDTH_RVA),
            game_read(DECODED_HEIGHT_RVA),
            game_read(LINE_BYTES_RVA),
            game_read(PLANES_RVA),
        );
        if width == 0 || height == 0 {
            return Err(BadInput("a PCX picture without rows or columns"));
        }
        let stride: u32 = game_read(STRIDE_RVA);
        let mut to: u32 = game_read(PIXELS_RVA);
        // eax as the original keeps it (al is the byte a run repeats), and what is left of
        // the run, which is counted down before it is looked at, as a signed number.
        let (mut eax, mut run) = (0u32, 0i32);
        for _ in 0..height {
            to = to.wrapping_add(planes);
            let mut plane = 0u32;
            loop {
                to = to.wrapping_sub(1);
                let mut column = 0u32;
                loop {
                    run = run.wrapping_sub(1);
                    if run <= 0 {
                        eax = read_stream_byte(stream, eax).eax;
                        run = 1;
                        if eax as u8 >= 0xC0 {
                            run = (eax as u8 - 0xC0) as i32;
                            eax = read_stream_byte(stream, eax).eax;
                        }
                    }
                    if column < width {
                        (to.wrapping_add(column.wrapping_mul(4)) as *mut u8).write(eax as u8);
                    }
                    column = column.wrapping_add(1);
                    if column >= line_bytes {
                        break;
                    }
                }
                plane = plane.wrapping_add(1);
                if plane >= planes {
                    break;
                }
            }
            // The fourth byte of every pixel, from the last one down.
            eax = 0;
            for pixel in (1..=width).rev() {
                (to.wrapping_add(pixel.wrapping_mul(4)).wrapping_sub(1) as *mut u8).write(0);
            }
            to = to.wrapping_add(stride);
        }
        Ok(true)
    }
}

pub unsafe fn read_pcx_24bit(path: *const u8) -> Result<bool, BadInput> {
    unsafe {
        if !open_stream(stream(), path, 0)? {
            return Ok(false);
        }
        let rows = read_rows();
        let closed = close_stream(stream());
        Ok(rows? && closed)
    }
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_reader(&HOOK, r, call_original, read_pcx_24bit);
}

// ---- self-test: PCX files of the test's own ----

/// A whole file: header and coded rows.
pub fn test_file(width: u16, height: u16, planes: u8, line_bytes: u16, seed: u32) -> Vec<u8> {
    let mut file = test_header(1, 1, width, height, planes, line_bytes);
    let count = line_bytes as usize * planes as usize * height as usize;
    file.extend(test_runs(&test_plane_bytes(count, seed)));
    file
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let Some(test) = ReaderTest::begin() else {
        return t.skip("the picture buffer is not allocated");
    };
    let original = ORIGINAL.load(Ordering::SeqCst);
    let mut files: Vec<(String, Vec<u8>)> = Vec::new();
    // (width, height, planes, bytes in a plane's row): the last three are not what the game
    // has, a fourth plane, one plane, and rows shorter than the width.
    for (width, height, planes, line_bytes) in [
        (10u16, 4u16, 3u8, 10u16),
        (9, 5, 3, 10),
        (33, 17, 3, 34),
        (1, 1, 3, 2),
        (64, 8, 3, 64),
        (7, 3, 4, 8),
        (7, 3, 1, 8),
        (12, 3, 3, 8),
    ] {
        for seed in [1u32, 0xBEEF] {
            files.push((
                format!("{width}x{height} planes {planes} line {line_bytes} seed {seed:#x}"),
                test_file(width, height, planes, line_bytes, seed),
            ));
        }
    }
    // Rows that end early, a header alone, less than a header, and runs of none.
    let mut short = test_file(20, 6, 3, 20, 7);
    short.truncate(HEADER_LEN + 25);
    files.push(("rows that end early".into(), short));
    files.push(("a header alone".into(), test_header(0, 0, 8, 2, 3, 8)));
    // Cut off just past the two fields the layout reads last, so that the size is still the
    // file's own.
    let mut cut = test_header(0, 0, 5, 2, 3, 6);
    cut.truncate(0x44);
    files.push(("less than a header".into(), cut));
    let mut none = test_header(0, 0, 6, 3, 3, 6);
    none.extend([5, 0xC3, 9, 0xC0, 0x41, 0xC0, 0xC0, 1, 2, 3, 0xC5, 0xC5, 4]);
    files.push(("runs of none".into(), none));

    for (what, content) in files {
        if !write_test_file(&content) {
            test.end();
            return t.skip("cannot create a test file in the game's var folder");
        }
        test.check(&mut t, &what, original, TEST_PATH, read_pcx_24bit);
    }
    remove_test_file();
    test.check(
        &mut t,
        "a missing file",
        original,
        MISSING_PATH,
        read_pcx_24bit,
    );
    test.end();
    t.finish()
}
