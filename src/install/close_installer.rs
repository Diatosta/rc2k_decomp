//! `install::close_installer` (0x414490): the installer's memory list (`stru_51BAFC`) freed
//! (`memory::free_list`) and its folder- and file-name table pointers (`dword_51BB40`,
//! `dword_51BB44`) zeroed.
//!
//! In:  nothing
//! Out: every register kept (`pusha` / `popa`)
//! Caller `install::run_installer`. The free through `part`. Shadow mode runs the original
//! alone (memory cannot be freed twice). Self-test: a copy whose one call goes to a
//! stand-in, against the Rust version.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::install::installer::{
    BUFFERS, DIRS, FILES, FREE_LIST, Part, a, at, both, p, quiet, region,
};
use crate::race::viewer::{Saved, dispatch_parts};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("install::close_installer", rva(0x414490), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

pub fn close_installer(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = Regs {
        ebx: a(BUFFERS),
        ..*entry
    };
    part(at(FREE_LIST), &mut r)?;
    p(DIRS, 0);
    p(FILES, 0);
    Ok(*entry)
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_parts(&HOOK, r, call_original, &|r| {
        close_installer(r, &mut crate::install::installer::game_part)
    });
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // pusha; lea ebx, stru_51BAFC
    let copy = match crate::install::installer::copy(
        0x414490,
        0x1A,
        &[0x60, 0x8D, 0x1D, 0xFC, 0xBA, 0x51, 0x00],
        &[(0x7, 0x4028F5)],
        &[0x4028F5],
        &[],
        &[],
        &[],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    quiet();
    let regions = [region(DIRS, 4), region(FILES, 4)];
    let saved = Saved::new(&regions);
    both(
        &mut t,
        "free and zero",
        &copy,
        Regs::SENTINEL,
        &regions,
        0,
        &[],
        &|| {
            p(DIRS, 0x1234);
            p(FILES, 0x5678);
        },
        &|r, part, _| close_installer(r, part),
    );
    drop(saved);
    t.finish()
}
