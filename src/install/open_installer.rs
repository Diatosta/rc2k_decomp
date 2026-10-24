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
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.
//! Conversion phase 3 (2026-10-23): plain Rust with typed arguments; the registers above
//! are the original's.

use crate::errors::Refused;
use crate::install::installer::{
    BUFFERS, DIR_COUNT, DIR_CURSOR, DIR_SCAN, DIRS, FILE_COUNT, FILE_CURSOR, FILES, a,
    components_to_pixels, copy_picture, draw_string, fatal, p, show,
};

const BAR_GLYPHS: usize = 0x51BB3C; // dword_51BB3C
const INSTALL_BUN: usize = 0x51BEA1; // "files\d\gfx\install.bun"
const INSTALL_PAL: usize = 0x51BEB9; // "files\d\gfx\install.pal"
const INSTALL_PAL_BUF: usize = 0x4F3874; // unk_4F3874
const BACKDROP: usize = 0x51BED1; // "files\d\gfx\backdrop.pcx"
const MODE_FLAGS: usize = 0x4F4934; // g_screenModeFlags
const FONT_94: usize = 0x51E944; // stru_51E944
const FONT_1000: usize = 0x51E404; // stru_51E404
const FONT_1002: usize = 0x51E3E4; // stru_51E3E4

/// `memory::allocate` (0x4028A7) on the installer's list (it never fails with the
/// original's carry: the fatal error `_code` is the original's for that).
fn alloc(bytes: u32, _code: u32) -> Result<u32, Refused> {
    Ok(unsafe { crate::memory::allocate::allocate(a(BUFFERS) as *mut _, bytes) }? as u32)
}

/// `files::read_file` (0x402BB6): `count` bytes of `path` into `buffer`; failing is fatal
/// error `code`.
fn read(path: usize, buffer: u32, count: u32, code: u32) -> Result<(), Refused> {
    let read = unsafe {
        crate::files::read_file::read_file(a(path) as *const u8, buffer as *mut u8, count, 0)
    }?;
    if !read.ok {
        return Err(fatal(code));
    }
    Ok(())
}

pub fn open_installer() -> Result<(), Refused> {
    let dirs = alloc(0x10100, 0x80AF_7121)?;
    p(DIRS, dirs);
    p(DIR_SCAN, dirs);
    p(DIR_CURSOR, dirs);
    let files = alloc(0x40100, 0x80AF_7122)?;
    p(FILES, files);
    p(FILE_CURSOR, files);
    p(DIR_COUNT, 0);
    p(FILE_COUNT, 0);
    let glyphs = alloc(0x3000, 0x80AF_7022)?;
    p(BAR_GLYPHS, glyphs);
    read(INSTALL_BUN, glyphs, 0x3000, 0x80AF_7444)?;
    read(INSTALL_PAL, a(INSTALL_PAL_BUF), 0x300, 0x80AF_7445)?;
    unsafe {
        crate::gfx::palette_to_dwords::palette_to_dwords(
            a(INSTALL_PAL_BUF) as *const u8,
            crate::text::blend_font::BLENDED_PALETTE.addr() as *mut u32,
        )?;
        let blended = crate::text::blend_font::BLENDED_PALETTE.addr() as u32;
        components_to_pixels(
            blended,
            blended,
            crate::session::SCREEN.format_addr() as u32,
        );
        // gfx::load_picture: its carry is not read.
        crate::gfx::load_picture::load_picture(a(BACKDROP) as *mut u8)?;
        crate::gfx::convert_picture::convert_picture_now()?;
    }
    unsafe {
        crate::gfx::block::game_write(
            crate::gfx::picture::FLAGS_RVA,
            crate::install::installer::g(MODE_FLAGS) & 0xFFF,
        )
    };
    unsafe { crate::gfx::picture_as_surface::picture_as_surface() }?;
    draw_string(0x94, a(FONT_94), 0x316, 0x2F)?;
    draw_string(0x1000, a(FONT_1000), 0x320, 0x24B)?;
    draw_string(0x1002, a(FONT_1002), 0, 0x24B)?;
    copy_picture()?;
    show();
    Ok(())
}
