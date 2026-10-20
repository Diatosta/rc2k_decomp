//! `gfx::read_picture_file` (0x4119F4): reads a picture file, PNG or PCX, into the picture
//! buffer as 32-bit pixels.
//!
//! In:  eax = path, which may be written to (see `files::swap_missing_extension`)
//! Out: carry = failed; every register preserved
//! A path ending in `.pcx` goes to `gfx::read_pcx`. Any other first has its ending swapped
//! if the file is missing, and then goes to `gfx::read_png` and, if that fails, to
//! `read_pcx` (which is how a `.png` path whose file is missing ends up read as PCX).
//! That is with the setting `dword_51B97C` at 0, as it is in the owner's game. With another
//! value a path ending in `.png` is not read at all (the value 2: success, nothing done) or
//! replaced by `files\d\gfx\chatzone.pcx`, and other paths go to `read_pcx` alone.
//! 14 callers. Shadow mode runs the Rust version, puts the path back and runs the original,
//! and compares the picture and its variables (not the palette: see `decode_png`).
//! Manual patch with a private copy that takes `read_pcx` with it (0x8E bytes, eight calls
//! relocated): the first call starts at the fourth byte, and the function leaves by two
//! short jumps to `read_pcx`, which follows it.
//! Refuses a path of fewer than four characters (the original looks at the four bytes
//! before the text's end) and what the readers refuse. The name is a reading.

use crate::errors::{BadInput, Refused};
use crate::files::stream::LastPath;
use crate::files::swap_missing_extension::swap_missing_extension;
use crate::framework::{
    RawHook, Regs, SelfTest, call_game, game_addr, game_read, game_write, hook_asm, log, rva,
    shadow,
};
use crate::gfx::pcx::{PALETTE_RVA, decoded_region, layout_regions, reader_inputs, reader_regions};
use crate::gfx::picture::{FLAGS_RVA, HEIGHT_RVA, ROW_BYTES_RVA, WIDTH_RVA, picture};
use crate::gfx::png::{INFO_RVA, PngInfo};
use crate::gfx::read_pcx::read_pcx;
use crate::gfx::read_png::read_png;
use crate::service::harness::{Region, at, capture, restore};
use crate::strings::text_length::text_length;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new("gfx::read_picture_file", rva(0x4119F4), thunk, &ORIGINAL)
    .private_copy(0x8E)
    // The call at 0x42 goes to `read_pcx` inside the copy and stays as it is.
    .relocate_calls(&[0x03, 0x13, 0x24, 0x49, 0x5E, 0x65, 0x76, 0x87])
    .manual_patch()
    .self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// `read_pcx`'s first bytes as they are in the image: what the copy has to hold where that
/// function begins in it. If `read_pcx` was hooked before the copy was made, the copy holds
/// the hook's jump there instead, which in the copy leads nowhere: the first build with
/// this hook ended the game at start-up that way (2026-10-12).
const READ_PCX_AT: usize = 0x51;
const READ_PCX_ENTRY: [u8; 8] = [0x60, 0x50, 0x8D, 0x1D, 0xF4, 0xB5, 0x51, 0x00];

fn copy_is_sound() -> bool {
    // The standalone has no original (nor its bytes): the Rust version runs alone there.
    if crate::framework::standalone() {
        return false;
    }
    let copy = ORIGINAL.load(Ordering::SeqCst);
    copy != 0 && unsafe { ((copy + READ_PCX_AT) as *const [u8; 8]).read() } == READ_PCX_ENTRY
}

static SAID_UNSOUND: AtomicBool = AtomicBool::new(false);

// ---- game data ----

const PNG_SETTING_RVA: usize = rva(0x51B97C); // dword_51B97C: a setting; 0 = PNG files are read
const CHATZONE_PATH_RVA: usize = rva(0x5AB400); // "files\d\gfx\chatzone.pcx"

// ---- Rust version ----

/// The path's last four characters.
unsafe fn ending(path: *const u8) -> Result<[u8; 4], BadInput> {
    let length = unsafe { text_length(path) }? as usize;
    if length < 4 {
        return Err(BadInput("a picture path of fewer than four characters"));
    }
    Ok(unsafe { (path.add(length - 4) as *const [u8; 4]).read() })
}

/// Returns whether a picture was read.
pub unsafe fn read_picture_file(path: *mut u8) -> Result<bool, Refused> {
    unsafe {
        if &ending(path)? == b".pcx" {
            return Ok(read_pcx(path)?);
        }
        swap_missing_extension(path)?;
        match game_read::<u32>(PNG_SETTING_RVA) {
            0 => Ok(read_png(path)? || read_pcx(path)?),
            setting => {
                if &ending(path)? != b".png" {
                    Ok(read_pcx(path)?)
                } else if setting == 2 {
                    Ok(true)
                } else {
                    Ok(read_pcx(game_addr(CHATZONE_PATH_RVA) as *const u8)?)
                }
            }
        }
    }
}

// ---- hook ----

/// What a reading leaves that is defined on both sides: the picture's variables, the layout
/// without its palette, the decoded rows.
pub fn left_behind() -> Vec<u8> {
    let mut regions: Vec<Region> = layout_regions()
        .into_iter()
        .filter(|&(addr, _)| addr != game_addr(PALETTE_RVA) as u32)
        .collect();
    regions.extend([
        at(WIDTH_RVA),
        at(HEIGHT_RVA),
        at(ROW_BYTES_RVA),
        at(FLAGS_RVA),
    ]);
    regions.extend(decoded_region());
    capture(&regions)
}

unsafe fn path_bytes(path: *const u8) -> Vec<u8> {
    unsafe { std::ffi::CStr::from_ptr(path.cast()) }
        .to_bytes()
        .to_vec()
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    let path = r.eax as *mut u8;
    if !copy_is_sound() {
        // The original cannot be run: the Rust version alone, whatever the mode.
        if !crate::framework::standalone() && !SAID_UNSOUND.swap(true, Ordering::SeqCst) {
            log(
                "gfx::read_picture_file: its copy of the original is not usable (hooked after read_pcx?); the Rust version runs alone",
            );
        }
        let worked = unsafe { read_picture_file(path) }.unwrap_or(false);
        r.set_carry(!worked);
        return;
    }
    if !shadow() {
        match unsafe { read_picture_file(path) } {
            Ok(worked) => r.set_carry(!worked),
            Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
        }
        return;
    }
    if path.is_null() {
        unsafe { call_original(r) };
        return;
    }
    let before = unsafe { path_bytes(path) };
    let mine = unsafe { read_picture_file(path) };
    let state = (unsafe { path_bytes(path) }, left_behind());
    unsafe { std::ptr::copy_nonoverlapping(before.as_ptr(), path, before.len()) };
    let mut o = *r;
    unsafe { call_original(&mut o) };
    let theirs = (unsafe { path_bytes(path) }, left_behind());
    if let Ok(worked) = mine
        && (worked == o.carry()
            || theirs.0 != state.0
            || (worked && theirs.1 != state.1)
            || !o.preserved_except(r, 0))
    {
        HOOK.mismatch(|| {
            format!(
                "path={:?}: original worked={} left the path {:?}; rust worked={worked} left {:?}; same picture={}",
                String::from_utf8_lossy(&before),
                !o.carry(),
                String::from_utf8_lossy(&theirs.0),
                String::from_utf8_lossy(&state.0),
                theirs.1 == state.1
            )
        });
    }
    r.eflags = o.eflags;
}

// ---- self-test: a PNG and a PCX file of the test's own, asked for by several paths, with
// the setting at each of its values ----

const PNG_FILE: &str = "var\\rc2k_hook_picture_a.png";
const PCX_FILE: &str = "var\\rc2k_hook_picture_b.pcx";

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    if picture().is_null() {
        return t.skip("the picture buffer is not allocated");
    }
    if !copy_is_sound() {
        return t.skip("the copy of the original does not hold read_pcx as it is in the image");
    }
    let made = std::fs::write(
        PNG_FILE,
        crate::gfx::png::test_png(14, 6, 3, &[0, 1, 4], 2).0,
    )
    .and_then(|_| {
        std::fs::write(
            PCX_FILE,
            crate::gfx::read_pcx_24bit::test_file(11, 5, 3, 12, 6),
        )
    });
    if made.is_err() {
        return t.skip("cannot create a test file in the game's var folder");
    }
    const PICTURE_BYTES: usize = 0x4000;
    let regions = [
        &reader_regions()[..],
        &reader_inputs(),
        &[
            at(PNG_SETTING_RVA),
            at(crate::gfx::decode_png::FILE_RVA),
            at(crate::gfx::decode_png::FILE_LEN_RVA),
            (game_addr(rva(0xA8C190)) as u32, 0x14),
            (game_addr(INFO_RVA) as u32, std::mem::size_of::<PngInfo>()),
            (picture() as u32, PICTURE_BYTES),
        ],
    ]
    .concat();
    let saved = capture(&regions);
    let last_path = LastPath::save();
    let original = ORIGINAL.load(Ordering::SeqCst);

    // The paths: a PNG that is there; a PCX that is there; a PNG that is not, whose PCX
    // partner is; one with neither; a PCX that is not there; another ending.
    let paths: [&[u8]; 6] = [
        b"var\\rc2k_hook_picture_a.png",
        b"var\\rc2k_hook_picture_b.pcx",
        b"var\\rc2k_hook_picture_b.png",
        b"var\\rc2k_hook_picture_c.png",
        b"var\\rc2k_hook_picture_c.pcx",
        b"var\\rc2k_hook_picture_a.xyz",
    ];
    for setting in [0u32, 1, 2] {
        for path in paths {
            // The chatzone picture is the game's own and of a size the test has no room for.
            if setting == 1 && path.ends_with(b".png") && path != paths[2] && path != paths[3] {
                continue;
            }
            let run = |rust: bool| {
                let written = reader_regions();
                restore(&written, &vec![0xAA; written.iter().map(|r| r.1).sum()]);
                unsafe {
                    std::ptr::write_bytes(picture(), 0x5A, PICTURE_BYTES);
                    game_write(WIDTH_RVA, 800u32);
                    game_write(HEIGHT_RVA, 600u32);
                    game_write(ROW_BYTES_RVA, 0xFFFF_FFFDu32);
                    game_write(FLAGS_RVA, 0u32);
                    game_write(crate::gfx::pcx::WIDTH_LIMIT_RVA, 0x800u32);
                    game_write(crate::gfx::pcx::HEIGHT_LIMIT_RVA, 0x800u32);
                    game_write(PNG_SETTING_RVA, setting);
                }
                let mut buffer = path.to_vec();
                buffer.push(0);
                let before = Regs {
                    eax: buffer.as_mut_ptr() as u32,
                    ..Regs::SENTINEL
                };
                let worked = if rust {
                    unsafe { read_picture_file(buffer.as_mut_ptr()) }.ok()
                } else {
                    let mut ro = before;
                    unsafe { call_game(&mut ro, original) };
                    ro.preserved_except(&before, 0).then_some(!ro.carry())
                };
                let picture_now =
                    unsafe { std::slice::from_raw_parts(picture(), PICTURE_BYTES) }.to_vec();
                // After a failure what is left of the attempts is not compared.
                let state = (worked == Some(true)).then(|| (left_behind(), picture_now));
                (worked, buffer, state)
            };
            let (theirs, mine) = (run(false), run(true));
            t.check(theirs == mine && theirs.0.is_some(), || {
                format!(
                    "setting={setting} path={:?}: original worked={:?} left the path {:?}; rust worked={:?} left {:?}; same picture={}",
                    String::from_utf8_lossy(path),
                    theirs.0,
                    String::from_utf8_lossy(&theirs.1),
                    mine.0,
                    String::from_utf8_lossy(&mine.1),
                    theirs.2 == mine.2
                )
            });
        }
    }
    restore(&regions, &saved);
    last_path.restore();
    let _ = std::fs::remove_file(PNG_FILE);
    let _ = std::fs::remove_file(PCX_FILE);
    t.finish()
}
