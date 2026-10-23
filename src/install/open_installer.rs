//! `install::open_installer` (0x414363): the installer's buffers taken and its backdrop
//! drawn. Three blocks are allocated onto the installer memory list (`stru_51BAFC`): the
//! folder names (0x10100 bytes -> `dword_51BB40`/`50`/`54`), the file names (0x40100 ->
//! `dword_51BB44`/`58`), the counts cleared, and the progress glyphs (0x3000 ->
//! `dword_51BB3C`) filled from `files\d\gfx\install.bun`. `install.pal` becomes the blended
//! palette (`gfx::palette_to_dwords`, `gfx::components_to_pixels`); `backdrop.pcx` is loaded,
//! converted and made the surface; three strings are drawn and the frame shown. A failed
//! allocation or read is fatal error 0x80AF7121/7122/7022/7444/7445.
//!
//! In:  nothing
//! Out: every register kept (`pusha` / `popa`)
//! Caller `install::run_installer`. Its calls through `part`. Shadow mode runs the original
//! alone. Self-test: a copy whose allocate/read/graphics calls go to stand-ins (allocation
//! and reads scripted), against the Rust version; the table pointers, counts and the call
//! sequence compared, and each fatal path.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::install::installer::{
    ALLOCATE, BUFFERS, COPY_PICTURE, DIR_COUNT, DIR_CURSOR, DIR_SCAN, DIRS, DRAW_STRING,
    FILE_COUNT, FILE_CURSOR, FILES, Part, READ_FILE, SHOW, a, at, both, fatal, p, quiet, region,
};
use std::sync::Mutex;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("install::open_installer", rva(0x414363), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const BAR_GLYPHS: usize = 0x51BB3C; // dword_51BB3C
const INSTALL_BUN: usize = 0x51BEA1; // "files\d\gfx\install.bun"
const INSTALL_PAL: usize = 0x51BEB9; // "files\d\gfx\install.pal"
const INSTALL_PAL_BUF: usize = 0x4F3874; // unk_4F3874
const BLENDED: usize = 0x4F3B74; // g_blendedPalette
const SURFACE_FORMAT: usize = 0x4F496C; // g_surfaceFormat
const BACKDROP: usize = 0x51BED1; // "files\d\gfx\backdrop.pcx"
const MODE_FLAGS: usize = 0x4F4934; // g_screenModeFlags
const PICTURE_FLAGS: usize = 0x4F49CC; // g_pictureFlags
const PALETTE_TO_DWORDS: usize = 0x405DDE;
const COMPONENTS_TO_PIXELS: usize = 0x405E02;
const LOAD_PICTURE: usize = 0x411863;
const CONVERT_PICTURE: usize = 0x406126;
const PICTURE_AS_SURFACE: usize = 0x405FE7;
const FONT_94: usize = 0x51E944; // stru_51E944
const FONT_1000: usize = 0x51E404; // stru_51E404
const FONT_1002: usize = 0x51E3E4; // stru_51E3E4

fn alloc(bytes: u32, code: u32, r: &mut Regs, part: Part) -> Result<u32, Refused> {
    r.eax = bytes;
    r.ebx = a(BUFFERS);
    part(at(ALLOCATE), r)?;
    if r.carry() {
        return Err(fatal(code));
    }
    Ok(r.eax)
}

fn read(
    path: usize,
    buf: u32,
    count: u32,
    code: u32,
    r: &mut Regs,
    part: Part,
) -> Result<(), Refused> {
    r.eax = a(path);
    r.ebx = buf;
    r.ecx = count;
    r.edx = 0;
    part(at(READ_FILE), r)?;
    if r.carry() {
        return Err(fatal(code));
    }
    Ok(())
}

fn draw(id: u32, font: usize, x: u32, y: u32, r: &mut Regs, part: Part) -> Result<(), Refused> {
    r.eax = id;
    r.ebx = a(font);
    r.ecx = x;
    r.edx = y;
    part(at(DRAW_STRING), r)
}

pub fn open_installer(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    let dirs = alloc(0x10100, 0x80AF_7121, &mut r, part)?;
    p(DIRS, dirs);
    p(DIR_SCAN, dirs);
    p(DIR_CURSOR, dirs);
    let files = alloc(0x40100, 0x80AF_7122, &mut r, part)?;
    p(FILES, files);
    p(FILE_CURSOR, files);
    r.eax = 0;
    p(DIR_COUNT, 0);
    p(FILE_COUNT, 0);
    r.ecx = 0x3000; // mov eax, 3000h; mov ecx, eax
    let glyphs = alloc(0x3000, 0x80AF_7022, &mut r, part)?;
    p(BAR_GLYPHS, glyphs);
    read(INSTALL_BUN, glyphs, 0x3000, 0x80AF_7444, &mut r, part)?;
    read(
        INSTALL_PAL,
        a(INSTALL_PAL_BUF),
        0x300,
        0x80AF_7445,
        &mut r,
        part,
    )?;
    r.esi = a(INSTALL_PAL_BUF);
    r.edi = a(BLENDED);
    part(at(PALETTE_TO_DWORDS), &mut r)?;
    r.ebx = a(SURFACE_FORMAT);
    r.esi = r.edi;
    part(at(COMPONENTS_TO_PIXELS), &mut r)?;
    r.eax = a(BACKDROP);
    part(at(LOAD_PICTURE), &mut r)?;
    part(at(CONVERT_PICTURE), &mut r)?;
    r.eax = crate::install::installer::g(MODE_FLAGS) & 0xFFF;
    p(PICTURE_FLAGS, r.eax);
    part(at(PICTURE_AS_SURFACE), &mut r)?;
    draw(0x94, FONT_94, 0x316, 0x2F, &mut r, part)?;
    draw(0x1000, FONT_1000, 0x320, 0x24B, &mut r, part)?;
    draw(0x1002, FONT_1002, 0, 0x24B, &mut r, part)?;
    part(at(COPY_PICTURE), &mut r)?;
    part(at(SHOW), &mut r)?;
    Ok(Regs { ..*entry })
}

extern "C" fn dispatch(r: &mut Regs) {
    crate::race::viewer::dispatch_parts(&HOOK, r, call_original, &|r| {
        open_installer(r, &mut crate::install::installer::game_part)
    });
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    crate::race::viewer::regs_entry(r, &|r| {
        open_installer(r, &mut crate::install::installer::game_part)
    })
}

static ALLOC_N: Mutex<u32> = Mutex::new(0);

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // pusha; lea ebx, stru_51BAFC
    let copy = match crate::install::installer::copy(
        0x414363,
        0x12D,
        &[0x60, 0x8D, 0x1D, 0xFC, 0xBA, 0x51, 0x00],
        &[
            (0xC, 0x4028A7),
            (0x2B, 0x4028A7),
            (0x53, 0x4028A7),
            (0x6D, 0x402BB6),
            (0x89, 0x402BB6),
            (0xA0, 0x405DDE),
            (0xAD, 0x405E02),
            (0xB8, 0x411863),
            (0xBD, 0x406126),
            (0xD1, 0x405FE7),
            (0xEB, 0x40234A),
            (0x105, 0x40234A),
            (0x11C, 0x40234A),
            (0x121, 0x4060E3),
            (0x126, 0x4057D1),
        ],
        &[
            0x4028A7, 0x402BB6, 0x405DDE, 0x405E02, 0x411863, 0x406126, 0x405FE7, 0x40234A,
            0x4060E3, 0x4057D1,
        ],
        &[],
        &[
            (0x11, 0x80AF_7121),
            (0x30, 0x80AF_7122),
            (0x58, 0x80AF_7022),
            (0x72, 0x80AF_7444),
            (0x8E, 0x80AF_7445),
        ],
        &[],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    let regions = [
        region(DIRS, 4),
        region(DIR_SCAN, 4),
        region(DIR_CURSOR, 4),
        region(FILES, 4),
        region(FILE_CURSOR, 4),
        region(DIR_COUNT, 4),
        region(FILE_COUNT, 4),
        region(BAR_GLYPHS, 4),
        region(PICTURE_FLAGS, 4),
    ];
    let saved = crate::race::viewer::Saved::new(&regions);
    // what `prepare` writes besides the compared regions: put back at the end
    let _kept =
        crate::race::viewer::Saved::new(&[crate::install::installer::region(MODE_FLAGS, 4)]);
    for fail_at in [6u32, 0, 1, 2, 3, 4] {
        // fail_at: which allocate(0..2)/read(3..4) carries; 6 = none
        let prepare = || {
            *ALLOC_N.lock().unwrap() = 0;
            crate::install::installer::answer(Box::new(move |place, r| {
                if place == a(ALLOCATE) {
                    let mut n = ALLOC_N.lock().unwrap();
                    let idx = *n;
                    *n += 1;
                    if idx == fail_at {
                        r.set_carry(true);
                    } else {
                        r.eax = 0x1000_0000 + idx * 0x10000;
                        r.set_carry(false);
                    }
                } else if place == a(READ_FILE) {
                    // reads are calls 3 and 4
                    let which = if r.ecx == 0x3000 { 3 } else { 4 };
                    r.set_carry(which == fail_at);
                    r.eax = r.ecx;
                }
            }));
            p(MODE_FLAGS, 0x1234);
        };
        both(
            &mut t,
            &format!("fail_at {fail_at}"),
            &copy,
            Regs::SENTINEL,
            &regions,
            0,
            &[],
            &prepare,
            &|r, part, _| open_installer(r, part),
        );
    }
    quiet();
    drop(saved);
    t.finish()
}
