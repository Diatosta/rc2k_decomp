//! `install::make_folders` (0x413A97): the folders of a path made, segment by segment. A
//! drive path (`X:`) first changes to the drive's root; a UNC path (`\\`) skips the server
//! and share. Then for each `\`-separated segment the path is cut there and changed into
//! (`files::set_current_directory`); where that fails the folder is made
//! (`files::make_directory`) and entered. The path's bytes are cut with a NUL and put back
//! as the walk goes. A failed change or make ends it.
//!
//! In:  edx the path (normalised: ending in `\\`)
//! Out: every register kept (`pusha` / `popa`); carry set when a folder could not be gone
//!      into or made (the failing call's flags), else zero set and carry clear
//! Refuses (as met) a path that does not end in `\\` (the original reads past its end).
//! Callers `install::install_files`, `install::install_clio`. Its calls through `part`
//! (`files::set_current_directory`, `files::make_directory`, both edx the path). Ungated.
//! Shadow mode runs the original alone (folders are made). Self-test: a copy whose two
//! calls go to stand-ins (their success scripted), against the Rust version, on paths of
//! the test's; the path buffer is compared (it is put back).

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::install::installer::{CF, MAKE_DIR, Part, SET_CD, at, both, quiet, rb, rw, wb};

use std::sync::Mutex;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("install::make_folders", rva(0x413A97), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL,
    ungated
);

const LIMIT: u32 = 0x400;

/// A path that does not end in `\\`: the original looks for the next one past its end.
const PAST_END: crate::errors::BadInput =
    crate::errors::BadInput("a path that does not end in \\ (the original reads past its end)");

/// The Rust version, register for register as the original: the registers its `popa`
/// leaves (the entry's) with the flags of its end: carry set when a folder could not be
/// gone into or made (the failing call's flags), else the end's `cmp byte ptr [esi+1], 0`
/// (zero, carry clear).
pub fn make_folders(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    let failed = |r: &Regs| -> Result<Regs, Refused> {
        Ok(Regs {
            eflags: r.eflags,
            ..*entry
        })
    };
    r.esi = r.edx;
    r.eax = (r.eax & !0xFFFF) | rw(r.esi) as u32;
    if (r.eax >> 8) as u8 == b':' {
        r.esi += 2;
        // xor cl, cl; xchg cl, [esi+1]
        r.ecx = (r.ecx & !0xFF) | rb(r.esi + 1) as u32;
        wb(r.esi + 1, 0);
        part(at(SET_CD), &mut r)?;
        wb(r.esi + 1, r.ecx as u8);
        if r.carry() {
            return failed(&r);
        }
    } else if r.eax as u16 == 0x5C5C {
        r.esi += 1;
        loop {
            r.esi += 1;
            match rb(r.esi) {
                b'\\' => break,
                0 => return Err(PAST_END.into()),
                _ => {}
            }
        }
        loop {
            r.esi += 1;
            match rb(r.esi) {
                b'\\' => break,
                0 => return Err(PAST_END.into()),
                _ => {}
            }
        }
        wb(r.esi, 0);
        part(at(SET_CD), &mut r)?;
        wb(r.esi, b'\\');
        if r.carry() {
            return failed(&r);
        }
    }
    // loc_413AD6
    let mut guard = 0;
    while rb(r.esi + 1) != 0 {
        guard += 1;
        if guard > LIMIT {
            return Err(crate::errors::BadInput("a path with no end in 0x400 bytes").into());
        }
        loop {
            r.esi += 1;
            match rb(r.esi) {
                b'\\' => break,
                0 => return Err(PAST_END.into()),
                _ => {}
            }
        }
        wb(r.esi, 0);
        part(at(SET_CD), &mut r)?;
        wb(r.esi, b'\\');
        if !r.carry() {
            continue;
        }
        wb(r.esi, 0);
        part(at(MAKE_DIR), &mut r)?;
        wb(r.esi, b'\\');
        if r.carry() {
            return failed(&r);
        }
        wb(r.esi, 0);
        part(at(SET_CD), &mut r)?;
        wb(r.esi, b'\\');
        if r.carry() {
            return failed(&r);
        }
    }
    let mut out = *entry;
    out.set_carry(false);
    out.set_zero(true);
    Ok(out)
}

extern "C" fn dispatch(r: &mut Regs) {
    crate::race::viewer::dispatch_parts(&HOOK, r, call_original, &|r| {
        make_folders(r, &mut crate::install::installer::game_part)
    });
}

// ---- self-test ----

static STEP: Mutex<(usize, Vec<bool>)> = Mutex::new((0, Vec::new()));

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // pusha; mov esi, edx; mov ax, [esi]
    let copy = match crate::install::installer::copy(
        0x413A97,
        0x74,
        &[0x60, 0x8B, 0xF2, 0x66, 0x8B, 0x06],
        &[
            (0x13, 0x403105),
            (0x35, 0x403105),
            (0x4E, 0x403105),
            (0x5B, 0x403113),
            (0x68, 0x403105),
        ],
        &[0x403105, 0x403113],
        &[],
        &[],
        &[],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    quiet();
    let mut buf = vec![0u8; 0x80];
    let ptr = buf.as_mut_ptr() as u32;
    let write_path = |path: &[u8]| {
        for (k, &b) in path.iter().enumerate() {
            wb(ptr + k as u32, b);
        }
        wb(ptr + path.len() as u32, 0);
    };
    // Normalised paths, as every caller gives (`install::normalise_path` ends them with
    // `\\`): without one the original looks for the next `\\` past the path's end.
    for path in [
        &b"c:\\games\\rally\\data\\"[..],
        b"c:\\one\\",
        b"\\\\server\\share\\a\\b\\",
        b"d:folder\\sub\\",
        b"c:\\",
    ] {
        for results in [
            vec![false, false, false, false, false, false],
            vec![false, true, false, true, false],
            vec![true],
            vec![false, true, true],
        ] {
            let regions = [(ptr, 0x80usize)];
            let prepare = || {
                *STEP.lock().unwrap() = (0, results.clone());
                crate::install::installer::answer(Box::new(|place, r| {
                    let set_cd = place == crate::install::installer::a(0x403105);
                    let make = place == crate::install::installer::a(0x403113);
                    if set_cd || make {
                        let mut s = STEP.lock().unwrap();
                        let n = s.0;
                        s.0 += 1;
                        r.set_carry(s.1.get(n).copied().unwrap_or(false));
                    }
                }));
                write_path(path);
            };
            both(
                &mut t,
                &format!(
                    "path {}, results {results:?}",
                    crate::strings::utils::bytes_str(path)
                ),
                &copy,
                Regs {
                    edx: ptr,
                    ..Regs::SENTINEL
                },
                &regions,
                CF,
                &[],
                &prepare,
                &|r, part, _| make_folders(r, part),
            );
        }
    }
    drop(buf);
    t.finish()
}
