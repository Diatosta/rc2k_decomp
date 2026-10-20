//! `gfx::read_pcx_8bit` (0x411C6B): decodes a PCX file of one plane, 8 bits a pixel with a
//! palette, into the picture buffer as 32-bit pixels.
//!
//! In:  eax = path
//! Out: carry = failed; every register preserved
//! Opens the file as the stream `dword_5195E4`; reads the palette, the last 0x300 bytes of
//! the file, with a seek and a plain read on the stream's handle, and seeks back to the
//! start; reads the header through the stream; sets `g_pictureFlags` to 0x800 and calls
//! `lay_out_pcx`. Then the rows: for each of the row's bytes (the header's count, which may
//! be more than the width) the next palette number, whose dword is stored if the byte is
//! within the decoded width. A run goes on across the end of a row. After a row the
//! destination moves on by the row gap. What `read_stream_byte` says about the end of the
//! file is not looked at: a file that ends early is filled up with palette entry 0.
//! It fails if the open, a seek, the palette's read, the header's read or the close fails.
//! A count byte of 0xC0, a run of none, makes a run of 2^32: the rest of the picture.
//! One caller, `gfx::read_pcx`. Shadow mode decodes the file twice and compares.
//! Refuses a decoded height of 0 (the original counts rows through all 32 bits) and what
//! `open_stream` refuses. The name is a reading.

use crate::errors::BadInput;
use crate::files::close_stream::close_stream;
use crate::files::handle::{read, seek};
use crate::files::open_stream::open_stream;
use crate::files::read_stream_block::read_stream_block;
use crate::files::read_stream_byte::read_stream_byte;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva};
use crate::gfx::lay_out_pcx::lay_out_pcx;
use crate::gfx::pcx::{
    FILE_PALETTE_LEN, FILE_PALETTE_RVA, HEADER_LEN, HEADER_RVA, MISSING_PATH, PALETTE_RVA,
    PIXELS_RVA, TEST_PATH, remove_test_file, stream, test_header, test_palette, test_plane_bytes,
    test_runs, write_test_file,
};
use crate::gfx::pcx_reader::{ReaderTest, dispatch_reader};
use crate::gfx::picture::FLAGS_RVA;
use std::sync::atomic::{AtomicUsize, Ordering};

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("gfx::read_pcx_8bit", rva(0x411C6B), thunk, &ORIGINAL).self_test(self_test);

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
        let file = (*stream).file;
        if seek(file, -(FILE_PALETTE_LEN as i32), 2).is_none() {
            return Ok(false);
        }
        let palette_bytes = game_addr(FILE_PALETTE_RVA) as *mut u8;
        if !read(file, palette_bytes, FILE_PALETTE_LEN as u32).1 {
            return Ok(false);
        }
        if seek(file, 0, 0).is_none() {
            return Ok(false);
        }
        let header = game_addr(HEADER_RVA) as *mut u8;
        if read_stream_block(stream, header, HEADER_LEN as u32).1 {
            return Ok(false);
        }
        game_write(FLAGS_RVA, 0x800u32);
        let layout = lay_out_pcx();
        // Converted: `rc2k_core::pcx::decode_8bit`.
        let palette: [u32; 256] = game_read(PALETTE_RVA);
        let pixels: u32 = game_read(PIXELS_RVA);
        rc2k_core::pcx::decode_8bit(
            &layout,
            &palette,
            |eax| read_stream_byte(stream, eax).eax,
            |to, pixel| (pixels.wrapping_add(to) as *mut u32).write_unaligned(pixel),
        )
        .ok_or(BadInput("a PCX picture without rows"))?;
        Ok(true)
    }
}

pub unsafe fn read_pcx_8bit(path: *const u8) -> Result<bool, BadInput> {
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
    dispatch_reader(&HOOK, r, call_original, read_pcx_8bit);
}

// ---- self-test: PCX files of the test's own ----

/// A whole file: header, coded rows, palette.
pub fn test_file(width: u16, height: u16, line_bytes: u16, seed: u32) -> Vec<u8> {
    let mut file = test_header(2, 3, width, height, 1, line_bytes);
    let rows = test_plane_bytes(line_bytes as usize * height as usize, seed);
    file.extend(test_runs(&rows));
    file.extend(test_palette());
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
    for (width, height, line_bytes) in [
        (10u16, 4u16, 10u16),
        (9, 5, 10),
        (33, 17, 34),
        (1, 1, 2),
        (64, 8, 64),
        (12, 3, 8),
    ] {
        for seed in [1u32, 0xBEEF] {
            files.push((
                format!("{width}x{height} line {line_bytes} seed {seed:#x}"),
                test_file(width, height, line_bytes, seed),
            ));
        }
    }
    // A file whose rows end early, one that is only a header and a palette, one shorter
    // than a palette, and one with a run of none in it.
    let mut short = test_file(20, 6, 20, 7);
    short.drain(HEADER_LEN + 10..short.len() - FILE_PALETTE_LEN);
    files.push(("rows that end early".into(), short));
    let mut none = test_header(0, 0, 8, 2, 1, 8);
    none.extend(test_palette());
    files.push(("no rows at all".into(), none));
    files.push((
        "a file of 0x90 bytes".into(),
        test_header(0, 0, 4, 4, 1, 4)
            .into_iter()
            .chain([7u8; 0x10])
            .collect(),
    ));
    let mut endless = test_header(0, 0, 6, 3, 1, 6);
    endless.extend([5, 0xC3, 9, 0xC0, 0x41, 1, 2, 3]);
    endless.extend(test_palette());
    files.push(("a run of none".into(), endless));

    for (what, content) in files {
        if !write_test_file(&content) {
            test.end();
            return t.skip("cannot create a test file in the game's var folder");
        }
        test.check(&mut t, &what, original, TEST_PATH, read_pcx_8bit);
    }
    remove_test_file();
    test.check(
        &mut t,
        "a missing file",
        original,
        MISSING_PATH,
        read_pcx_8bit,
    );
    test.end();
    t.finish()
}
