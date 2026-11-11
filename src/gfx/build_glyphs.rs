//! `gfx::build_glyphs` (0x412085): makes a glyph table from numbered sheets of pictures.
//!
//! The sprite and font builder ("Extracting:" and "Appending:" in the game's log). For
//! number 0, 1, .. while such a file exists: the sheet is read (`gfx/read_sheet.rs`: a PCX
//! of one plane stays palette numbers, one of three becomes components), its backdrop is
//! what its four corners agree on and its transparent colour magenta, every glyph on it
//! is found and coded (`gfx/glyph_builder.rs`), and the sheet's table is appended to the
//! table being built (`SheetVars::table`, its size in `table_size`), whose memory comes from
//! the memory list piece by piece. The builder's flags (`g_glyphBuilderFlags`): 1 = glyphs
//! are run-length coded, 0x10 = the path is one file's and not numbered, 0x20 = every
//! sheet has a second one with its alpha; bits 2 and 4 are set here for a sheet of
//! components. The callers (`gfx::build_menu_sprites` for the menus' sprites,
//! `gfx::build_fonts` for the fonts, `gfx::build_game_fonts`) set the flags, take the work
//! memory `g_lexiconMemory` and point the table at where the list stands.
//! Errors: a sheet that does not read is fatal error 0x80E4F010; no room for a glyph, a
//! sheet's table or the table's growth 0x80E48001, 0x80E48011, 0x80E48012; a table to
//! append to that is nowhere 0x80E48077.
//! With the alpha flag every sheet has two files: the one with the lower number is read
//! first and kept in the picture buffer's second half, and the low four bits of its
//! pixels become the top four of the other's (both must be palette sheets: fatal errors
//! 0x80E4F412 and 0x80E4F422; with the one-file flag 0x80E4F411). If that first file is
//! not there, the alpha is made from each glyph's own shape (`Sheet::soften`).
//! The glyphs are kept in memory of its own and only counts them off the work memory, where
//! it puts the sheet's table as the original does. It leaves alone: the list of glyph
//! records, the five work variables of the row scan, and the up to three bytes after a
//! first table that the original's copy by dwords drags along.

use crate::errors::{BadInput, FatalError, Refused};
use crate::files::file_exists::file_exists;
use crate::files::numbered_path::numbered_path_bytes;
use crate::gfx::components_to_pixel::components_to_pixel;
use crate::gfx::fill_row_table::fill_row_table;
use crate::gfx::glyph_builder::{
    DIRECT_COLOUR, Glyph, ONE_FILE, RUN_LENGTHS, RowTable, Sheet, WIDE_HEAD, WITH_ALPHA, append,
    backdrop_of, direct_pixel_size, table_of, transparent_of,
};
use crate::gfx::glyph_store;
use crate::gfx::picture::{read_bytes, with_buffer, with_picture, write_bytes};
use crate::gfx::read_sheet::read_sheet;
use crate::logfile::log_hex8::log_hex8;
use crate::logfile::log_int::log_int;
use crate::logfile::log_int_fmt::log_int_fmt;
use crate::logfile::write_line::level;
use crate::memory::take_from_list::{rewind_list, take_from_list};
use crate::session::PlainDword;

// ---- game data ----

/// IDA's address of the sheet builder's variables.
pub const START: usize = 0x519598;
/// Their length.
pub const LEN: usize = 0x5195D4 - START;

/// The sheet and table being built.
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct SheetVars {
    /// `dword_519598`: the sheet's table, and
    pub sheet_table: PlainDword,
    /// `dword_51959C`: its size.
    pub sheet_table_size: PlainDword,
    /// `dword_5195A0`: the memory list the caller gave. Not used: nothing reads it, so the list
    /// is not kept here.
    pub table_list: PlainDword,
    /// `g_builtTable` (`dword_5195A4`): the table being built.
    pub table: PlainDword,
    /// `g_builtTableSize` (`dword_5195A8`): its size so far.
    pub table_size: PlainDword,
    /// `0x5195AC`: nothing reads or writes it (a gap).
    pub _gap_14: PlainDword,
    /// `g_sheetGlyphCount` (`dword_5195B0`): the sheet's glyphs, and
    pub glyph_count: PlainDword,
    /// `g_sheetGlyphBytes` (`dword_5195B4`): their records' bytes.
    pub glyph_bytes: PlainDword,
    /// `dword_5195B8` .. `dword_5195C8`: nothing reads or writes them (a gap).
    pub _gap_20: [PlainDword; 5],
    /// `g_glyphPixelSize` (`dword_5195CC`).
    pub pixel_size: PlainDword,
    /// `g_alphaSheetLoaded` (`dword_5195D0`): 1 = the sheet read last was an alpha sheet,
    /// kept for the next one.
    pub alpha_loaded: PlainDword,
}

impl SheetVars {
    /// The sheet builder's variables; the first values are the image's (all zero).
    pub const fn new() -> Self {
        SheetVars {
            sheet_table: PlainDword::new(0),
            sheet_table_size: PlainDword::new(0),
            table_list: PlainDword::new(0),
            table: PlainDword::new(0),
            table_size: PlainDword::new(0),
            _gap_14: PlainDword::new(0),
            glyph_count: PlainDword::new(0),
            glyph_bytes: PlainDword::new(0),
            _gap_20: [const { PlainDword::new(0) }; 5],
            pixel_size: PlainDword::new(0),
            alpha_loaded: PlainDword::new(0),
        }
    }
}

const RECORD_HEAD: u32 = 0x1C;
/// Half the picture buffer: where a sheet lies, and what an alpha sheet is kept behind.
const HALF_PICTURE_DWORDS: usize = 0x3A980;

fn fatal(code: u32, message: &'static str) -> Refused {
    FatalError { code, message }.into()
}

fn text(text: &[u8], newline: bool) {
    debug_assert!(text.ends_with(&[0]));
    crate::logfile::log_write::log_nul(text, newline, 1);
}

/// The two lines about a sheet.
fn log_sheet(
    path: &[u8],
    image: bool,
    backdrop: u32,
    transparent: u32,
    gfx: &crate::gfx::owner::Gfx,
) {
    let pictures = &gfx.pictures;
    text(b"Extracting: PCX\0", false);
    crate::logfile::log_write::log_nul(path, false, 1);
    log_int(pictures.pcx.file_width.get() as i32, false, 1);
    log_int(pictures.pcx.file_height.get() as i32, false, 1);
    log_int((pictures.pcx.planes.get() as i32).wrapping_mul(8), true, 1);
    text(b"Extracting:\0", false);
    text(if image { b"IMAGE\0" } else { b"ALPHA\0" }, false);
    log_int(pictures.decoder.width.get() as i32, false, 1);
    log_int(pictures.decoder.height.get() as i32, false, 1);
    log_int(pictures.decoder.pixel_bits.get() as i32, false, 1);
    text(b"backdrop=\0", false);
    log_hex8(backdrop, false, 1);
    text(b"transparent=\0", false);
    log_hex8(transparent, true, 1);
}

/// The line about a glyph, the `number`th of its sheet.
fn log_glyph(glyph: &Glyph, number: u32, flags: u32, pixel_size: u32) {
    let padded = |value: u32| log_int_fmt(value as i32, 0x13, false, 1);
    text(b"\0", false);
    padded(number);
    text(b"(\0", false);
    padded(glyph.width);
    text(b"x\0", false);
    padded(glyph.height);
    text(b":\0", false);
    padded(glyph.x);
    padded(glyph.y);
    text(b")\0", false);
    if flags & RUN_LENGTHS != 0 {
        let byte = |at: usize| glyph.data.get(at).copied().unwrap_or(0) as u32;
        let head = if flags & WIDE_HEAD != 0 {
            [4, 6, 8, 10].map(|at| byte(at) | byte(at + 1) << 8)
        } else {
            [0, 1, 2, 3].map(byte)
        };
        head.into_iter().for_each(padded);
        let size = glyph.data.len() as u32;
        log_int_fmt(size as i32, 0x27, false, 1);
        let plain = glyph
            .width
            .wrapping_mul(glyph.height)
            .wrapping_mul(pixel_size);
        log_int(size.wrapping_sub(plain) as i32, false, 1);
    }
    text(b"\0", true);
    if flags & RUN_LENGTHS != 0 {
        for (bit, warning) in [
            &b"WARNING: Skip GT 255 bytes\0"[..],
            b"WARNING: Count GT 255 bytes\0",
            b"WARNING: XO GT 255\0",
            b"WARNING: YO GT 255\0",
            b"WARNING: XS GT 255\0",
            b"WARNING: YS GT 255\0",
        ]
        .into_iter()
        .enumerate()
        {
            if glyph.warnings & 1 << bit != 0 {
                text(warning, true);
            }
        }
    }
}

/// On the table being built: the sheet's table of `size` bytes at `sheet`
/// becomes part of it.
fn append_to_table(vars: &SheetVars, sheet: u32, more: &[u8]) -> Result<(), Refused> {
    let size = more.len() as u32;
    let (table, held): (u32, u32) = (vars.table.get(), vars.table_size.get());
    if level() as i32 >= 3 {
        text(b"Appending:\0", false);
        log_hex8(table, false, 1);
        log_int(held as i32, false, 1);
        text(b"+\0", false);
        log_hex8(sheet, false, 1);
        log_int(size as i32, true, 1);
    }
    if sheet == 0 || size == 0 {
        return Ok(());
    }
    if table == 0 {
        return Err(fatal(0x80E4_8077, ""));
    }
    let mut joined = glyph_store::read_bytes(table, 0, held as usize, "the glyph table")?;
    append(&mut joined, more)?;
    glyph_store::write_bytes(table, 0, &joined, "the glyph table")?;
    vars.table_size.set(joined.len() as u32);
    Ok(())
}

/// `gfx::read_alpha_sheet`: the sheet with the next number, if it is there and reads, is kept in
/// the picture buffer's second half as the alpha of the one after it. The number moves on
/// either way.
/// The path's text, up to its NUL.
fn read_alpha_sheet(
    vars: &SheetVars,
    template: &[u8],
    gfx: &crate::gfx::owner::Gfx,
) -> Result<(), Refused> {
    let pictures = &gfx.pictures;
    vars.alpha_loaded.set(0u32);
    let flags: u32 = pictures.decoder.builder_flags.get();
    if flags & ONE_FILE != 0 {
        return Err(fatal(0x80E4_F411, ""));
    }
    let number: u32 = pictures.decoder.number.get();
    let path = numbered_path_bytes(template, number)?;
    if file_exists(&path)? && read_sheet(&path, gfx)? {
        if pictures.picture.flags.get() & 0x100 == 0 {
            return Err(fatal(
                0x80E4_F412,
                "Component colour/alpha files must be 8bit",
            ));
        }
        let flags = flags & !(DIRECT_COLOUR | WIDE_HEAD);
        vars.pixel_size.set(1u32);
        pictures.decoder.builder_flags.set(flags);
        fill_row_table(
            0,
            pictures.decoder.stride.get(),
            pictures.decoder.height.get(),
            gfx,
        )?;
        let mut convert = |components: u32| components;
        let backdrop = with_buffer(pictures.decoder.pixels.get(), |pixels| {
            let sheet = Sheet {
                flags,
                pixel_size: 1,
                width: pictures.decoder.width.get(),
                height: pictures.decoder.height.get(),
                backdrop: 0,
                transparent: 0,
                convert: &mut convert,
                pixels: &mut RowTable::new(pixels),
                soften: false,
            };
            if sheet.width == 0 {
                return Err(BadInput("a sheet without columns"));
            }
            Ok(backdrop_of(&sheet))
        })??;
        pictures.decoder.backdrop.set(backdrop);
        let transparent = transparent_of(flags, backdrop, &pictures.decoder.palette);
        pictures.decoder.transparent.set(transparent);
        if level() as i32 >= 3 {
            let path = numbered_path_bytes(template, number)?;
            log_sheet(&path, false, backdrop, transparent, gfx);
        }
        vars.alpha_loaded.set(1u32);
        let half = 4 * HALF_PICTURE_DWORDS;
        with_picture(pictures, |b| {
            (b.len() >= 2 * half).then(|| b.copy_within(..half, half))
        })?
        .ok_or(BadInput("a picture buffer shorter than two halves"))?;
    }
    pictures.decoder.number.set(number.wrapping_add(1));
    Ok(())
}

/// `template`: the sheets' path (its NUL, if any, ends it).
pub fn build_glyphs(
    vars: &SheetVars,
    template: &[u8],
    list: &dyn crate::pointers::Store,
    gfx: &crate::gfx::owner::Gfx,
    strings: &crate::strings::owner::Strings,
) -> Result<(), Refused> {
    let pictures = &gfx.pictures;
    let template = &template[..template
        .iter()
        .position(|&b| b == 0)
        .unwrap_or(template.len())];
    let work = &strings.lexicon.memory.memory;
    pictures.decoder.number.set(0u32);
    loop {
        if pictures.decoder.builder_flags.get() & WITH_ALPHA != 0 {
            read_alpha_sheet(vars, template, gfx)?;
        }
        let mut flags: u32 = pictures.decoder.builder_flags.get();
        let path = if flags & ONE_FILE != 0 {
            template.to_vec()
        } else {
            let path = numbered_path_bytes(template, pictures.decoder.number.get())?;
            if !file_exists(&path)? {
                return Ok(());
            }
            path
        };
        if !read_sheet(&path, gfx)? {
            return Err(fatal(0x80E4_F010, "Error loading PCX file"));
        }

        let mut pixel_size = 1;
        flags &= !(DIRECT_COLOUR | WIDE_HEAD);
        let mut format = None;
        if pictures.picture.flags.get() & 0x100 == 0 {
            flags |= DIRECT_COLOUR | WIDE_HEAD;
            let layout = pictures.decoder.format.get();
            if layout == 0 {
                return Err(BadInput("no pixel layout for a sheet of components").into());
            }
            let layout = crate::gfx::pixel_format::layout(layout)?;
            pixel_size = direct_pixel_size(&layout);
            format = Some(layout);
        }
        pictures.decoder.builder_flags.set(flags);
        vars.pixel_size.set(pixel_size);
        fill_row_table(
            0,
            pictures.decoder.stride.get(),
            pictures.decoder.height.get(),
            gfx,
        )?;

        let mut convert = |components: u32| match &format {
            Some(format) => components_to_pixel(format, components),
            None => components,
        };
        let (width, height) = (pictures.decoder.width.get(), pictures.decoder.height.get());
        if width == 0 {
            return Err(BadInput("a sheet without columns").into());
        }
        let soften = flags & WITH_ALPHA != 0 && vars.alpha_loaded.get() != 1;
        // The sheet, its pixels the decoded buffer's bytes, lent for one step at a time.
        let backdrop = with_buffer(pictures.decoder.pixels.get(), |pixels| {
            backdrop_of(&Sheet {
                flags,
                pixel_size,
                width,
                height,
                backdrop: 0,
                transparent: 0,
                convert: &mut convert,
                pixels: &mut RowTable::new(pixels),
                soften,
            })
        })?;
        pictures.decoder.backdrop.set(backdrop);
        let transparent = transparent_of(flags, backdrop, &pictures.decoder.palette);
        pictures.decoder.transparent.set(transparent);
        let logging = level() as i32 >= 3;
        if logging {
            // The original makes the path anew for the line.
            let path = match flags & ONE_FILE {
                0 => numbered_path_bytes(template, pictures.decoder.number.get())?,
                _ => template.to_vec(),
            };
            log_sheet(&path, true, backdrop, transparent, gfx);
        }
        if flags & WITH_ALPHA != 0 && vars.alpha_loaded.get() == 1 {
            // The kept sheet's low four bits become the top four here.
            if flags & DIRECT_COLOUR != 0 {
                return Err(fatal(
                    0x80E4_F422,
                    "Component colour/alpha files must be 8bit",
                ));
            }
            let half = 4 * HALF_PICTURE_DWORDS;
            let buffer = pictures.picture.buffer.get();
            let alphas = read_bytes(buffer, half, half)?;
            let mut kept = read_bytes(buffer, 0, half)?;
            for (pixel, alpha) in kept
                .as_chunks_mut::<4>()
                .0
                .iter_mut()
                .zip(alphas.as_chunks::<4>().0)
            {
                let alpha = u32::from_le_bytes(*alpha);
                *pixel = (u32::from_le_bytes(*pixel) | (alpha & 0x0F0F_0F0F) << 4).to_le_bytes();
            }
            write_bytes(buffer, 0, &kept)?;
        }

        rewind_list(work)?;
        let glyphs = with_buffer(pictures.decoder.pixels.get(), |pixels| {
            Sheet {
                flags,
                pixel_size,
                width,
                height,
                backdrop,
                transparent,
                convert: &mut convert,
                pixels: &mut RowTable::new(pixels),
                soften,
            }
            .glyphs()
        })??;
        let mut bytes = 0u32;
        for (number, glyph) in glyphs.iter().enumerate() {
            let record = glyph.data.len() as u32;
            if take_from_list(work, record.wrapping_add(RECORD_HEAD))?.is_none() {
                return Err(fatal(0x80E4_8001, ""));
            }
            bytes = bytes.wrapping_add(record);
            if logging {
                log_glyph(glyph, number as u32 + 1, flags, pixel_size);
            }
        }
        vars.glyph_count.set(glyphs.len() as u32);
        vars.glyph_bytes.set(bytes);

        let table = table_of(&glyphs);
        let size = table.len() as u32;
        let Some(place) = take_from_list(work, size)? else {
            return Err(fatal(0x80E4_8011, ""));
        };
        vars.sheet_table_size.set(size);
        // The sheet's table is counted off the work memory where the original put it
        // (`place` is that list's cursor, logged as the original logs its address), and
        // appended from Rust's bytes (`gfx::glyph_store`).
        vars.sheet_table.set(place);
        if take_from_list(list, size)?.is_none() {
            return Err(fatal(0x80E4_8012, ""));
        }
        append_to_table(vars, place, &table)?;

        if flags & ONE_FILE != 0 {
            return Ok(());
        }
        pictures
            .decoder
            .number
            .set(pictures.decoder.number.get().wrapping_add(1));
    }
}

#[cfg(test)]
mod block_tests {
    use super::*;
    use core::mem::offset_of;

    #[test]
    fn first_values_are_the_games() {
        crate::gfx::first_values_are_the_games(
            &crate::gfx::owner::GFX.unchecked().sheet,
            START,
            START + LEN,
        );
    }

    #[test]
    fn moved_knows_the_block() {
        assert_eq!(offset_of!(SheetVars, table), 0x5195A4 - START);
        assert_eq!(offset_of!(SheetVars, glyph_count), 0x5195B0 - START);
        assert_eq!(offset_of!(SheetVars, pixel_size), 0x5195CC - START);
        assert_eq!(
            crate::moved::place(START),
            crate::gfx::owner::GFX.unchecked().sheet.sheet_table.addr() as usize
        );
        assert_eq!(
            crate::moved::place(0x5195D0),
            crate::gfx::owner::GFX.unchecked().sheet.alpha_loaded.addr() as usize
        );
    }
}
