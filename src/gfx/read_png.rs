//! `gfx::read_png` (0x41CB50): reads a PNG file into the picture buffer as 32-bit pixels.
//!
//! In:  eax = path
//! Out: carry = failed; every register preserved
//! Reads the file's first 0x80 bytes into `buf` (0x93E8B0) and gives up unless they are
//! all there and are a PNG signature and an IHDR chunk of 13 bytes, with a width of at most
//! 800, a height of at most 600 and no interlacing. Then the file's size (`file_size`),
//! memory for the whole file from the game's allocator (`memory::allocate`, on the list
//! `dword_5BEC38`), the whole file read into it, `decode_png` into the picture buffer with
//! `buf` as the info block, `g_pictureFlags` = 0x802, `lay_out_png`, the picture's width,
//! height and row length set from the layout, and the pixels spread out to dwords
//! (`palette_pixels_to_dwords` if the file had a palette, else `rgb_pixels_to_dwords`).
//! The memory is given back (`memory::free_list`) on every way out. Once the decoder has been
//! called the carry flag is clear, whatever it returned.
//! One caller, the picture loader `sub_4119F4`, which falls back on the PCX reader.
//! Shadow mode runs the Rust version and then the original and compares the picture, its
//! variables and the layout (the palette only when the file has one: without one it is
//! made from the info block's leftovers, see `decode_png`). The file is read and decoded
//! twice, and its memory counted twice in the game's memory figures.
//! Refuses what `read_file`, `file_size` and `decode_png` refuse. The name is a reading.

use crate::errors::Refused;
use crate::files::file_size::file_size;
use crate::files::read_file::read_file;
use crate::files::stream::LastPath;
use crate::framework::{
    RawHook, Regs, SelfTest, call_game, game_addr, game_ptr, game_read, game_write, hook_asm, rva,
    shadow,
};
use crate::gfx::decode_png::{FILE_LEN_RVA, FILE_RVA, decode_png};
use crate::gfx::lay_out_png::lay_out_png;
use crate::gfx::palette_pixels_to_dwords::palette_pixels_to_dwords;
use crate::gfx::pcx::{
    DECODED_HEIGHT_RVA, DECODED_WIDTH_RVA, PALETTE_RVA, STRIDE_RVA, layout_regions,
};
use crate::gfx::picture::{
    FLAGS_RVA, HEIGHT_RVA, PICTURE_DWORDS, ROW_BYTES_RVA, WIDTH_RVA, picture,
};
use crate::gfx::png::{
    INFO_RVA, MISSING_PATH, PngInfo, TEST_PATH, remove_test_file, test_png, write_test_file,
};
use crate::gfx::rgb_pixels_to_dwords::rgb_pixels_to_dwords;
use crate::memory::MemoryList;
use crate::memory::allocate::allocate;
use crate::memory::free_list::free_list;
use crate::service::harness::{Region, at, capture, restore};
use std::sync::atomic::{AtomicUsize, Ordering};

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("gfx::read_png", rva(0x41CB50), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

const FILE_MEMORY_RVA: usize = rva(0x5BEC38); // dword_5BEC38: the list the file's memory is on
const HEAD_LEN: u32 = 0x80;

// ---- Rust version ----

/// What comes before the memory is taken: whether the file's head is a PNG the game takes.
unsafe fn head_is_taken(path: *const u8) -> Result<bool, Refused> {
    unsafe {
        let head = game_addr(INFO_RVA) as *mut u8;
        (head as *mut u32).write_unaligned(0);
        let read = read_file(path, head, HEAD_LEN, 0)?;
        if !read.ok || read.bytes_read.is_none_or(|n| n < HEAD_LEN) {
            return Ok(false);
        }
        // Converted: `rc2k_core::gfx::png::head_taken`.
        Ok(rc2k_core::gfx::png::head_taken(
            &(head as *const [u8; 0x80]).read_unaligned(),
        ))
    }
}

/// The part with the file's memory held: whether it worked.
unsafe fn read_held(path: *const u8, size: u32) -> Result<bool, Refused> {
    unsafe {
        let file = allocate(game_addr(FILE_MEMORY_RVA) as *mut MemoryList, size)?;
        game_write(FILE_RVA, file as u32);
        let read = read_file(path, file, size, 0)?;
        if !read.ok || read.bytes_read != Some(size) {
            return Ok(false);
        }
        decode_png(
            std::slice::from_raw_parts(file, size as usize),
            picture(),
            game_ptr(INFO_RVA),
        )?;
        game_write(FLAGS_RVA, 0x802u32);
        lay_out_png();
        game_write(WIDTH_RVA, game_read::<u32>(DECODED_WIDTH_RVA));
        game_write(HEIGHT_RVA, game_read::<u32>(DECODED_HEIGHT_RVA));
        game_write(ROW_BYTES_RVA, game_read::<u32>(STRIDE_RVA));
        if game_read::<u8>(INFO_RVA) == 1 {
            palette_pixels_to_dwords()?;
        } else {
            rgb_pixels_to_dwords()?;
        }
        Ok(true)
    }
}

/// Returns whether the picture was read.
pub unsafe fn read_png(path: *const u8) -> Result<bool, Refused> {
    unsafe {
        let free = || {
            // Refused only with the game's own heap, which `allocate` refuses first.
            let _ = free_list(game_addr(FILE_MEMORY_RVA) as *mut MemoryList);
        };
        // The original gives the list back on every way out, also with nothing on it.
        let taken = head_is_taken(path).inspect_err(|_| free())?;
        if !taken {
            free();
            return Ok(false);
        }
        let (size, sized) = file_size(path).inspect_err(|_| free())?;
        if !sized {
            free();
            return Ok(false);
        }
        game_write(FILE_LEN_RVA, size);
        let worked = read_held(path, size);
        free();
        worked
    }
}

// ---- hook ----

/// What a reading leaves that is defined: the picture's variables, the layout (its palette
/// only with a file that has one), the info block's set fields, and the pixels.
fn left_behind() -> Vec<u8> {
    unsafe {
        let info: *const PngInfo = game_ptr(INFO_RVA);
        let has_palette = (*info).has_palette == 1;
        let mut regions: Vec<Region> = layout_regions()
            .into_iter()
            .filter(|&(addr, _)| has_palette || addr != game_addr(PALETTE_RVA) as u32)
            .collect();
        regions.extend([
            at(WIDTH_RVA),
            at(HEIGHT_RVA),
            at(ROW_BYTES_RVA),
            at(FLAGS_RVA),
        ]);
        regions.push((info as u32, 1));
        regions.push((info as u32 + 4, 8));
        let pixels = game_read::<u32>(WIDTH_RVA) as usize * game_read::<u32>(HEIGHT_RVA) as usize;
        if !picture().is_null() && pixels <= PICTURE_DWORDS {
            regions.push((picture() as u32, pixels * 4));
        }
        capture(&regions)
    }
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    let path = r.eax as *const u8;
    if !shadow() {
        match unsafe { read_png(path) } {
            Ok(worked) => {
                r.set_carry(!worked);
            }
            Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
        }
        return;
    }
    let mine = unsafe { read_png(path) };
    let state = (mine == Ok(true)).then(left_behind);
    let mut o = *r;
    unsafe { call_original(&mut o) };
    if let Ok(worked) = mine
        && (worked == o.carry()
            || state.is_some_and(|state| state != left_behind())
            || !o.preserved_except(r, 0))
    {
        HOOK.mismatch(|| {
            format!(
                "path={:?}: original worked={} rust worked={worked}; what they left differs: {}",
                unsafe { std::ffi::CStr::from_ptr(path.cast()) },
                !o.carry(),
                worked != o.carry()
            )
        });
    }
    r.eflags = o.eflags;
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let path = r.eax as *const u8;
    {
        let worked = unsafe { read_png(path) }?;
        r.set_carry(!worked);
    }
    Ok(())
}

// ---- self-test: PNG files made for the purpose, read into the game's own picture buffer,
// which is put back with everything else ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    if picture().is_null() {
        return t.skip("the picture buffer is not allocated");
    }
    const PICTURE_BYTES: usize = 0x8000;
    let regions = [
        &layout_regions()[..],
        &[
            at(WIDTH_RVA),
            at(HEIGHT_RVA),
            at(ROW_BYTES_RVA),
            at(FLAGS_RVA),
            at(FILE_RVA),
            at(FILE_LEN_RVA),
            (game_addr(rva(0xA8C190)) as u32, 0x14),
            (game_addr(INFO_RVA) as u32, std::mem::size_of::<PngInfo>()),
            (picture() as u32, PICTURE_BYTES),
        ],
    ]
    .concat();
    let saved = capture(&regions);
    let last_path = LastPath::save();
    let original = ORIGINAL.load(Ordering::SeqCst);

    let mut files: Vec<(String, Vec<u8>)> = Vec::new();
    for (width, height, bpp, filters) in [
        (16u32, 8u32, 3usize, &[0u8, 1, 2, 3, 4][..]),
        (40, 30, 3, &[1, 4, 2]),
        (2, 1, 3, &[0]),
        (20, 10, 1, &[0, 1, 2, 3, 4]),
    ] {
        files.push((
            format!("{width}x{height} bpp {bpp}"),
            test_png(width, height, bpp, filters, 9).0,
        ));
    }
    let (plain, _) = test_png(12, 6, 3, &[0], 4);
    let mut interlaced = plain.clone();
    interlaced[0x1C] = 1;
    files.push(("interlaced".into(), interlaced));
    let mut wide = plain.clone();
    wide[0x10..0x14].copy_from_slice(&801u32.to_be_bytes());
    files.push(("801 wide".into(), wide));
    let mut high = plain.clone();
    high[0x14..0x18].copy_from_slice(&601u32.to_be_bytes());
    files.push(("601 high".into(), high));
    files.push(("less than 0x80 bytes".into(), plain[..0x7F].to_vec()));
    let mut not_png = plain.clone();
    not_png[1] = b'Q';
    files.push(("not a PNG".into(), not_png));

    let observed = || {
        (
            left_behind(),
            unsafe { std::slice::from_raw_parts(picture(), PICTURE_BYTES) }.to_vec(),
            LastPath::now(),
        )
    };
    let prepare = || unsafe {
        std::ptr::write_bytes(picture(), 0x5A, PICTURE_BYTES);
        game_write(WIDTH_RVA, 800u32);
        game_write(HEIGHT_RVA, 600u32);
        game_write(ROW_BYTES_RVA, 0xFFFF_FFFDu32);
        game_write(FLAGS_RVA, 0u32);
        std::ptr::write_bytes(
            game_addr(INFO_RVA) as *mut u8,
            0xAA,
            std::mem::size_of::<PngInfo>(),
        );
    };
    let mut check = |what: &str, path: &[u8]| {
        let before = Regs {
            eax: path.as_ptr() as u32,
            ..Regs::SENTINEL
        };
        prepare();
        let mut ro = before;
        unsafe { call_game(&mut ro, original) };
        let theirs = observed();
        prepare();
        let mine = unsafe { read_png(path.as_ptr()) };
        let state = observed();
        let preserved = ro.preserved_except(&before, 0);
        // After a failure the leftovers of the info block are not the same on both sides.
        let same = mine == Ok(false) && theirs.2 == state.2 || theirs == state;
        t.check(mine == Ok(!ro.carry()) && same && preserved, || {
            format!(
                "{what}: original worked={} preserved={preserved}; rust {mine:?}; same variables={} same picture={} last path {:#x} / {:#x}",
                !ro.carry(),
                theirs.0 == state.0,
                theirs.1 == state.1,
                theirs.2,
                state.2
            )
        });
    };
    for (what, content) in &files {
        if !write_test_file(content) {
            restore(&regions, &saved);
            last_path.restore();
            return t.skip("cannot create a test file in the game's var folder");
        }
        check(what, TEST_PATH);
    }
    remove_test_file();
    check("a missing file", MISSING_PATH);
    restore(&regions, &saved);
    last_path.restore();
    t.finish()
}
