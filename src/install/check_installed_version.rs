//! `install::check_installed_version` (0x413B0B): whether the already-installed `ral.exe` is
//! this same build (`dword_51B9C8`). Only when installed (`g_installStatus` 1) with a source
//! (`g_installSource`): the source is normalised into `byte_95E7F8` and made current, then
//! `.\ral.exe` is read in 0xFF00-byte chunks (`g_pngInfo` the buffer, the last 0x100 bytes of
//! each chunk carried to the front of the next so a match can span a boundary). Each chunk is
//! scanned for `$VER: Rally 5`; on a hit the bytes are skipped to the next NUL and compared
//! against the copyright line (`byte_4E001C`, up to its first `.`); a full match sets
//! `dword_51B9C8`. The file is closed and the current directory set to `byte_95E6F8`.
//!
//! In:  nothing
//! Out: every register kept (`pusha` / `popa`)
//! Caller `install::run_installer`. Its directory and file calls through `part`; the scan
//! here. Shadow mode runs the original alone (it reads a file). Self-test: a copy whose
//! directory/file calls go to the fake file system, against the Rust version, with a
//! matching ral.exe, a non-matching one, and no installation.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::install::installer::{
    CLOSE, INSTALLED, OPEN, Part, READ, SET_CD, STATUS, a, at, gb, p, rb,
};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "install::check_installed_version",
    rva(0x413B0B),
    thunk,
    &ORIGINAL,
)
.self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const SAME_VERSION: usize = 0x51B9C8; // dword_51B9C8
const SOURCE: usize = 0x51BB8D; // g_installSource
const BASE_A: usize = 0x95E6F8; // byte_95E6F8: the folder to go back to
const BUF: usize = 0x93E8B0; // g_pngInfo
const READ_TARGET: usize = BUF + 0x100;
const TAIL: usize = BUF + 0xFF00;
const SCAN_LEN: u32 = 0xFF00;
const RAL_EXE: usize = 0x51BE3E; // ".\ral.exe"
const COPYRIGHT: usize = 0x4E001C; // byte_4E001C

fn matches_at(ebx: u32) -> bool {
    // "$VER" then ": Ra" (0x6152203A) then "lly " (0x20796C6C) then '5'
    rb(ebx) == b'$'
        && rb(ebx + 1) == b'V'
        && rb(ebx + 2) == b'E'
        && rb(ebx + 3) == b'R'
        && dword(ebx + 4) == 0x6152_203A
        && dword(ebx + 8) == 0x2079_6C6C
        && rb(ebx + 0xC) == b'5'
}

fn dword(at: u32) -> u32 {
    unsafe { (at as *const u32).read_unaligned() }
}

pub fn check_installed_version(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    p(SAME_VERSION, 0);
    if gb(STATUS) != 1 || gb(SOURCE) == 0 {
        return Ok(*entry);
    }
    // normalise the source into byte_95E7F8, cd there
    let written = unsafe {
        crate::install::normalise_path::normalise_path(
            a(SOURCE) as *const u8,
            a(INSTALLED) as *mut u8,
        )
    }?;
    if !written {
        return Ok(*entry);
    }
    r.esi = a(SOURCE);
    r.edx = a(INSTALLED);
    part(at(SET_CD), &mut r)?;
    if r.carry() {
        return Ok(*entry);
    }
    unsafe { crate::system::zero_memory::zero_memory(a(BUF) as *mut u8, 0x10000) }?;
    r.eax = a(RAL_EXE);
    r.ebx = 0;
    part(at(OPEN), &mut r)?;
    if !r.carry() {
        // Register for register: edx the handle; the close gets ebx and ecx where the scan
        // stopped (`loc_413C13`).
        r.edx = r.eax;
        'read: loop {
            // the previous tail carried to the front (`rep movsd`, 0x40 dwords)
            for n in 0..0x100u32 {
                let b = rb(a(TAIL) + n);
                crate::install::installer::wb(a(BUF) + n, b);
            }
            r.esi = a(TAIL) + 0x100;
            r.edi = a(BUF) + 0x100;
            r.eax = r.edx;
            r.ebx = a(READ_TARGET);
            r.ecx = SCAN_LEN;
            part(at(READ), &mut r)?;
            if r.carry() || r.eax == 0 {
                break;
            }
            r.ebx = a(BUF);
            r.ecx = SCAN_LEN;
            loop {
                if rb(r.ebx) == b'$' && matches_at(r.ebx) {
                    // to the next NUL (at most 0x7F bytes)
                    r.ecx = 0x80;
                    loop {
                        r.ecx -= 1;
                        if r.ecx == 0 {
                            break 'read;
                        }
                        let b = rb(r.ebx);
                        r.ebx += 1;
                        if b < 1 {
                            break;
                        }
                    }
                    // the copyright line up to its first '.'
                    r.ecx = 0;
                    loop {
                        let want = rb(a(COPYRIGHT) + r.ecx);
                        r.eax = (r.eax & !0xFF) | want as u32;
                        if want != rb(r.ebx + r.ecx) {
                            break 'read;
                        }
                        r.ecx += 1;
                        if want == b'.' {
                            p(SAME_VERSION, 1);
                            break 'read;
                        }
                    }
                }
                r.ebx += 1;
                r.ecx -= 1;
                if r.ecx == 0 {
                    continue 'read;
                }
            }
        }
        r.eax = r.edx;
        part(at(CLOSE), &mut r)?;
    }
    r.edx = a(BASE_A);
    part(at(SET_CD), &mut r)?;
    Ok(Regs { ..*entry })
}

extern "C" fn dispatch(r: &mut Regs) {
    crate::race::viewer::dispatch_parts(&HOOK, r, call_original, &|r| {
        check_installed_version(r, &mut crate::install::installer::game_part)
    });
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    use crate::install::fakefs::{World, answer, set_world};
    // pusha; mov dword_51B9C8, 0
    let copy = match crate::install::installer::copy(
        0x413B0B,
        0x11C,
        &[
            0x60, 0xC7, 0x05, 0xC8, 0xB9, 0x51, 0x00, 0x00, 0x00, 0x00, 0x00,
        ],
        &[
            (0x31, 0x4139EC),
            (0x3C, 0x403105),
            (0x54, 0x401829),
            (0x66, 0x402DE8),
            (0x93, 0x402E3D),
            (0x10A, 0x402E23),
            (0x115, 0x403105),
        ],
        &[0x403105, 0x402DE8, 0x402E3D, 0x402E23],
        &[],
        &[],
        &[],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    // build a matching ral.exe image: "$VER: Rally 5 x\0" + the copyright line
    let mut matching = b"junk....$VER: Rally 5 1.0\0".to_vec();
    let mut k = 0u32;
    loop {
        let b = rb(a(COPYRIGHT) + k);
        matching.push(b);
        k += 1;
        if b == b'.' || k > 200 {
            break;
        }
    }
    matching.push(0);
    let non_matching = b"no version string here at all".to_vec();
    let regions = [
        crate::install::installer::region(SAME_VERSION, 4),
        crate::install::installer::region(INSTALLED, 0x100),
        crate::install::installer::region(BASE_A, 0x100),
        crate::install::installer::region(BUF, 0x10000),
        crate::install::installer::region(0x4F52B0, 4),
    ];
    let saved = crate::race::viewer::Saved::new(&regions);
    // what `prepare` writes besides the compared regions: put back at the end
    let _kept = crate::race::viewer::Saved::new(&[
        crate::install::installer::region(0x51BB8C, 1),
        crate::install::installer::region(SOURCE, 0xFF),
    ]);
    for (what, status, source, image) in [
        ("matching", 1u8, b'c', Some(matching.clone())),
        ("non-matching", 1, b'c', Some(non_matching.clone())),
        ("not installed", 0, 0, None),
    ] {
        let image = image.clone();
        let prepare = || {
            crate::install::installer::answer(Box::new(answer));
            let mut w = World::default();
            if let Some(bytes) = &image {
                w.reads.insert(".\\ral.exe".into(), bytes.clone());
            }
            set_world(w);
            crate::install::installer::pb(STATUS, status);
            // g_installSource: a short absolute path so normalise is pure
            for (n, &b) in b"c:\\rally\0".iter().enumerate() {
                crate::install::installer::wb(
                    a(SOURCE) + n as u32,
                    if source == 0 { 0 } else { b },
                );
            }
            p(SAME_VERSION, 9);
        };
        crate::install::installer::both(
            &mut t,
            what,
            &copy,
            Regs::SENTINEL,
            &regions,
            0,
            &[],
            &prepare,
            &|r, part, _| check_installed_version(r, part),
        );
    }
    crate::install::installer::quiet();
    drop(saved);
    t.finish()
}
