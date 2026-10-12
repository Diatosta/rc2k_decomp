//! `gfx::load_jade` (0x4075F3): loads the glyphs of the fatal error's own font,
//! `files\d\gfx\jade.bun`, and works out its colour.
//!
//! In:  nothing
//! Out: every register preserved
//! Up to 0x1400 bytes of the file to `unk_4F52C4` (the glyph table of the four font
//! objects at 0x51DE30 on); a file that cannot be read is fatal error 0x80F0B0D1, one
//! that fills all 0x1400 bytes 0x80F0B0D2. Then `dword_4F66C4` (those fonts' palette) =
//! white in the surface's format (`sub_407629`: `gfx::components_to_pixel` of 0xFFFFFF).
//! One caller, `system::main_loop`.
//! Self-test: a redirected copy that reads files of the test's.
//! Refuses what `read_file` refuses.

use crate::errors::{FatalError, Refused};
use crate::files::read_file::read_file;
use crate::framework::{RawHook, Regs, SelfTest, call_game, game_addr, game_write, hook_asm, rva};
use crate::gfx::components_to_pixel::components_to_pixel;
use crate::gfx::pixel_format::{SCRATCH_RVA, surface_format};
use crate::install::stand_ins::{WENT_FATAL, fatal_landing};
use crate::redirect::RedirectedCopy;
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, dispatch_state, restore};
use std::sync::atomic::{AtomicUsize, Ordering};

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("gfx::load_jade", rva(0x4075F3), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

const PATH_RVA: usize = rva(0x4F66C8); // "files\d\gfx\jade.bun"
const GLYPHS_RVA: usize = rva(0x4F52C4); // unk_4F52C4
const ROOM: u32 = 0x1400;
const COLOUR_RVA: usize = rva(0x4F66C4); // dword_4F66C4

// ---- Rust version ----

pub unsafe fn load_jade(path: *const u8) -> Result<(), Refused> {
    unsafe {
        let read = read_file(path, game_addr(GLYPHS_RVA) as *mut u8, ROOM, 0)?;
        if !read.ok {
            return Err(FatalError {
                code: 0x80F0_B0D1,
                message: "",
            }
            .into());
        }
        if read.bytes_read == Some(ROOM) {
            return Err(FatalError {
                code: 0x80F0_B0D2,
                message: "",
            }
            .into());
        }
        game_write(
            COLOUR_RVA,
            components_to_pixel(&surface_format(), 0xFF_FFFF),
        );
        Ok(())
    }
}

// ---- hook ----

fn regions() -> [Region; 4] {
    [
        (game_addr(GLYPHS_RVA) as u32, ROOM as usize),
        at(COLOUR_RVA),
        at(SCRATCH_RVA),
        at(crate::files::read_file::LAST_FILE_PATH_RVA),
    ]
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { load_jade(game_addr(PATH_RVA) as *const u8) }?;
    Ok(Left::nothing())
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

// ---- self-test ----

const ENTRY_BYTES: [u8; 8] = [0x60, 0x8D, 0x05, 0xC8, 0x66, 0x4F, 0x00, 0x8D];
/// The copy takes `sub_407629` along, which follows the function at once: 0x36 + 0x13
/// bytes.
const LEN: usize = 0x49;
/// `files::read_file`, and in `sub_407629` `gfx::components_to_pixel`; the call of
/// `sub_407629` itself stays inside the copy.
const CALLS: [usize; 2] = [0x14, 0x3D];
const PATH_OPERAND: usize = 0x03;
const FATAL_JUMP_OPERANDS: [usize; 2] = [0x1B, 0x26];
const TEST_FILE: &str = "var\\rc2k_hook_jade.bun";
const TEST_PATH: &[u8] = b"var\\rc2k_hook_jade.bun\0";

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let copy = unsafe {
        RedirectedCopy::new(game_addr(rva(0x4075F3)), LEN, &ENTRY_BYTES, &CALLS, &|_| {
            None
        })
    };
    let copy = match copy {
        Ok(copy) => copy,
        Err(why) => return t.skip(why),
    };
    unsafe {
        let code = std::slice::from_raw_parts(copy.addr() as *const u8, LEN);
        let named = u32::from_le_bytes(code[PATH_OPERAND..PATH_OPERAND + 4].try_into().unwrap());
        if named != game_addr(PATH_RVA) as u32 {
            return t.skip("the original is not what the test was made from");
        }
        copy.patch_u32(PATH_OPERAND, TEST_PATH.as_ptr() as u32);
        for operand in FATAL_JUMP_OPERANDS {
            copy.retarget(operand, fatal_landing as *const () as usize);
        }
    }
    let regions = regions();
    let saved = capture(&regions);
    let pattern = |len: usize| -> Vec<u8> { (0..len).map(|n| (n * 13 + 5) as u8).collect() };
    let files: [(&str, Option<Vec<u8>>); 6] = [
        ("a small file", Some(pattern(0x321))),
        ("one byte under the room", Some(pattern(0x13FF))),
        ("the room exactly", Some(pattern(0x1400))),
        ("more than the room", Some(pattern(0x2000))),
        ("empty", Some(Vec::new())),
        ("not there", None),
    ];
    let mut made = true;
    for (what, file) in files {
        let _ = std::fs::remove_file(TEST_FILE);
        if let Some(file) = &file
            && std::fs::write(TEST_FILE, file).is_err()
        {
            made = false;
            break;
        }
        let before = Regs::SENTINEL;
        let run = |rust_side: bool| unsafe {
            restore(&regions, &vec![0xAA; saved.len()]);
            WENT_FATAL.store(0, Ordering::SeqCst);
            let ended = if rust_side {
                match load_jade(TEST_PATH.as_ptr()) {
                    Ok(()) => 0,
                    Err(Refused::Fatal(error)) => error.code,
                    Err(Refused::BadInput(_)) => 3,
                }
            } else {
                let mut ro = before;
                call_game(&mut ro, copy.addr());
                match WENT_FATAL.load(Ordering::SeqCst) {
                    0 => !ro.preserved_except(&before, 0) as u32,
                    // Which of the two stubs it was going to is told by the file.
                    _ => 2,
                }
            };
            // The last path's address is not compared.
            let mut left = capture(&regions);
            let end = left.len() - 4;
            left[end..].fill(0);
            (ended, left)
        };
        let (theirs, mine) = (run(false), run(true));
        let expected = match &file {
            None => 0x80F0_B0D1,
            Some(file) if file.len() >= 0x1400 => 0x80F0_B0D2,
            Some(_) => 0,
        };
        let same_end = match theirs.0 {
            2 => mine.0 == expected && expected != 0,
            end => end == mine.0 && expected == 0,
        };
        t.check(same_end && theirs.1 == mine.1, || {
            format!(
                "{what}: original ended {:#x} (2: a fatal stub), rust ended {:#x}, expected {expected:#x}; first different byte left {:?}",
                theirs.0,
                mine.0,
                theirs.1.iter().zip(&mine.1).position(|(a, b)| a != b),
            )
        });
    }
    restore(&regions, &saved);
    let _ = std::fs::remove_file(TEST_FILE);
    if !made {
        return t.skip("cannot create a test file in the game's var folder");
    }
    t.finish()
}
