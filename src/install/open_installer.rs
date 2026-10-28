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
//! Caller `install::run_installer`.
//!
//! Conversion phase 3 (2026-10-23): plain Rust with typed arguments; the registers above
//! are the original's.

use crate::errors::Refused;
use crate::gfx::picture_store::PICTURE;
use crate::gfx::screen_state::SCREEN_MODE;
use crate::install::installer::{components_to_pixels, copy_picture, draw_string, fatal, show};
use crate::install::vars::INSTALLER_NAMES;
use crate::install::vars::INSTALLER_PALETTE;
use crate::install::vars::INSTALLER_RUN;

const FONT_94: crate::gfx::typefaces::FontId = crate::gfx::typefaces::fonts::HEAD;
const FONT_1000: crate::gfx::typefaces::FontId = crate::gfx::typefaces::fonts::NOTICE;
const FONT_1002: crate::gfx::typefaces::FontId = crate::gfx::typefaces::fonts::LABEL;

/// `files::read_file` (0x402BB6): `count` bytes of `path` into `buffer`; failing is fatal
/// error `code`.
fn read(path: usize, buffer: u32, count: u32, code: u32) -> Result<(), Refused> {
    let read = crate::files::read_file::read_file(
        crate::pointers::host_at(path) as *const u8,
        crate::pointers::host(buffer),
        count,
        0,
    )?;
    if !read.ok {
        return Err(fatal(code));
    }
    Ok(())
}

pub fn open_installer() -> Result<(), Refused> {
    // The names' tables are Rust's (phase 12), counted on the list as the game's blocks
    // were; the scan and cursors count names and bytes from 0.
    use crate::install::name_tables::{FILES, FOLDERS, NameTable};
    let list = INSTALLER_RUN.buffers.as_ptr() as *mut _;
    FOLDERS.take(list, 0x10100, NameTable::default())?;
    INSTALLER_RUN.dir_scan.set(0);
    INSTALLER_RUN.dir_cursor.set(0);
    FILES.take(list, 0x40100, NameTable::default())?;
    INSTALLER_RUN.file_cursor.set(0);
    INSTALLER_RUN.dir_count.set(0);
    INSTALLER_RUN.file_count.set(0);
    // The progress glyphs: a glyph table of Rust's (`gfx::glyph_store`), counted on the
    // list as the game's block was (its fatal error 0x80AF7022 for none cannot come).
    let mut bytes = crate::memory::owned::counted(list, 0x3000)?;
    let glyphs = crate::gfx::glyph_store::new_table(list)?;
    INSTALLER_RUN.bar_glyphs.set(glyphs);
    let path = crate::pointers::GamePtr::new(INSTALLER_NAMES.installer_data.addr());
    if !crate::files::read_file::read_file_into(path, &mut bytes[..0x3000], 0)?.ok {
        return Err(fatal(0x80AF_7444));
    }
    crate::gfx::glyph_store::write_bytes(glyphs, 0, &bytes, "the progress glyphs")?;
    read(
        INSTALLER_NAMES.install_pal.addr() as usize,
        INSTALLER_PALETTE.palette.addr(),
        0x300,
        0x80AF_7445,
    )?;
    unsafe {
        crate::gfx::palette_to_dwords::palette_to_dwords(
            INSTALLER_PALETTE.palette.addr(),
            crate::text::blend_font::BLENDED_PALETTE.addr(),
        )?;
        let blended = crate::text::blend_font::BLENDED_PALETTE.addr();
        components_to_pixels(blended, blended, crate::session::SCREEN.format_addr());
        // gfx::load_picture: its carry is not read.
        crate::gfx::load_picture::load_picture(INSTALLER_NAMES.backdrop.as_ptr())?;
        crate::gfx::convert_picture::convert_picture_now()?;
    }
    PICTURE.flags.set(SCREEN_MODE.mode_flags.get() & 0xFFF);
    crate::gfx::picture_as_surface::picture_as_surface()?;
    draw_string(0x94, FONT_94.addr(), 0x316, 0x2F)?;
    draw_string(0x1000, FONT_1000.addr(), 0x320, 0x24B)?;
    draw_string(0x1002, FONT_1002.addr(), 0, 0x24B)?;
    copy_picture()?;
    show();
    Ok(())
}
