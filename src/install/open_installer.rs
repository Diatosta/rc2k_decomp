//! `install::open_installer` (0x414363): the installer's buffers taken and its backdrop
//! drawn. Three blocks are allocated onto the installer memory list (`buffers`): the
//! folder names (0x10100 bytes -> `dirs`/`50`/`54`), the file names (0x40100 ->
//! `files`/`58`), the counts cleared, and the progress glyphs (0x3000 ->
//! `bar_glyphs`) filled from `files\d\gfx\install.bun`. `install.pal` becomes the blended
//! palette (`gfx::palette_to_dwords`, `gfx::components_to_pixels`); `backdrop.pcx` is loaded,
//! converted and made the surface; three strings are drawn and the frame shown. A failed
//! allocation or read is fatal error 0x80AF7121/7122/7022/7444/7445.
//!
//! Caller `install::run_installer`.

use crate::errors::Refused;
use crate::install::installer::{components_to_pixels, copy_picture, draw_string, fatal, show};

const FONT_94: crate::gfx::typefaces::FontId = crate::gfx::typefaces::fonts::HEAD;
const FONT_1000: crate::gfx::typefaces::FontId = crate::gfx::typefaces::fonts::NOTICE;
const FONT_1002: crate::gfx::typefaces::FontId = crate::gfx::typefaces::fonts::LABEL;

/// `files::read_file` (0x402BB6): `count` bytes of `path` into `buffer`; failing is fatal
/// error `code`.
/// `count` bytes of the file `path` into `buffer`; the fatal error `code` when it cannot be
/// read.
fn read<const N: usize>(
    path: &[u8],
    buffer: &crate::session::PlainBytes<N>,
    count: usize,
    code: u32,
) -> Result<(), Refused> {
    let mut bytes = vec![0u8; count];
    let read = crate::files::read_file::read_file_into(path, &mut bytes, 0)?;
    // What was read goes in (the rest of the buffer is left as it was).
    let got = read.bytes_read.unwrap_or(0) as usize;
    buffer.write_at(0, &bytes[..got.min(count)], "a file's bytes")?;
    if !read.ok {
        return Err(fatal(code));
    }
    Ok(())
}

pub fn open_installer(
    install: &'static crate::install::owner::Install,
    gfx: &crate::gfx::owner::Gfx,
    system: &crate::system::owner::System,
) -> Result<(), Refused> {
    // The names' tables are Rust's, counted on the list as the game's blocks
    // were; the scan and cursors count names and bytes from 0.
    use crate::install::name_tables::{FILES, FOLDERS, NameTable};
    let list = &install.run.buffers;
    FOLDERS.take(list, 0x10100, NameTable::default())?;
    install.run.dir_scan.set(0);
    install.run.dir_cursor.set(0);
    FILES.take(list, 0x40100, NameTable::default())?;
    install.run.file_cursor.set(0);
    install.run.dir_count.set(0);
    install.run.file_count.set(0);
    // The progress glyphs: a glyph table of Rust's (`gfx::glyph_store`), counted on the
    // list as the game's block was (its fatal error 0x80AF7022 for none cannot come).
    let mut bytes = crate::memory::owned::counted(list, 0x3000)?;
    let glyphs = crate::gfx::glyph_store::new_table(list)?;
    install.run.bar_glyphs.set(glyphs);
    let path = install.names.installer_data.text();
    if !crate::files::read_file::read_file_into(&path[..], &mut bytes[..0x3000], 0)?.ok {
        return Err(fatal(0x80AF_7444));
    }
    crate::gfx::glyph_store::write_bytes(glyphs, 0, &bytes, "the progress glyphs")?;
    read(
        &install.names.install_pal.text(),
        &install.palette.palette,
        0x300,
        0x80AF_7445,
    )?;
    crate::gfx::palette_to_dwords::palette_to_dwords(
        &install.palette.palette,
        0,
        crate::text::blend_font::BLENDED_PALETTE.get(),
    )?;
    components_to_pixels(
        crate::text::blend_font::BLENDED_PALETTE.get(),
        crate::text::blend_font::BLENDED_PALETTE.get(),
        gfx,
    );
    // gfx::load_picture: its carry is not read.
    crate::gfx::load_picture::load_picture_with(&install.names.backdrop, gfx, system)?;
    crate::gfx::convert_picture::convert_picture_now_in(gfx, system)?;
    gfx.pictures
        .picture
        .flags
        .set(gfx.display.mode.mode_flags.get() & 0xFFF);
    crate::gfx::picture_as_surface::picture_as_surface(gfx)?;
    draw_string(0x94, FONT_94, 0x316, 0x2F, gfx)?;
    draw_string(0x1000, FONT_1000, 0x320, 0x24B, gfx)?;
    draw_string(0x1002, FONT_1002, 0, 0x24B, gfx)?;
    copy_picture()?;
    show();
    Ok(())
}
