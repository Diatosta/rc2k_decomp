//! `gfx::build_glyphs` (0x412085): makes a glyph table from numbered sheets of pictures.
//!
//! In:  eax = the sheets' path with `00` or `!!` where the number goes, ebx = the list
//!      (`MemoryList`) the table's memory is taken from
//! Out: every register preserved
//! The sprite and font builder ("Extracting:" and "Appending:" in the game's log). For
//! number 0, 1, .. while such a file exists: the sheet is read (`gfx/read_sheet.rs`: a PCX
//! of one plane stays palette numbers, one of three becomes components), its backdrop is
//! what its four corners agree on and its transparent colour magenta, every glyph on it
//! is found and coded (`gfx/glyph_builder.rs`), and the sheet's table is appended to the
//! table at `dword_5195A4` (its size in `dword_5195A8`), whose memory comes from the list
//! in ebx piece by piece. The builder's flags (`dword_519118`): 1 = glyphs are run-length
//! coded, 0x10 = the path is one file's and not numbered, 0x20 = every sheet has a second
//! one with its alpha; bits 2 and 4 are set here for a sheet of components. The callers
//! (`sub_47C4A1` for the menus' sprites, `sub_415563` for the fonts, `sub_41B178`) set the
//! flags, take the work memory `g_lexiconMemory` and point `dword_5195A4` at where the list
//! stands.
//! Errors: a sheet that does not read is fatal error 0x80E4F010; no room for a glyph, a
//! sheet's table or the table's growth 0x80E48001, 0x80E48011, 0x80E48012; a table to
//! append to that is nowhere 0x80E48077.
//! With the alpha flag every sheet has two files: the one with the lower number is read
//! first and kept in the picture buffer's second half, and the low four bits of its
//! pixels become the top four of the other's (both must be palette sheets: fatal errors
//! 0x80E4F412 and 0x80E4F422; with the one-file flag 0x80E4F411). If that first file is
//! not there, the alpha is made from each glyph's own shape (`Sheet::soften`).
//! The Rust version keeps its glyphs in memory of its own and only counts them off the work memory, where
//! it puts the sheet's table as the original does. It leaves alone: the list of glyph
//! records (`unk_5195AC`), the five work variables of the row scan (`dword_5195B8` ..
//! `dword_5195C8`), and the up to three bytes after a first table that the original's copy
//! by dwords drags along.
//! Shadow mode builds twice (Rust, everything put back, the original) and compares the
//! whole table and the variables. Self-test: with the game running (the original logs),
//! a font sheet and the menus' sprite sheets built by both into memory of the test's.

use crate::errors::{BadInput, FatalError, Refused};
use crate::files::file_exists::file_exists;
use crate::files::numbered_path::numbered_path_in_game;
use crate::framework::{
    RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva, shadow,
};
use crate::gfx::components_to_pixel::components_to_pixel;
use crate::gfx::fill_row_table::fill_row_table;
use crate::gfx::glyph_builder::{
    DIRECT_COLOUR, Glyph, ONE_FILE, RUN_LENGTHS, Sheet, WIDE_HEAD, WITH_ALPHA, append, backdrop_of,
    direct_pixel_size, table_of, transparent_of,
};
use crate::gfx::load_picture::LoaderTest;
use crate::gfx::pcx::{
    DECODED_HEIGHT_RVA, DECODED_WIDTH_RVA, FILE_HEIGHT_RVA, FILE_WIDTH_RVA, PALETTE_RVA,
    PIXEL_BITS_RVA, PIXELS_RVA, PLANES_RVA, STRIDE_RVA,
};
use crate::gfx::picture::{FLAGS_RVA, HEIGHT_RVA, ROW_BYTES_RVA, WIDTH_RVA, picture};
use crate::gfx::pixel_format::{PixelFormat, SURFACE_FORMAT_RVA};
use crate::gfx::read_sheet::read_sheet;
use crate::logfile::log_hex8::log_hex8;
use crate::logfile::log_int::log_int;
use crate::logfile::log_int_fmt::log_int_fmt;
use crate::logfile::log_write::log_write;
use crate::logfile::write_line::LOG_LEVEL_RVA;
use crate::memory::MemoryList;
use crate::memory::allocate::allocate;
use crate::memory::free_list::free_list;
use crate::memory::take_from_list::{rewind_list, take_from_list};
use crate::service::harness::{Region, at, capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new("gfx::build_glyphs", rva(0x412085), thunk, &ORIGINAL)
    .self_test(self_test)
    .self_test_unpaused();

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

pub const BUILDER_FLAGS_RVA: usize = rva(0x519118); // dword_519118
const TEMPLATE_RVA: usize = rva(0x51911C); // template_
const NUMBER_RVA: usize = rva(0x519120); // number
/// dword_519148: the pixel layout a sheet of components is coded in.
pub const FORMAT_RVA: usize = rva(0x519148);
const BACKDROP_RVA: usize = rva(0x51914C); // dword_51914C
const TRANSPARENT_RVA: usize = rva(0x519150); // dword_519150
const SHEET_TABLE_RVA: usize = rva(0x519598); // dword_519598: the sheet's table, and
const SHEET_TABLE_SIZE_RVA: usize = rva(0x51959C); // dword_51959C: its size
const TABLE_LIST_RVA: usize = rva(0x5195A0); // dword_5195A0: the list given in ebx
/// dword_5195A4, dword_5195A8: the table being built and its size so far.
pub const TABLE_RVA: usize = rva(0x5195A4);
pub const TABLE_SIZE_RVA: usize = rva(0x5195A8);
const GLYPH_COUNT_RVA: usize = rva(0x5195B0); // dword_5195B0: the sheet's glyphs, and
const GLYPH_BYTES_RVA: usize = rva(0x5195B4); // dword_5195B4: their records' bytes
const PIXEL_SIZE_RVA: usize = rva(0x5195CC); // dword_5195CC
/// g_lexiconMemory (stru_519558): the builder's work memory, which its callers take.
pub const WORK_MEMORY_RVA: usize = rva(0x519558);
const RECORD_HEAD: u32 = 0x1C;
/// dword_5195D0: 1 = the sheet read last was an alpha sheet, kept for the next one.
const ALPHA_LOADED_RVA: usize = rva(0x5195D0);
/// Half the picture buffer: where a sheet lies, and what an alpha sheet is kept behind.
const HALF_PICTURE_DWORDS: usize = 0x3A980;

// ---- Rust version ----

fn fatal(code: u32, message: &'static str) -> Refused {
    FatalError { code, message }.into()
}

unsafe fn text(text: &[u8], newline: bool) {
    debug_assert!(text.ends_with(&[0]));
    unsafe { log_write(text.as_ptr(), newline, 1) };
}

/// `sub_41299B` with eax = 1: the two lines about a sheet.
unsafe fn log_sheet(path: *const u8, image: bool, backdrop: u32, transparent: u32) {
    unsafe {
        text(b"Extracting: PCX\0", false);
        log_write(path, false, 1);
        log_int(game_read(FILE_WIDTH_RVA), false, 1);
        log_int(game_read(FILE_HEIGHT_RVA), false, 1);
        log_int(game_read::<i32>(PLANES_RVA).wrapping_mul(8), true, 1);
        text(b"Extracting:\0", false);
        text(if image { b"IMAGE\0" } else { b"ALPHA\0" }, false);
        log_int(game_read(DECODED_WIDTH_RVA), false, 1);
        log_int(game_read(DECODED_HEIGHT_RVA), false, 1);
        log_int(game_read(PIXEL_BITS_RVA), false, 1);
        text(b"backdrop=\0", false);
        log_hex8(backdrop, false, 1);
        text(b"transparent=\0", false);
        log_hex8(transparent, true, 1);
    }
}

/// `sub_412A86`: the line about a glyph, the `number`th of its sheet.
unsafe fn log_glyph(glyph: &Glyph, number: u32, flags: u32, pixel_size: u32) {
    unsafe {
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
}

/// `sub_412C92` on the table being built: the sheet's table of `size` bytes at `sheet`
/// becomes part of it.
unsafe fn append_to_table(sheet: u32, size: u32) -> Result<(), Refused> {
    unsafe {
        let (table, held): (u32, u32) = (game_read(TABLE_RVA), game_read(TABLE_SIZE_RVA));
        if game_read::<i32>(LOG_LEVEL_RVA) >= 3 {
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
        let more = std::slice::from_raw_parts(sheet as *const u8, size as usize);
        let mut joined = std::slice::from_raw_parts(table as *const u8, held as usize).to_vec();
        append(&mut joined, more)?;
        std::ptr::copy(joined.as_ptr(), table as *mut u8, joined.len());
        game_write(TABLE_SIZE_RVA, joined.len() as u32);
        Ok(())
    }
}

/// `sub_412446`: the sheet with the next number, if it is there and reads, is kept in the
/// picture buffer's second half as the alpha of the one after it. The number moves on
/// either way.
unsafe fn read_alpha_sheet(template: *const u8) -> Result<(), Refused> {
    unsafe {
        game_write(ALPHA_LOADED_RVA, 0u32);
        let flags: u32 = game_read(BUILDER_FLAGS_RVA);
        if flags & ONE_FILE != 0 {
            return Err(fatal(0x80E4_F411, ""));
        }
        let number: u32 = game_read(NUMBER_RVA);
        let path = numbered_path_in_game(template, number)?;
        if file_exists(path)? && read_sheet(path)? {
            if game_read::<u32>(FLAGS_RVA) & 0x100 == 0 {
                return Err(fatal(
                    0x80E4_F412,
                    "Component colour/alpha files must be 8bit",
                ));
            }
            let flags = flags & !(DIRECT_COLOUR | WIDE_HEAD);
            game_write(PIXEL_SIZE_RVA, 1u32);
            game_write(BUILDER_FLAGS_RVA, flags);
            fill_row_table(
                game_read(PIXELS_RVA),
                game_read(STRIDE_RVA),
                game_read(DECODED_HEIGHT_RVA),
            )?;
            let mut convert = |components: u32| components;
            let sheet = Sheet {
                flags,
                pixel_size: 1,
                width: game_read(DECODED_WIDTH_RVA),
                height: game_read(DECODED_HEIGHT_RVA),
                backdrop: 0,
                transparent: 0,
                convert: &mut convert,
                soften: false,
            };
            if sheet.width == 0 {
                return Err(BadInput("a sheet without columns").into());
            }
            let backdrop = backdrop_of(&sheet);
            game_write(BACKDROP_RVA, backdrop);
            let transparent = transparent_of(flags, backdrop, PALETTE_RVA);
            game_write(TRANSPARENT_RVA, transparent);
            if game_read::<i32>(LOG_LEVEL_RVA) >= 3 {
                let path = numbered_path_in_game(template, number)?;
                log_sheet(path, false, backdrop, transparent);
            }
            game_write(ALPHA_LOADED_RVA, 1u32);
            let pixels = picture() as *mut u32;
            if pixels.is_null() {
                return Err(BadInput("no picture buffer").into());
            }
            std::ptr::copy(pixels, pixels.add(HALF_PICTURE_DWORDS), HALF_PICTURE_DWORDS);
        }
        game_write(NUMBER_RVA, number.wrapping_add(1));
        Ok(())
    }
}

pub unsafe fn build_glyphs(template: *const u8, list: *mut MemoryList) -> Result<(), Refused> {
    unsafe {
        if template.is_null() {
            return Err(BadInput("no path for the sheets").into());
        }
        let work = game_addr(WORK_MEMORY_RVA) as *mut MemoryList;
        game_write(TEMPLATE_RVA, template as u32);
        game_write(TABLE_LIST_RVA, list as u32);
        game_write(NUMBER_RVA, 0u32);
        loop {
            if game_read::<u32>(BUILDER_FLAGS_RVA) & WITH_ALPHA != 0 {
                read_alpha_sheet(template)?;
            }
            let mut flags: u32 = game_read(BUILDER_FLAGS_RVA);
            let path = if flags & ONE_FILE != 0 {
                template
            } else {
                let path = numbered_path_in_game(template, game_read(NUMBER_RVA))?;
                if !file_exists(path)? {
                    return Ok(());
                }
                path
            };
            if !read_sheet(path)? {
                return Err(fatal(0x80E4_F010, "Error loading PCX file"));
            }

            let mut pixel_size = 1;
            flags &= !(DIRECT_COLOUR | WIDE_HEAD);
            let mut format = None;
            if game_read::<u32>(FLAGS_RVA) & 0x100 == 0 {
                flags |= DIRECT_COLOUR | WIDE_HEAD;
                let layout = game_read::<u32>(FORMAT_RVA) as *const PixelFormat;
                if layout.is_null() {
                    return Err(BadInput("no pixel layout for a sheet of components").into());
                }
                let layout = layout.read_unaligned();
                pixel_size = direct_pixel_size(&layout);
                format = Some(layout);
            }
            game_write(BUILDER_FLAGS_RVA, flags);
            game_write(PIXEL_SIZE_RVA, pixel_size);
            fill_row_table(
                game_read(PIXELS_RVA),
                game_read(STRIDE_RVA),
                game_read(DECODED_HEIGHT_RVA),
            )?;

            let mut convert = |components: u32| match &format {
                Some(format) => components_to_pixel(format, components),
                None => components,
            };
            let mut sheet = Sheet {
                flags,
                pixel_size,
                width: game_read(DECODED_WIDTH_RVA),
                height: game_read(DECODED_HEIGHT_RVA),
                backdrop: 0,
                transparent: 0,
                convert: &mut convert,
                soften: flags & WITH_ALPHA != 0 && game_read::<u32>(ALPHA_LOADED_RVA) != 1,
            };
            if sheet.width == 0 {
                return Err(BadInput("a sheet without columns").into());
            }
            sheet.backdrop = backdrop_of(&sheet);
            game_write(BACKDROP_RVA, sheet.backdrop);
            sheet.transparent = transparent_of(flags, sheet.backdrop, PALETTE_RVA);
            game_write(TRANSPARENT_RVA, sheet.transparent);
            let logging = game_read::<i32>(LOG_LEVEL_RVA) >= 3;
            if logging {
                // The original makes the path anew for the line.
                let path = match flags & ONE_FILE {
                    0 => numbered_path_in_game(template, game_read(NUMBER_RVA))?,
                    _ => template,
                };
                log_sheet(path, true, sheet.backdrop, sheet.transparent);
            }
            if flags & WITH_ALPHA != 0 && game_read::<u32>(ALPHA_LOADED_RVA) == 1 {
                // `sub_4124E9`: the kept sheet's low four bits become the top four here.
                if flags & DIRECT_COLOUR != 0 {
                    return Err(fatal(
                        0x80E4_F422,
                        "Component colour/alpha files must be 8bit",
                    ));
                }
                let pixels = picture() as *mut u32;
                if pixels.is_null() {
                    return Err(BadInput("no picture buffer").into());
                }
                for n in 0..HALF_PICTURE_DWORDS {
                    let alpha = pixels.add(HALF_PICTURE_DWORDS + n).read_unaligned();
                    let pixel = pixels.add(n);
                    pixel.write_unaligned(pixel.read_unaligned() | (alpha & 0x0F0F_0F0F) << 4);
                }
            }

            rewind_list(work);
            let glyphs = sheet.glyphs()?;
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
            game_write(GLYPH_COUNT_RVA, glyphs.len() as u32);
            game_write(GLYPH_BYTES_RVA, bytes);

            let table = table_of(&glyphs);
            let size = table.len() as u32;
            let Some(place) = take_from_list(work, size)? else {
                return Err(fatal(0x80E4_8011, ""));
            };
            game_write(SHEET_TABLE_SIZE_RVA, size);
            game_write(SHEET_TABLE_RVA, place);
            std::ptr::copy(table.as_ptr(), place as *mut u8, table.len());
            if take_from_list(list, size)?.is_none() {
                return Err(fatal(0x80E4_8012, ""));
            }
            append_to_table(place, size)?;

            if flags & ONE_FILE != 0 {
                return Ok(());
            }
            game_write(NUMBER_RVA, game_read::<u32>(NUMBER_RVA).wrapping_add(1));
        }
    }
}

// ---- hook ----

/// The builder's own variables.
fn variables() -> Vec<Region> {
    [
        BUILDER_FLAGS_RVA,
        TEMPLATE_RVA,
        NUMBER_RVA,
        BACKDROP_RVA,
        TRANSPARENT_RVA,
        SHEET_TABLE_RVA,
        SHEET_TABLE_SIZE_RVA,
        TABLE_LIST_RVA,
        TABLE_RVA,
        TABLE_SIZE_RVA,
        GLYPH_COUNT_RVA,
        GLYPH_BYTES_RVA,
        PIXEL_SIZE_RVA,
        ALPHA_LOADED_RVA,
    ]
    .map(at)
    .to_vec()
}

/// What a build changes that the other version must find as it was: the variables, the two
/// lists and the table as far as it is built.
struct Before {
    regions: Vec<Region>,
    saved: Vec<u8>,
}

impl Before {
    /// `with_table`: the table so far is a caller's, just begun or being built, and is
    /// kept too. A self-test builds in memory of its own, and what the variables name
    /// then may be a table whose memory has been given back (a stage's font).
    unsafe fn capture(list: *mut MemoryList, with_table: bool) -> Self {
        let mut regions = variables();
        regions.extend([
            (game_addr(rva(0x5195AC)) as u32, 4),
            (game_addr(rva(0x5195B8)) as u32, 0x14),
            (game_addr(WORK_MEMORY_RVA) as u32, 16),
        ]);
        if !list.is_null() {
            regions.push((list as u32, 16));
        }
        let (table, held): (u32, u32) =
            unsafe { (game_read(TABLE_RVA), game_read(TABLE_SIZE_RVA)) };
        if with_table && table != 0 && held != 0 && held < 0x1000_0000 {
            regions.push((table, held as usize));
        }
        Self {
            saved: capture(&regions),
            regions,
        }
    }

    fn restore(&self) {
        restore(&self.regions, &self.saved);
    }
}

/// What a build leaves, for comparing: the variables, the list the table grows on, and the
/// table.
unsafe fn left(list: *mut MemoryList) -> Vec<u8> {
    let mut regions = variables();
    if !list.is_null() {
        regions.push((list as u32, 16));
    }
    let (table, held): (u32, u32) = unsafe { (game_read(TABLE_RVA), game_read(TABLE_SIZE_RVA)) };
    if table != 0 && held != 0 && held < 0x1000_0000 {
        regions.push((table, held as usize));
    }
    capture(&regions)
}

fn difference(theirs: &[u8], mine: &[u8]) -> String {
    let first = theirs.iter().zip(mine).position(|(a, b)| a != b);
    format!(
        "original left {} bytes, rust {}; first different byte {first:?} (the variables are the first {})",
        theirs.len(),
        mine.len(),
        variables().len() * 4
    )
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    let (template, list) = (r.eax as *const u8, r.ebx as *mut MemoryList);
    let before = unsafe { Before::capture(list, true) };
    let mine = unsafe { build_glyphs(template, list) };
    if !shadow() {
        if let Err(e) = mine {
            before.restore();
            unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) };
        }
        return;
    }
    let mine = mine.map(|()| unsafe { left(list) });
    before.restore();
    match mine {
        Ok(mine) => {
            let mut o = *r;
            unsafe { call_original(&mut o) };
            let theirs = unsafe { left(list) };
            if theirs != mine || !o.preserved_except(r, 0) {
                HOOK.mismatch(|| {
                    format!(
                        "template={:?}: {}",
                        unsafe { std::ffi::CStr::from_ptr(template as *const _) },
                        difference(&theirs, &mine)
                    )
                });
            }
        }
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

// ---- self-test: the game's own sheets, built by both into memory of the test's ----

const ROOM: usize = 0x30_0000;
/// (the builder's flags, the path): a font's sheet coded, uncoded and as one file; the
/// menus' sprites, two sheets of components.
const SOFT_FILE: &str = "var\\rc2k_hook_soft_01.pcx";
const CASES: [(u32, &[u8]); 6] = [
    // A font with its alpha sheet; and a sheet whose alpha sheet (number 00) is not
    // there, so that the alpha is made from the glyphs.
    (RUN_LENGTHS | WITH_ALPHA, b"files\\d\\fnt\\fnt22_00.pcx\0"),
    (RUN_LENGTHS | WITH_ALPHA, b"var\\rc2k_hook_soft_00.pcx\0"),
    (RUN_LENGTHS, b"files\\d\\fnt\\fnt13_00.pcx\0"),
    (0, b"files\\d\\fnt\\fnt13_00.pcx\0"),
    (RUN_LENGTHS | ONE_FILE, b"files\\d\\fnt\\fnt21_00.pcx\0"),
    (RUN_LENGTHS, b"files\\g\\graphics\\sprites\\mbuts00.pcx\0"),
];

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let work = game_addr(WORK_MEMORY_RVA) as *mut MemoryList;
    if unsafe { work.read_unaligned() } != MemoryList::default() {
        return t.skip("the builder's work memory is in use");
    }
    // Everything a picture's reading writes is put back at the end.
    if std::fs::copy("files\\d\\fnt\\fnt13_00.pcx", SOFT_FILE).is_err() {
        return t.skip("cannot copy a font sheet into the game's var folder");
    }
    let pictures = match LoaderTest::begin() {
        Ok(test) => test,
        Err(why) => {
            let _ = std::fs::remove_file(SOFT_FILE);
            return t.skip(why);
        }
    };
    let before = unsafe { Before::capture(std::ptr::null_mut(), false) };
    let format_before: u32 = unsafe { game_read(FORMAT_RVA) };
    // As the builder's callers set it up (`sub_412014`); the work memory as large as the
    // largest sheet could need.
    let ready = unsafe {
        game_write(WIDTH_RVA, 800u32);
        game_write(HEIGHT_RVA, 600u32);
        game_write(ROW_BYTES_RVA, 0xFFFF_FFFDu32);
        game_write(FORMAT_RVA, game_addr(SURFACE_FORMAT_RVA) as u32);
        allocate(work, 0x80_0000)
    };
    if ready.is_ok() {
        let mut room = vec![0u8; ROOM];
        let mut list = MemoryList::default();
        for (flags, template) in CASES {
            let mut run = |rust: bool| unsafe {
                room.fill(0xAA);
                list = MemoryList {
                    head: 0,
                    total: ROOM as u32,
                    cursor: room.as_mut_ptr() as u32,
                    remaining: ROOM as u32,
                };
                game_write(BUILDER_FLAGS_RVA, flags);
                game_write(TABLE_RVA, list.cursor);
                game_write(TABLE_SIZE_RVA, 0u32);
                let regs = Regs {
                    eax: template.as_ptr() as u32,
                    ebx: &mut list as *mut MemoryList as u32,
                    ..Regs::SENTINEL
                };
                let done = if rust {
                    build_glyphs(template.as_ptr(), &mut list).is_ok()
                } else {
                    let mut ro = regs;
                    call_original(&mut ro);
                    ro.preserved_except(&regs, 0)
                };
                (done, left(&mut list))
            };
            let (theirs, mine) = (run(false), run(true));
            let built = unsafe { game_read::<u32>(TABLE_SIZE_RVA) } != 0;
            t.check(theirs == mine && theirs.0 && built, || {
                format!(
                    "flags={flags:#x} template={:?}: original done={} rust done={} built={built}; {}",
                    String::from_utf8_lossy(template),
                    theirs.0,
                    mine.0,
                    difference(&theirs.1, &mine.1)
                )
            });
        }
        let _ = unsafe { free_list(work) };
    }
    unsafe { game_write(FORMAT_RVA, format_before) };
    before.restore();
    pictures.end();
    let _ = std::fs::remove_file(SOFT_FILE);
    if ready.is_err() {
        return t.skip("no memory for the builder's work");
    }
    t.finish()
}
