//! `saves::save_options` (0x4AAE57): writes the game's kept options, `var\ral.dat`.
//!
//! In:  nothing
//! Out: every register preserved; carry = not written
//! The file is put together in the scratch block at 0x93E8B0: "R5CI", 0, 0x50004, the
//! time (`system::packed_local_time`), the 69 pieces of `saves::load_options` in the
//! same order, each copied from its place, and "####"; then `files::write_file`.
//! One caller, `saves::save_game_data`.
//! Shadow mode writes the file twice (Rust, then the original) and compares what was put
//! together, but for the time. Self-test: a redirected copy that writes a file of the
//! test's; the two files are compared but for the time.
//! Refuses what `write_file` refuses.

use crate::errors::Refused;
use crate::files::write_file::write_file;
use crate::framework::{
    RawHook, Regs, SelfTest, call_game, game_addr, game_write, hook_asm, rva, shadow,
};
use crate::redirect::RedirectedCopy;
use crate::saves::load_options::OPTIONS;
use crate::service::harness::{capture, restore};
use crate::settings::TEXT_RVA;
use crate::system::packed_local_time::packed_local_time;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("saves::save_options", rva(0x4AAE57), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

const PATH_RVA: usize = rva(0x5295D8); // "var\ral.dat"
const MAGIC: u32 = 0x4943_3552; // "R5CI"
const VERSION: u32 = 0x50004;
const END_MARK: u32 = 0x2323_2323; // "####"
const HEAD_LEN: usize = 0x10;
/// Where the time is in the head.
const TIME: std::ops::Range<usize> = 0xC..0x10;

/// The whole file's length.
pub fn file_len() -> usize {
    HEAD_LEN + OPTIONS.iter().map(|&(_, len)| len).sum::<usize>() + 4
}

// ---- Rust version ----

/// True if the file was written.
pub unsafe fn save_options(path: *const u8) -> Result<bool, Refused> {
    unsafe {
        let file = game_addr(TEXT_RVA) as *mut u8;
        game_write(TEXT_RVA, MAGIC);
        game_write(TEXT_RVA + 4, 0u32);
        game_write(TEXT_RVA + 8, VERSION);
        game_write(TEXT_RVA + 0xC, packed_local_time());
        let mut cursor = file.add(HEAD_LEN);
        for (place, len) in OPTIONS {
            std::ptr::copy(game_addr(rva(place as usize)) as *const u8, cursor, len);
            cursor = cursor.add(len);
        }
        (cursor as *mut u32).write_unaligned(END_MARK);
        Ok(write_file(path, file, file_len() as u32)?.ok)
    }
}

// ---- hook ----

/// What was put together, without the time.
fn put_together() -> Vec<u8> {
    let mut file = capture(&[(game_addr(TEXT_RVA) as u32, file_len())]);
    file[TIME].fill(0);
    file
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    let mine = match unsafe { save_options(game_addr(PATH_RVA) as *const u8) } {
        Ok(written) => written,
        Err(e) => {
            unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) };
            return;
        }
    };
    if !shadow() {
        r.set_carry(!mine);
        r.set_zero(mine);
        return;
    }
    let file = put_together();
    let before = *r;
    unsafe { call_original(r) };
    if put_together() != file || r.carry() == mine || !r.preserved_except(&before, 0) {
        let theirs = put_together();
        HOOK.mismatch(|| {
            format!(
                "original carry={}; rust written={mine}; first different byte of the file {:?}",
                r.carry(),
                theirs.iter().zip(&file).position(|(a, b)| a != b),
            )
        });
    }
}

// ---- self-test ----

const LEN: usize = 0x2FE;
const ENTRY_BYTES: [u8; 8] = [0x60, 0x8D, 0x1D, 0xB0, 0xE8, 0x93, 0x00, 0xB9];
/// `system::zero_memory`, `system::packed_local_time`, `files::write_file`.
const CALLS: [usize; 3] = [0x0C, 0x1E, 0x2F7];
const PATH_OPERAND: usize = 0x2EF;
const TEST_FILES: [&str; 2] = ["var\\rc2k_hook_ral_a.dat", "var\\rc2k_hook_ral_b.dat"];
const TEST_PATHS: [&[u8]; 2] = [b"var\\rc2k_hook_ral_a.dat\0", b"var\\rc2k_hook_ral_b.dat\0"];

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let copy = unsafe {
        RedirectedCopy::new(game_addr(rva(0x4AAE57)), LEN, &ENTRY_BYTES, &CALLS, &|_| {
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
        copy.patch_u32(PATH_OPERAND, TEST_PATHS[0].as_ptr() as u32);
    }
    let scratch = [(game_addr(TEXT_RVA) as u32, file_len())];
    let saved = capture(&scratch);
    let last = crate::files::stream::LastPath::save();
    for file in TEST_FILES {
        let _ = std::fs::remove_file(file);
    }

    let before = Regs::SENTINEL;
    restore(&scratch, &vec![0xAA; saved.len()]);
    let mut ro = before;
    unsafe { call_game(&mut ro, copy.addr()) };
    let theirs = (
        !ro.carry() && ro.preserved_except(&before, 0),
        put_together(),
    );
    restore(&scratch, &vec![0xAA; saved.len()]);
    let mine = (
        matches!(unsafe { save_options(TEST_PATHS[1].as_ptr()) }, Ok(true)),
        put_together(),
    );
    let written = TEST_FILES.map(|file| {
        std::fs::read(file).ok().map(|mut bytes| {
            if bytes.len() >= HEAD_LEN {
                bytes[TIME].fill(0);
            }
            bytes
        })
    });
    t.check(
        theirs == mine
            && theirs.0
            && written[0].is_some()
            && written[0] == written[1]
            && written[0].as_ref().is_some_and(|file| *file == theirs.1),
        || {
            format!(
                "original written={} file of {:?} bytes; rust written={} file of {:?} bytes; same put together={}; files the same={}",
                theirs.0,
                written[0].as_ref().map(Vec::len),
                mine.0,
                written[1].as_ref().map(Vec::len),
                theirs.1 == mine.1,
                written[0] == written[1],
            )
        },
    );

    restore(&scratch, &saved);
    last.restore();
    for file in TEST_FILES {
        let _ = std::fs::remove_file(file);
    }
    t.finish()
}
