//! `files::file_exists` (0x402EBE): opens a file for reading and closes it again.
//!
//! In:  eax = path
//! Out: carry = it could not be opened (or not closed); zero flag = it could; every register
//!      preserved
//! `open_by_mode` with mode 0, then `close`. `g_lastFilePath` is 0 after a file that
//! exists and the resolved path after one that does not. 20 callers.
//! Shadow mode opens the file twice. Refuses what `resolve_path` refuses. The name is a
//! reading.

use crate::errors::BadInput;
use crate::files::handle::{close, open_by_mode};
use crate::files::stream::{LastPath, MISSING_PATH, TEST_PATH, make_test_file, remove_test_file};
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva, shadow};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("files::file_exists", rva(0x402EBE), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- Rust version ----

pub unsafe fn file_exists(path: *const u8) -> Result<bool, BadInput> {
    unsafe {
        Ok(match open_by_mode(path, 0)? {
            Some(file) => close(file),
            None => false,
        })
    }
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    let mine = match unsafe { file_exists(r.eax as *const u8) } {
        Ok(exists) => exists,
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
    let mut o = *r;
    unsafe { call_original(&mut o) };
    if o.carry() == mine || !o.preserved_except(r, 0) {
        HOOK.mismatch(|| {
            format!(
                "path={:#x}: original exists={} rust {mine}",
                r.eax,
                !o.carry()
            )
        });
    }
    r.eflags = o.eflags;
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    if !make_test_file() {
        return t.skip("cannot create a test file in the game's var folder");
    }
    let saved = LastPath::save();
    for (path, there) in [(TEST_PATH, true), (MISSING_PATH, false)] {
        let before = Regs {
            eax: path.as_ptr() as u32,
            ..Regs::SENTINEL
        };
        let mut ro = before;
        unsafe { call_original(&mut ro) };
        let (zero_o, last_o) = (ro.eflags & 0x40 != 0, LastPath::now());
        let mine = unsafe { file_exists(path.as_ptr()) };
        let last_r = LastPath::now();
        let preserved = ro.preserved_except(&before, 0);
        t.check(
            mine == Ok(!ro.carry()) && mine == Ok(there) && zero_o == there && last_o == last_r && preserved,
            || {
                format!(
                    "a file that is {}: original carry={} zero={zero_o} last={last_o:#x} preserved={preserved}; rust {mine:?} last={last_r:#x}",
                    if there { "there" } else { "missing" },
                    ro.carry()
                )
            },
        );
    }
    saved.restore();
    remove_test_file();
    t.finish()
}
