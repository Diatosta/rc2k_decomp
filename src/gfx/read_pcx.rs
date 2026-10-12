//! `gfx::read_pcx` (0x411A45): decodes a PCX file into the picture buffer.
//!
//! In:  eax = path
//! Out: carry = failed; every register preserved
//! Zeroes the header (`dword_51B5F4`, 0x80 bytes) and reads the file's first 0x80 bytes
//! into it with `files::read_file`; then by the header's number of planes: 1 goes to
//! `read_pcx_8bit`, 3 to `read_pcx_24bit` (each opens the file again), anything else is a
//! failure. Three callers, all in the picture loader `sub_4119F4`.
//! Shadow mode runs the original alone: the two readers it calls are compared in their own
//! hooks. Refuses what `read_file` and the readers refuse. The name is a reading.

use crate::errors::BadInput;
use crate::files::read_file::read_file;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_read, hook_asm, rva, shadow};
use crate::gfx::pcx::{
    HEADER_LEN, HEADER_PLANES, HEADER_RVA, MISSING_PATH, TEST_PATH, remove_test_file,
    write_test_file,
};
use crate::gfx::pcx_reader::ReaderTest;
use crate::gfx::read_pcx_8bit::read_pcx_8bit;
use crate::gfx::read_pcx_24bit::read_pcx_24bit;
use crate::system::zero_memory::zero_memory;
use std::sync::atomic::{AtomicUsize, Ordering};

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("gfx::read_pcx", rva(0x411A45), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- Rust version ----

/// Returns whether the picture was decoded.
pub unsafe fn read_pcx(path: *const u8) -> Result<bool, BadInput> {
    unsafe {
        let header = game_addr(HEADER_RVA) as *mut u8;
        zero_memory(header, HEADER_LEN as u32)?;
        if !read_file(path, header, HEADER_LEN as u32, 0)?.ok {
            return Ok(false);
        }
        match game_read::<u8>(HEADER_RVA + HEADER_PLANES) {
            1 => read_pcx_8bit(path),
            3 => read_pcx_24bit(path),
            _ => Ok(false),
        }
    }
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    match unsafe { read_pcx(r.eax as *const u8) } {
        Ok(worked) => {
            r.set_carry(!worked);
            r.set_zero(worked);
        }
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

// ---- self-test: PCX files of the test's own, of both kinds and of neither ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let Some(test) = ReaderTest::begin() else {
        return t.skip("the picture buffer is not allocated");
    };
    let original = ORIGINAL.load(Ordering::SeqCst);
    let mut two_planes = crate::gfx::read_pcx_24bit::test_file(8, 4, 2, 8, 3);
    two_planes.extend([0u8; 0x300]);
    let files = [
        (
            "one plane",
            crate::gfx::read_pcx_8bit::test_file(21, 9, 22, 5),
        ),
        (
            "three planes",
            crate::gfx::read_pcx_24bit::test_file(21, 9, 3, 22, 5),
        ),
        ("two planes", two_planes),
        ("less than a header", vec![1u8; 0x50]),
        ("an empty file", Vec::new()),
    ];
    for (what, content) in files {
        if !write_test_file(&content) {
            test.end();
            return t.skip("cannot create a test file in the game's var folder");
        }
        test.check(&mut t, what, original, TEST_PATH, read_pcx);
    }
    remove_test_file();
    test.check(&mut t, "a missing file", original, MISSING_PATH, read_pcx);
    test.end();
    t.finish()
}
