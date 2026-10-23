//! `install::list_folder_tree` (0x41371C): a folder and everything under it added to the
//! tables, breadth first. The skip mask (`FindFileData+0x13E`) is set to 0 for the walk
//! (and put back at the end). The folder (eax) is resolved; if not empty it goes into the
//! folders table (`dword_51BB54`). Then, starting with that folder and working through every
//! folder recorded from `dword_51BB50` to `dword_51BB54`, each is searched (`<folder>\*.*`
//! built in `byte_94E8B0`): a sub-folder (not `.`/`..`) is appended to the folders table
//! behind its parent, a file to the files table (`dword_51BB58`); the folders table is the
//! queue. Overflow of either table is fatal error 0x80AF2201 or 0x80AF2202.
//!
//! In:  eax the folder
//! Out: every register kept (`pusha` / `popa`)
//! Caller `install::run_script` (opcode 4). Its resolve and search through `part`; the
//! string work here. Shadow mode runs the original alone. Self-test: a copy whose disk
//! calls go to the fake file system, against the Rust version, on a small tree.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::install::installer::{
    CLOSE_SEARCH, DIR_COUNT, DIR_CURSOR, DIR_SCAN, DIRS, FILE_COUNT, FILE_CURSOR, FILES,
    FIND_FIRST, FIND_NEXT, Part, RESOLVE_PATH, a, append_byte, at, fatal, g, p, rb, region,
    str_append, wb,
};
use crate::race::viewer::Saved;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("install::list_folder_tree", rva(0x41371C), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const SEARCH_BASE: usize = 0x94E8B0; // byte_94E8B0
const DOT: usize = 0x5295B7; // "."
const DOTDOT: usize = 0x5295B9; // ".."
const STAR: usize = 0x5295BC; // "*.*"
const SKIP_MASK: usize = 0x4E06E6; // FindFileData+0x13E
const FIND_NAME: usize = 0x4E05A8 + 0x2C; // FindFileData.cFileName
const FIND_ATTRS: usize = 0x4E05A8; // FindFileData.dwFileAttributes
const DIRS_ROOM: u32 = 0x10000;
const FILES_ROOM: u32 = 0x40000;

fn same_name(a1: u32, a2: u32) -> bool {
    unsafe { crate::strings::same_text::same_text(a1 as *const u8, a2 as *const u8) }
}

pub fn list_folder_tree(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    let saved_skip = g(SKIP_MASK);
    p(SKIP_MASK, 0);
    let result = (|| -> Result<(), Refused> {
        part(at(RESOLVE_PATH), &mut r)?;
        if rb(r.eax) != 0 {
            r.ebx = g(DIR_CURSOR);
            str_append(&mut r);
            append_byte(&mut r, 0);
            p(DIR_CURSOR, r.ebx);
            p(DIR_SCAN, r.ebx);
            p(DIR_COUNT, g(DIR_COUNT).wrapping_add(1));
        }
        loop {
            // list the folder named in eax
            let base = a(SEARCH_BASE);
            r.ebx = base;
            r.edx = base;
            str_append(&mut r); // the folder path
            if rb(r.ebx.wrapping_sub(1)) != b'\\' {
                wb(r.ebx, b'\\');
                r.ebx += 1;
            }
            r.eax = a(STAR);
            str_append(&mut r);
            append_byte(&mut r, 0);
            r.eax = base;
            part(at(FIND_FIRST), &mut r)?;
            if !r.carry() {
                loop {
                    let is_dir = g(FIND_ATTRS) & 0x10 != 0;
                    if is_dir {
                        // eax "." then "..", ebx the found name (`same_text` keeps them)
                        r.ebx = a(FIND_NAME);
                        r.eax = a(DOT);
                        let mut real = !same_name(r.eax, r.ebx);
                        if real {
                            r.eax = a(DOTDOT);
                            real = !same_name(r.eax, r.ebx);
                        }
                        if real {
                            r.ebx = g(DIR_CURSOR);
                            r = unsafe {
                                crate::files::append_directory::append_directory_regs(&r)
                            }?;
                            r.eax = a(FIND_NAME);
                            str_append(&mut r);
                            append_byte(&mut r, 0);
                            p(DIR_CURSOR, r.ebx);
                            p(DIR_COUNT, g(DIR_COUNT).wrapping_add(1));
                            r.ebx = r.ebx.wrapping_sub(g(DIRS)); // `sub ebx, g_dirNames`
                            if r.ebx >= DIRS_ROOM {
                                return Err(fatal(0x80AF_2201));
                            }
                        }
                    } else {
                        r.ebx = g(FILE_CURSOR);
                        r = unsafe { crate::files::append_directory::append_directory_regs(&r) }?;
                        r.eax = a(FIND_NAME);
                        str_append(&mut r);
                        append_byte(&mut r, 0);
                        p(FILE_CURSOR, r.ebx);
                        p(FILE_COUNT, g(FILE_COUNT).wrapping_add(1));
                        r.ebx = r.ebx.wrapping_sub(g(FILES)); // `sub ebx, g_fileNames`
                        if r.ebx >= FILES_ROOM {
                            return Err(fatal(0x80AF_2202));
                        }
                    }
                    part(at(FIND_NEXT), &mut r)?;
                    if r.carry() {
                        break;
                    }
                }
            }
            part(at(CLOSE_SEARCH), &mut r)?;
            // dequeue the next folder from the scan pointer
            r.eax = g(DIR_SCAN);
            if r.eax >= g(DIR_CURSOR) {
                break;
            }
            r.ebx = r.eax;
            r.ecx = unsafe { crate::strings::text_length::text_length(r.ebx as *const u8) }?;
            if r.ecx == 0 {
                break;
            }
            r.ebx = r.ebx.wrapping_add(r.ecx).wrapping_add(1);
            p(DIR_SCAN, r.ebx);
        }
        Ok(())
    })();
    p(SKIP_MASK, saved_skip);
    result?;
    Ok(Regs { ..*entry })
}

extern "C" fn dispatch(r: &mut Regs) {
    crate::race::viewer::dispatch_parts(&HOOK, r, call_original, &|r| {
        list_folder_tree(r, &mut crate::install::installer::game_part)
    });
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    crate::race::viewer::regs_entry(r, &|r| {
        list_folder_tree(r, &mut crate::install::installer::game_part)
    })
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    use crate::install::fakefs::{DIR, Entry, World, answer, set_world};
    // pusha; push dword ptr FindFileData+13Eh; mov dword ptr FindFileData+13Eh, 0
    let copy = match crate::install::installer::copy(
        0x41371C,
        0x15A,
        &[
            0x60, 0xFF, 0x35, 0xE6, 0x06, 0x4E, 0x00, 0xC7, 0x05, 0xE6, 0x06, 0x4E, 0x00, 0x00,
            0x00, 0x00, 0x00,
        ],
        // every call: the four disk ones to stand-ins, the string ones to the game's
        &[
            (0x11, 0x403206),
            (0x21, 0x401F7D),
            (0x26, 0x401F16),
            (0x45, 0x401F7D),
            (0x5A, 0x401F7D),
            (0x5F, 0x401F16),
            (0x66, 0x402FC2),
            (0x89, 0x401EDE),
            (0xA0, 0x401EDE),
            (0xAD, 0x403070),
            (0xB8, 0x401F7D),
            (0xBD, 0x401F16),
            (0xE8, 0x403070),
            (0xF3, 0x401F7D),
            (0xF8, 0x401F16),
            (0x11B, 0x402FF6),
            (0x126, 0x40301F),
            (0x13A, 0x4022B8),
        ],
        &[0x403206, 0x402FC2, 0x402FF6, 0x40301F],
        &[],
        &[(0xDA, 0x80AF_2201), (0x115, 0x80AF_2202)],
        &[],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    let mut dirs = vec![0xAAu8; 0x2000];
    let mut files = vec![0xAAu8; 0x2000];
    let dptr = dirs.as_mut_ptr() as u32;
    let fptr = files.as_mut_ptr() as u32;
    let regions = [
        region(DIRS, 4),
        region(DIR_CURSOR, 4),
        region(DIR_SCAN, 4),
        region(DIR_COUNT, 4),
        region(FILES, 4),
        region(FILE_CURSOR, 4),
        region(FILE_COUNT, 4),
        region(SKIP_MASK, 4),
        region(0x4E05A8, 0x140),
        region(0x4F52B0, 4),
        region(SEARCH_BASE, 0x200),
        (dptr, 0x2000),
        (fptr, 0x2000),
    ];
    let saved = Saved::new(&regions);
    let prepare = || {
        crate::install::installer::answer(Box::new(answer));
        let mut w = World::default();
        // files\b holds obj files and one sub-folder "sub"; files\b\sub holds one file
        w.listings.insert(
            "files\\b\\*.*".into(),
            vec![
                Entry {
                    name: ".",
                    attrs: DIR,
                },
                Entry {
                    name: "..",
                    attrs: DIR,
                },
                Entry {
                    name: "obj0570.ro3",
                    attrs: 0,
                },
                Entry {
                    name: "sub",
                    attrs: DIR,
                },
            ],
        );
        w.listings.insert(
            "files\\b\\sub\\*.*".into(),
            vec![Entry {
                name: "deep.ro3",
                attrs: 0,
            }],
        );
        set_world(w);
        p(DIRS, dptr);
        p(DIR_CURSOR, dptr);
        p(DIR_SCAN, dptr);
        p(DIR_COUNT, 0);
        p(FILES, fptr);
        p(FILE_CURSOR, fptr);
        p(FILE_COUNT, 0);
        crate::install::installer::fill(dptr, 0x2000, 0xAA);
        crate::install::installer::fill(fptr, 0x2000, 0xAA);
    };
    let mut src = b"files\\b\0".to_vec();
    crate::install::installer::both(
        &mut t,
        "files\\b tree",
        &copy,
        Regs {
            eax: src.as_mut_ptr() as u32,
            ..Regs::SENTINEL
        },
        &regions,
        0,
        &[],
        &prepare,
        &|r, part, _| list_folder_tree(r, part),
    );
    let _ = &mut src;
    crate::install::installer::quiet();
    drop(saved);
    drop((dirs, files));
    t.finish()
}
