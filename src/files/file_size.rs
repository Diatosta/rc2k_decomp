//! `files::file_size` (0x402ED4): the size of a file by its path.
//!
//! In:  eax = path
//! Out: eax = the size, carry clear; or carry set (eax is then 0xFFFFFFFF, unless only the
//!      close failed); everything else preserved
//! `open_by_mode` with mode 0, `files::handle_size` (0x402EA9, `GetFileSize`), `close`. Two
//! callers, the PNG reader and `sub_4C6625`.
//! Shadow mode opens the file twice. Refuses what `resolve_path` refuses. The name is a
//! reading.

use crate::errors::BadInput;
use crate::files::handle::{NO_HANDLE, close, handle_size, open_by_mode};
use crate::files::stream::{
    LastPath, MISSING_PATH, TEST_LEN, TEST_PATH, make_test_file, remove_test_file,
};
use crate::framework::{EAX, RawHook, Regs, SelfTest, hook_asm, rva, shadow};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("files::file_size", rva(0x402ED4), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- Rust version ----

/// (eax, whether it worked).
pub unsafe fn file_size(path: *const u8) -> Result<(u32, bool), BadInput> {
    unsafe {
        let Some(file) = open_by_mode(path, 0)? else {
            return Ok((NO_HANDLE, false));
        };
        let size = handle_size(file);
        let closed = close(file);
        Ok((size.unwrap_or(u32::MAX), closed && size.is_some()))
    }
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    let mine = match unsafe { file_size(r.eax as *const u8) } {
        Ok(mine) => mine,
        Err(e) => {
            unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) };
            return;
        }
    };
    if !shadow() {
        r.eax = mine.0;
        r.set_carry(!mine.1);
        return;
    }
    let mut o = *r;
    unsafe { call_original(&mut o) };
    if (o.eax, !o.carry()) != mine || !o.preserved_except(r, EAX) {
        HOOK.mismatch(|| {
            format!(
                "path={:#x}: original eax={:#x} carry={}; rust {mine:x?}",
                r.eax,
                o.eax,
                o.carry()
            )
        });
    }
    r.eax = o.eax;
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
    for (path, expected) in [(TEST_PATH, Some(TEST_LEN as u32)), (MISSING_PATH, None)] {
        let before = Regs {
            eax: path.as_ptr() as u32,
            ..Regs::SENTINEL
        };
        let mut ro = before;
        unsafe { call_original(&mut ro) };
        let last_o = LastPath::now();
        let mine = unsafe { file_size(path.as_ptr()) };
        let last_r = LastPath::now();
        let preserved = ro.preserved_except(&before, EAX);
        let ok = mine == Ok((ro.eax, !ro.carry()))
            && expected.is_none_or(|size| mine == Ok((size, true)))
            && (expected.is_some() || ro.carry())
            && last_o == last_r
            && preserved;
        t.check(ok, || {
            format!(
                "a file of {expected:?} bytes: original eax={:#x} carry={} last={last_o:#x} preserved={preserved}; rust {mine:x?} last={last_r:#x}",
                ro.eax,
                ro.carry()
            )
        });
    }
    saved.restore();
    remove_test_file();
    t.finish()
}
