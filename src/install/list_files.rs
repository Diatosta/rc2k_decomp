//! `install::list_files` (0x4138B2): the files matching a pattern added to the files-to-copy
//! table. The pattern (eax) is resolved (`files::resolve_path`) into edx and searched
//! (`files::find_first` / `files::find_next_wanted`); each found name is appended at
//! `dword_51BB58` (the cursor) behind its folder (`files::append_directory` of edx) with a
//! NUL, the cursor advanced and `dword_51BB4C` (the count) raised. More than 0x40000 bytes
//! of names is fatal error 0x80AF2206. The search is closed at the end.
//!
//! In:  eax the pattern
//! Out: every register kept (`pusha` / `popa`)
//! Caller `install::run_script` (opcode 3). Its resolve, search and `append_directory`
//! through `part`; the appending here. Shadow mode runs the original alone. Self-test: a
//! copy whose disk calls go to the fake file system, against the Rust version.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::install::installer::{
    CLOSE_SEARCH, FILE_COUNT, FILE_CURSOR, FILES, FIND_FIRST, FIND_NEXT, Part, RESOLVE_PATH, a,
    append_byte, at, fatal, g, p, region, str_append,
};
use crate::race::viewer::Saved;
use rc2k_core::install::script;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("install::list_files", rva(0x4138B2), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const FIND_NAME: usize = 0x4E05A8 + 0x2C; // FindFileData.cFileName

pub fn list_files(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    part(at(RESOLVE_PATH), &mut r)?;
    r.edx = r.eax;
    part(at(FIND_FIRST), &mut r)?;
    if r.carry() {
        part(at(CLOSE_SEARCH), &mut r)?;
        return Ok(*entry);
    }
    loop {
        r.ebx = g(FILE_CURSOR);
        // append_directory(edx = the pattern's path, ebx = the cursor): pure, called direct
        r = unsafe { crate::files::append_directory::append_directory_regs(&r) }?;
        r.eax = a(FIND_NAME);
        str_append(&mut r);
        append_byte(&mut r, 0);
        p(FILE_CURSOR, r.ebx);
        p(FILE_COUNT, g(FILE_COUNT).wrapping_add(1));
        // `sub ebx, [table]`: find_next gets the bytes used in ebx.
        r.ebx = r.ebx.wrapping_sub(g(FILES));
        // Converted: `rc2k_core::install::script::room_left`.
        script::room_left(r.ebx, script::FILES_ROOM, script::FILES_FULL).map_err(fatal)?;
        part(at(FIND_NEXT), &mut r)?;
        if r.carry() {
            break;
        }
    }
    part(at(CLOSE_SEARCH), &mut r)?;
    Ok(Regs { ..*entry })
}

extern "C" fn dispatch(r: &mut Regs) {
    crate::race::viewer::dispatch_parts(&HOOK, r, call_original, &|r| {
        list_files(r, &mut crate::install::installer::game_part)
    });
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    crate::race::viewer::regs_entry(r, &|r| {
        list_files(r, &mut crate::install::installer::game_part)
    })
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    use crate::install::fakefs::{DIR, Entry, World, answer, set_world};
    // pusha; call resolve_path
    let copy = match crate::install::installer::copy(
        0x4138B2,
        0x56,
        &[0x60, 0xE8, 0x4E, 0xF9, 0xFE, 0xFF],
        &[
            (0x1, 0x403206),
            (0x8, 0x402FC2),
            (0x20, 0x401F7D),
            (0x25, 0x401F16),
            (0x15, 0x403070),
            (0x48, 0x402FF6),
            (0x4F, 0x40301F),
        ],
        &[0x403206, 0x402FC2, 0x402FF6, 0x40301F],
        &[],
        &[(0x42, 0x80AF_2206)],
        &[],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    let mut table = vec![0xAAu8; 0x4000];
    let tptr = table.as_mut_ptr() as u32;
    let regions = [
        region(FILES, 4),
        region(FILE_CURSOR, 4),
        region(FILE_COUNT, 4),
        region(0x4E05A8, 0x140),
        region(0x4F52B0, 4),
        (tptr, 0x4000),
    ];
    let saved = Saved::new(&regions);
    for pattern in [
        &b"files\\b\\*.ro3\0"[..],
        b"files\\d\\gfx\\*.*\0",
        b"empty\\*.*\0",
    ] {
        let prepare = || {
            answer_world(pattern);
            p(FILES, tptr);
            p(FILE_CURSOR, tptr);
            p(FILE_COUNT, 0);
            crate::install::installer::fill(tptr, 0x4000, 0xAA);
        };
        let mut src = pattern.to_vec();
        crate::install::installer::both(
            &mut t,
            &format!("pattern {}", crate::strings::utils::bytes_str(pattern)),
            &copy,
            Regs {
                eax: src.as_mut_ptr() as u32,
                ..Regs::SENTINEL
            },
            &regions,
            0,
            &[],
            &prepare,
            &|r, part, _| list_files(r, part),
        );
        let _ = &mut src;
    }
    fn answer_world(pattern: &[u8]) {
        crate::install::installer::answer(Box::new(answer));
        let mut w = World::default();
        let key = String::from_utf8_lossy(&pattern[..pattern.len() - 1]).into_owned();
        if key.starts_with("files\\b") {
            w.listings.insert(
                key,
                vec![
                    Entry {
                        name: "obj0570.ro3",
                        attrs: 0,
                    },
                    Entry {
                        name: "obj0571.ro3",
                        attrs: 0,
                    },
                ],
            );
        } else if key.starts_with("files\\d") {
            w.listings.insert(
                key,
                vec![
                    Entry {
                        name: "backdrop.pcx",
                        attrs: 0,
                    },
                    Entry {
                        name: "sub",
                        attrs: DIR,
                    },
                    Entry {
                        name: "install.bun",
                        attrs: 0,
                    },
                ],
            );
        }
        set_world(w);
    }
    crate::install::installer::quiet();
    drop(saved);
    drop(table);
    t.finish()
}
