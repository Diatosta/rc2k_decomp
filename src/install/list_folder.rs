//! `install::list_folder` (0x413876): one folder path added to the folders-to-make table.
//! The path (eax) is resolved (`files::resolve_path`), appended with a NUL at
//! `dword_51BB54` (the cursor), the cursor and scan pointer advanced, and `dword_51BB48`
//! (the count) raised. More than 0x10000 bytes of names is fatal error 0x80AF2205.
//!
//! In:  eax the folder path
//! Out: every register kept (`pusha` / `popa`)
//! Caller `install::run_script` (opcode 1). Its resolve through `part`; the appending here.
//! Shadow mode runs the original alone. Self-test: a copy whose resolve goes to a stand-in,
//! against the Rust version, on paths of the test's; the table, cursors and count compared.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::install::installer::{
    DIR_COUNT, DIR_CURSOR, DIR_SCAN, DIRS, Part, RESOLVE_PATH, at, fatal, g, p, region, str_append,
};
use crate::race::viewer::Saved;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("install::list_folder", rva(0x413876), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const DIRS_ROOM: u32 = 0x10000;

pub fn list_folder(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    part(at(RESOLVE_PATH), &mut r)?;
    r.ebx = g(DIR_CURSOR);
    str_append(&mut r);
    crate::install::installer::append_byte(&mut r, 0);
    p(DIR_CURSOR, r.ebx);
    p(DIR_SCAN, r.ebx);
    p(DIR_COUNT, g(DIR_COUNT).wrapping_add(1));
    if r.ebx.wrapping_sub(g(DIRS)) >= DIRS_ROOM {
        return Err(fatal(0x80AF_2205));
    }
    Ok(Regs { ..*entry })
}

extern "C" fn dispatch(r: &mut Regs) {
    crate::race::viewer::dispatch_parts(&HOOK, r, call_original, &|r| {
        list_folder(r, &mut crate::install::installer::game_part)
    });
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // pusha; call resolve_path
    let copy = match crate::install::installer::copy(
        0x413876,
        0x3C,
        &[0x60, 0xE8, 0x8A, 0xF9, 0xFE, 0xFF],
        &[(0x1, 0x403206)],
        &[0x403206],
        &[],
        &[(0x34, 0x80AF_2205)],
        &[],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    let mut table = vec![0xAAu8; 0x2000];
    let tptr = table.as_mut_ptr() as u32;
    let regions = [
        region(DIRS, 4),
        region(DIR_CURSOR, 4),
        region(DIR_SCAN, 4),
        region(DIR_COUNT, 4),
        (tptr, 0x2000),
    ];
    let saved = Saved::new(&regions);
    for (count, off, path) in [
        (0u32, 0u32, &b"files\\b\0"[..]),
        (3, 0x40, b"files\\d\\gfx\0"),
        (1, 0x10, b"\0"),
    ] {
        let prepare = || {
            crate::install::installer::answer(Box::new(crate::install::fakefs::answer));
            crate::install::fakefs::set_world(crate::install::fakefs::World::default());
            p(DIRS, tptr);
            p(DIR_CURSOR, tptr + off);
            p(DIR_SCAN, 0);
            p(DIR_COUNT, count);
        };
        let mut src = path.to_vec();
        crate::install::installer::both(
            &mut t,
            &format!(
                "count {count}, path {}",
                crate::strings::utils::bytes_str(path)
            ),
            &copy,
            Regs {
                eax: src.as_mut_ptr() as u32,
                ..Regs::SENTINEL
            },
            &regions,
            0,
            &[],
            &prepare,
            &|r, part, _| list_folder(r, part),
        );
        let _ = &mut src;
    }
    crate::install::installer::quiet();
    drop(saved);
    drop(table);
    t.finish()
}
