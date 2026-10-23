//! `install::run_script` (0x41365D): the install script (a byte program at ebx) run to
//! build the folder- and file-name tables. A name is assembled in `byte_95EAF8` as nested
//! folders, with a stack of the segment starts (the real stack in the original). Opcodes:
//! - 1 n: read a name (`install::script_name`), add the whole path as a folder
//!   (`install::list_folder`), append `\`, and push the level;
//! - 2: pop a level (back to the parent; refused with nothing pushed: the original pops
//!   its caller's stack);
//! - 3 n: read a name and add the matching files (`install::list_files`), path unchanged;
//! - 4 n: read a name and add the folder tree (`install::list_folder_tree`);
//! - 8 n: if `g_mediaMode` is below n, skip to the matching 9 (they nest);
//! - 9, 6: nothing;
//! - any other byte (0): the end. An unbalanced stack is fatal error 0x80AF3011.
//!
//! In:  ebx the script
//! Out: every register kept (`pusha` / `popa`)
//! Callers `install::install_files`, `uninstall`, `install_clio`. Its three table calls
//! through `part`; `script_name` is called directly (it only builds the path). Shadow mode
//! runs the original alone. Self-test: a copy whose three table calls go to stand-ins (the
//! path noted), `script_name` kept, against the Rust version; scripts of every opcode, a
//! media-skip and an unbalanced stack (fatal).

use crate::bridge::Raw;
use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::install::installer::{
    LIST_FILES, LIST_FOLDER, LIST_FOLDER_TREE, MEDIA_MODE, Part, a, at, fatal, gb, rb, wb,
};
use rc2k_core::install::script::{self, Op};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("install::run_script", rva(0x41365D), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const NAME_BUF: usize = 0x95EAF8; // byte_95EAF8

pub fn run_script(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    let mut esi = a(NAME_BUF);
    wb(esi, 0);
    let mut stack: Vec<u32> = Vec::new();
    let media = gb(MEDIA_MODE);
    let mut guard = 0usize;
    // Converted: `rc2k_core::install::script` (`Op`, `skipped`, `skip`).
    loop {
        guard += 1;
        if guard > script::LIMIT {
            return Err(BadInput("a script with no end in 0x10000 bytes").into());
        }
        let al = rb(r.ebx);
        r.ebx = r.ebx.wrapping_add(1);
        match Op::of(al) {
            Op::Folder => {
                stack.push(esi);
                r.esi = esi;
                r = crate::install::script_name::script_name(&r)?;
                esi = r.esi;
                r.eax = a(NAME_BUF);
                part(at(LIST_FOLDER), &mut r)?;
                wb(esi, b'\\');
                wb(esi + 1, 0);
                esi += 1;
            }
            Op::Up => {
                let Some(level) = stack.pop() else {
                    return Err(BadInput(
                        "a pop with nothing pushed (the original pops its caller's stack)",
                    )
                    .into());
                };
                esi = level;
                wb(esi, 0);
            }
            Op::Files | Op::Tree => {
                stack.push(esi);
                r.esi = esi;
                r = crate::install::script_name::script_name(&r)?;
                r.eax = a(NAME_BUF);
                part(
                    at(if al == 3 {
                        LIST_FILES
                    } else {
                        LIST_FOLDER_TREE
                    }),
                    &mut r,
                )?;
                esi = stack.pop().unwrap_or(a(NAME_BUF)); // pushed just above
                wb(esi, 0);
            }
            Op::Media => {
                let n = rb(r.ebx);
                r.ebx = r.ebx.wrapping_add(1);
                if !script::skipped(n, media) {
                    continue;
                }
                let words = unsafe { Raw::new(r.ebx as *const u8) };
                let past = script::skip(&words, 0, &mut guard)?;
                r.ebx = r.ebx.wrapping_add(past as u32);
            }
            Op::Nothing => {}
            Op::End => {
                if !stack.is_empty() {
                    return Err(fatal(script::UNBALANCED));
                }
                break;
            }
        }
    }
    Ok(Regs { ..*entry })
}

extern "C" fn dispatch(r: &mut Regs) {
    crate::race::viewer::dispatch_parts(&HOOK, r, call_original, &|r| {
        run_script(r, &mut crate::install::installer::game_part)
    });
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    crate::race::viewer::regs_entry(r, &|r| {
        run_script(r, &mut crate::install::installer::game_part)
    })
}

fn path_of(_place: u32, r: &Regs) -> Vec<u8> {
    crate::install::installer::text_at(r.eax, 0x200)
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // pusha; lea esi, byte_95EAF8
    let copy = match crate::install::installer::copy(
        0x41365D,
        0xAF,
        &[0x60, 0x8D, 0x35, 0xF8, 0xEA, 0x95, 0x00],
        &[
            (0x14, 0x41370C),
            (0x1F, 0x413876),
            (0x3B, 0x41370C),
            (0x46, 0x4138B2),
            (0x56, 0x41370C),
            (0x61, 0x41371C),
        ],
        &[0x413876, 0x4138B2, 0x41371C],
        &[],
        &[(0xA7, 0x80AF_3011)],
        &[],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    crate::install::installer::quiet();
    crate::install::installer::set_text_of(Some(path_of));
    // ebp at the calls is the original's stack pointer (`mov ebp, esp`)
    crate::install::installer::note_no_ebp(true);
    let regions = [crate::install::installer::region(NAME_BUF, 0x200)];
    let saved = crate::race::viewer::Saved::new(&regions);
    // what `prepare` writes besides the compared regions: put back at the end
    let _kept = crate::race::viewer::Saved::new(&[crate::install::installer::region(0x5189A4, 4)]);
    // Scripts: a name ends at a control byte, which is not taken: it is the next opcode.
    // Never a 2 with nothing pushed (the original pops its caller's stack).
    let scripts: [&[u8]; 6] = [
        &[1, b'b', 2, 0],                                  // enter "b", leave
        &[1, b'b', 1, b'c', 3, b'*', b'.', b'r', 2, 2, 0], // nested, files inside
        &[3, b'*', b'.', b'r', b'o', b'3', 0],
        &[4, b'd', 9, 6, 0],
        &[8, 2, 1, b'x', 2, 8, 5, 4, b'y', 9, 9, 0], // media-gated, nested
        &[1, b'a', 0],                               // unbalanced: fatal
    ];
    for media in [0u32, 3] {
        for script in scripts {
            let prepare = || {
                crate::install::installer::answer(Box::new(|_, _| {}));
                crate::install::installer::p(MEDIA_MODE, media);
                crate::install::installer::fill(a(NAME_BUF), 0x200, 0xAA);
            };
            let mut prog = script.to_vec();
            crate::install::installer::both(
                &mut t,
                &format!("media {media}, script {script:?}"),
                &copy,
                Regs {
                    ebx: prog.as_mut_ptr() as u32,
                    ..Regs::SENTINEL
                },
                &regions,
                0,
                &[],
                &prepare,
                &|r, part, _| run_script(r, part),
            );
            let _ = &mut prog;
        }
    }
    crate::install::installer::set_text_of(None);
    crate::install::installer::note_no_ebp(false);
    crate::install::installer::quiet();
    drop(saved);
    t.finish()
}
