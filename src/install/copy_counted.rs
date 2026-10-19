//! `install::copy_counted` (0x4139DA): `dword_51BAE8` (the files copied so far) raised by one
//! and the file copied (`files::copy_file`, eax the source esi, ebx the destination edi).
//!
//! In:  esi the source path, edi the destination path
//! Out: every register kept (`pusha` / `popa`)
//! Callers `install::install_files`, `install::install_clio`. The copy through `part`.
//! Ungated. Shadow mode runs the original alone (one copy). Self-test: a copy whose one
//! call goes to a stand-in, against the Rust version.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::install::installer::{COPY_FILE, FILES_COPIED, Part, at, both, g, p, quiet, region};
use crate::race::viewer::{Saved, dispatch_parts};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("install::copy_counted", rva(0x4139DA), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL,
    ungated
);

pub fn copy_counted(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    p(FILES_COPIED, g(FILES_COPIED).wrapping_add(1));
    let mut r = Regs {
        eax: entry.esi,
        ebx: entry.edi,
        ..*entry
    };
    part(at(COPY_FILE), &mut r)?;
    Ok(*entry)
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_parts(&HOOK, r, call_original, &|r| {
        copy_counted(r, &mut crate::install::installer::game_part)
    });
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // pusha; inc dword_51BAE8
    let copy = match crate::install::installer::copy(
        0x4139DA,
        0x12,
        &[0x60, 0xFF, 0x05, 0xE8, 0xBA, 0x51, 0x00],
        &[(0xB, 0x402C28)],
        &[0x402C28],
        &[],
        &[],
        &[],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    quiet();
    let regions = [region(FILES_COPIED, 4)];
    let saved = Saved::new(&regions);
    let src = b"from.txt\0";
    let dst = b"to.txt\0";
    for count in [0u32, 5] {
        both(
            &mut t,
            &format!("count {count}"),
            &copy,
            Regs {
                esi: src.as_ptr() as u32,
                edi: dst.as_ptr() as u32,
                ..Regs::SENTINEL
            },
            &regions,
            0,
            &[],
            &|| p(FILES_COPIED, count),
            &|r, part, _| copy_counted(r, part),
        );
    }
    drop(saved);
    t.finish()
}
