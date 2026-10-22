//! `gfx::build_fonts` (0x415563): builds the fonts' glyph tables and palettes and hangs
//! them on the font objects.
//!
//! In:  nothing
//! Out: every register preserved
//! Fatal error 0x80A48020 if it has been done (`dword_51DEB0`, the palettes, is not 0).
//! Takes `dword_51B980` * 0x400 * `dword_4F4958` bytes on the list `stru_51B984`
//! (0x80E48023 if there are none), `begin_glyph_tables`, the builder's flags = 1, 0x400
//! bytes off the list for the palettes; then five builds with `gfx::build_glyphs`, a new
//! table for each: `files\d\fnt\pal_00.pcx` (a sheet without glyphs: its palette becomes
//! the 256 pixels at the palettes' place, and is kept as components at `unk_51DEC4`),
//! `fnt13_00.pcx`, `fnt21_00.pcx`, `fnt23_00.pcx`, `fnt27_00.pcx`; `end_glyph_tables`. Then
//! 125 stores: each font object's palette (+0x14: the palettes' place plus the offset of
//! its sixteen colours) and glyph table (+0x18), as listed in `HUNG`. A font that measures
//! (mode 4) gets a palette too, which `text::draw_text` then uses as its line counter.
//! Callers: `sub_47486C` (the game's start, before the main menu), `sub_4154CD`,
//! `sub_4703C2`, `sub_41CDC2`, `sub_412DE0`, `sub_443104`.
//! A refusal from one of the steps leaves the work half done; none can come with the
//! game's own files.
//! Shadow mode runs the original alone (the memory is taken once) and checks every one of
//! the 125 stores and the palette's pixels against what the Rust version would store;
//! the tables themselves are compared in `gfx::build_glyphs`' hook. Self-test: the part
//! with the calls as a redirected copy that ends before the stores and keeps the
//! palettes' place in a test variable, its sixteen calls noted; and the fonts as they
//! are, against `HUNG`.

use crate::errors::{FatalError, Refused};
use crate::framework::{
    RawHook, Regs, SelfTest, call_game, game_addr, game_read, game_write, hook_asm, rva, shadow,
};
use crate::gfx::build_glyphs::{BUILDER_FLAGS_RVA, FORMAT_RVA, TABLE_RVA};
use crate::gfx::components_to_pixel::components_to_pixel;
use crate::gfx::glyph_builder::RUN_LENGTHS;
use crate::gfx::glyph_tables::{
    ALLOCATE, BEGIN_TABLE, BEGIN_TABLES, BUILD_GLYPHS, END_TABLES, Game, Noted, PALETTE_TO_PIXELS,
    PLACES, Parts, REWIND_LIST, TABLES_LIST_RVA, TABLES_UNITS_RVA, TAKE_FROM_LIST, UNIT_FACTOR_RVA,
    stand_in_for,
};
use crate::gfx::pcx::PALETTE_RVA;
use crate::gfx::pixel_format::PixelFormat;
use crate::menu::frame_test;
use crate::redirect::RedirectedCopy;
use crate::service::harness::{Region, at, capture, restore};
use crate::text::font::MODE_MEASURE;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("gfx::build_fonts", rva(0x415563), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

/// dword_51DEB0: 256 pixels, the fonts' palettes; 0 = the fonts are not built.
pub const PALETTES_RVA: usize = rva(0x51DEB0);
const FONT27_RVA: usize = rva(0x51DEB4); // dword_51DEB4 (first the palette sheet's table)
const FONT23_RVA: usize = rva(0x51DEB8); // dword_51DEB8
const FONT21_RVA: usize = rva(0x51DEBC); // dword_51DEBC
const FONT13_RVA: usize = rva(0x51DEC0); // dword_51DEC0
const PALETTE_COPY_RVA: usize = rva(0x51DEC4); // unk_51DEC4: the palette sheet's 256 dwords
const PALETTE_BYTES: usize = 0x400;
const PALETTE_SHEET_RVA: usize = rva(0x5297DE); // "files\d\fnt\pal_00.pcx"
/// The four font sheets and where each one's table is kept.
const FONT_SHEETS: [(usize, usize); 4] = [
    (rva(0x52977A), FONT13_RVA), // fnt13_00.pcx
    (rva(0x529793), FONT21_RVA), // fnt21_00.pcx
    (rva(0x5297AC), FONT23_RVA), // fnt23_00.pcx
    (rva(0x5297C5), FONT27_RVA), // fnt27_00.pcx
];

const P: usize = PALETTES_RVA;
const F13: usize = FONT13_RVA;
const F21: usize = FONT21_RVA;
const F23: usize = FONT23_RVA;
const F27: usize = FONT27_RVA;
/// The stores, in the original's order: (where, the variable stored, plus this much).
#[rustfmt::skip]
const HUNG: [(u32, usize, u32); 125] = [
    (0x51E3B8, P, 0x0),
    (0x51E3BC, F13, 0x0),
    (0x51E3F8, P, 0x300),
    (0x51E3FC, F13, 0x0),
    (0x51E3D8, P, 0x300),
    (0x51E3DC, F13, 0x0),
    (0x51E418, P, 0x300),
    (0x51E41C, F13, 0x0),
    (0x51E438, P, 0x0),
    (0x51E43C, F21, 0x0),
    (0x51E478, P, 0x180),
    (0x51E47C, F21, 0x0),
    (0x51E498, P, 0x180),
    (0x51E49C, F21, 0x0),
    (0x51E458, P, 0x180),
    (0x51E45C, F21, 0x0),
    (0x51E4D8, P, 0x200),
    (0x51E4DC, F21, 0x0),
    (0x51E4F8, P, 0x200),
    (0x51E4FC, F21, 0x0),
    (0x51E4B8, P, 0x200),
    (0x51E4BC, F21, 0x0),
    (0x51E538, P, 0x80),
    (0x51E53C, F21, 0x0),
    (0x51E558, P, 0x80),
    (0x51E55C, F21, 0x0),
    (0x51E518, P, 0x80),
    (0x51E51C, F21, 0x0),
    (0x51E598, P, 0x0),
    (0x51E59C, F21, 0x0),
    (0x51E578, P, 0x0),
    (0x51E57C, F21, 0x0),
    (0x51E5B8, P, 0x0),
    (0x51E5BC, F21, 0x0),
    (0x51E5F8, P, 0x380),
    (0x51E5FC, F21, 0x0),
    (0x51E5D8, P, 0x380),
    (0x51E5DC, F21, 0x0),
    (0x51E618, P, 0x380),
    (0x51E61C, F21, 0x0),
    (0x51E638, P, 0x0),
    (0x51E63C, F23, 0x0),
    (0x51E678, P, 0x80),
    (0x51E67C, F23, 0x0),
    (0x51E658, P, 0x80),
    (0x51E65C, F23, 0x0),
    (0x51E698, P, 0x80),
    (0x51E69C, F23, 0x0),
    (0x51E6D8, P, 0x0),
    (0x51E6DC, F23, 0x0),
    (0x51E6B8, P, 0x0),
    (0x51E6BC, F23, 0x0),
    (0x51E6F8, P, 0x0),
    (0x51E6FC, F23, 0x0),
    (0x51E738, P, 0x380),
    (0x51E73C, F23, 0x0),
    (0x51E718, P, 0x380),
    (0x51E71C, F23, 0x0),
    (0x51E758, P, 0x380),
    (0x51E75C, F23, 0x0),
    (0x51E798, P, 0x340),
    (0x51E79C, F23, 0x0),
    (0x51E778, P, 0x340),
    (0x51E77C, F23, 0x0),
    (0x51E7B8, P, 0x340),
    (0x51E7BC, F23, 0x0),
    (0x51E7F8, P, 0x180),
    (0x51E7FC, F23, 0x0),
    (0x51E7D8, P, 0x180),
    (0x51E7DC, F23, 0x0),
    (0x51E818, P, 0x180),
    (0x51E81C, F23, 0x0),
    (0x51E838, P, 0x0),
    (0x51E83C, F27, 0x0),
    (0x51E878, P, 0x80),
    (0x51E87C, F27, 0x0),
    (0x51E858, P, 0x80),
    (0x51E85C, F27, 0x0),
    (0x51E898, P, 0x80),
    (0x51E89C, F27, 0x0),
    (0x51E8D8, P, 0x0),
    (0x51E8DC, F27, 0x0),
    (0x51E8B8, P, 0x0),
    (0x51E8BC, F27, 0x0),
    (0x51E8F8, P, 0x0),
    (0x51E8FC, F27, 0x0),
    (0x51E938, P, 0x100),
    (0x51E93C, F27, 0x0),
    (0x51E918, P, 0x100),
    (0x51E91C, F27, 0x0),
    (0x51E958, P, 0x100),
    (0x51E95C, F27, 0x0),
    (0x51E998, P, 0x380),
    (0x51E99C, F27, 0x0),
    (0x51E978, P, 0x380),
    (0x51E97C, F27, 0x0),
    (0x51E9B8, P, 0x380),
    (0x51E9BC, F27, 0x0),
    (0x51EA58, P, 0x340),
    (0x51EA5C, F27, 0x0),
    (0x51EA38, P, 0x340),
    (0x51EA3C, F27, 0x0),
    (0x51EA78, P, 0x340),
    (0x51EA7C, F27, 0x0),
    (0x51E9F8, P, 0x180),
    (0x51E9FC, F27, 0x0),
    (0x51E9D8, P, 0x180),
    (0x51E9DC, F27, 0x0),
    (0x51EA18, P, 0x180),
    (0x51EA1C, F27, 0x0),
    (0x51EA98, P, 0xC0),
    (0x51EA9C, F27, 0x0),
    (0x51EABC, F27, 0x0),
    (0x51EAF8, P, 0x300),
    (0x51EAFC, F13, 0x0),
    (0x51EAD8, P, 0x300),
    (0x51EADC, F13, 0x0),
    (0x51EB18, P, 0x300),
    (0x51EB1C, F13, 0x0),
    (0x51EB58, P, 0x0),
    (0x51EB5C, F21, 0x0),
    (0x51EB38, P, 0x0),
    (0x51EB3C, F21, 0x0),
    (0x51EB78, P, 0x0),
    (0x51EB7C, F21, 0x0),
];

// ---- Rust version ----

/// The part with the calls. `entry` are the registers at entry; `palettes` is where the
/// palettes' place is kept.
unsafe fn tables(entry: &Regs, parts: &mut dyn Parts, palettes: *mut u32) -> Result<(), Refused> {
    unsafe {
        let mut g = *entry;
        g.eax = game_read::<u32>(TABLES_UNITS_RVA)
            .wrapping_mul(0x400)
            .wrapping_mul(game_read(UNIT_FACTOR_RVA));
        g.ebx = game_addr(TABLES_LIST_RVA) as u32;
        parts.call(ALLOCATE, &mut g)?;
        if g.carry() {
            return Err(FatalError {
                code: 0x80E4_8023,
                message: "",
            }
            .into());
        }
        parts.call(REWIND_LIST, &mut g)?;
        parts.call(BEGIN_TABLES, &mut g)?;
        game_write(BUILDER_FLAGS_RVA, RUN_LENGTHS);
        g.eax = 0x400;
        parts.call(TAKE_FROM_LIST, &mut g)?;
        palettes.write_unaligned(g.eax);
        parts.call(BEGIN_TABLE, &mut g)?;
        g.eax = game_addr(PALETTE_SHEET_RVA) as u32;
        parts.call(BUILD_GLYPHS, &mut g)?;
        g.eax = game_read(TABLE_RVA);
        game_write(FONT27_RVA, g.eax);
        g.edi = palettes.read_unaligned();
        parts.call(PALETTE_TO_PIXELS, &mut g)?;
        std::ptr::copy(
            game_addr(PALETTE_RVA) as *const u8,
            game_addr(PALETTE_COPY_RVA) as *mut u8,
            PALETTE_BYTES,
        );
        g.esi = (game_addr(PALETTE_RVA) + PALETTE_BYTES) as u32;
        g.edi = (game_addr(PALETTE_COPY_RVA) + PALETTE_BYTES) as u32;
        g.ecx = 0;
        for (sheet, kept) in FONT_SHEETS {
            parts.call(BEGIN_TABLE, &mut g)?;
            g.eax = game_addr(sheet) as u32;
            parts.call(BUILD_GLYPHS, &mut g)?;
            g.eax = game_read(TABLE_RVA);
            game_write(kept, g.eax);
        }
        parts.call(END_TABLES, &mut g)?;
        Ok(())
    }
}

/// What a store of `HUNG` stores.
unsafe fn hung_value(variable: usize, plus: u32) -> u32 {
    unsafe { game_read::<u32>(variable).wrapping_add(plus) }
}

pub unsafe fn build_fonts(entry: &Regs, parts: &mut dyn Parts) -> Result<(), Refused> {
    unsafe {
        if game_read::<u32>(PALETTES_RVA) != 0 {
            return Err(FatalError {
                code: 0x80A4_8020,
                message: "",
            }
            .into());
        }
        tables(entry, parts, game_addr(PALETTES_RVA) as *mut u32)?;
        for (place, variable, plus) in HUNG {
            game_write(rva(place as usize), hung_value(variable, plus));
        }
        Ok(())
    }
}

// ---- hook ----

/// What of the fonts as they are is not as the Rust version leaves them, or None.
/// `in_use`: the fonts have been drawn with since, so the palette of a font that measures
/// is not looked at (`text::draw_text` counts lines in it).
unsafe fn fonts_wrong(in_use: bool) -> Option<String> {
    unsafe {
        for (place, variable, plus) in HUNG {
            if in_use
                && variable == PALETTES_RVA
                && game_read::<u8>(rva(place as usize - 0x14 + 2)) == MODE_MEASURE
            {
                continue;
            }
            let (theirs, mine): (u32, u32) =
                (game_read(rva(place as usize)), hung_value(variable, plus));
            if theirs != mine {
                return Some(format!("{place:#x} holds {theirs:#x}, not {mine:#x}"));
            }
        }
        let palettes = game_read::<u32>(PALETTES_RVA) as *const u32;
        let format = game_read::<u32>(FORMAT_RVA) as *const PixelFormat;
        if palettes.is_null() || format.is_null() {
            return Some("no palettes or no pixel layout".into());
        }
        let format = format.read_unaligned();
        for entry in 0..256 {
            let components: u32 = game_read(PALETTE_COPY_RVA + entry * 4);
            let (theirs, mine) = (
                palettes.add(entry).read_unaligned(),
                components_to_pixel(&format, components),
            );
            if theirs != mine {
                return Some(format!(
                    "palette entry {entry} is {theirs:#x}, not {mine:#x}"
                ));
            }
        }
        None
    }
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if !shadow() {
        let entry = *r;
        if let Err(e) = unsafe { build_fonts(&entry, &mut Game) } {
            unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) };
        }
        return;
    }
    let mut o = *r;
    unsafe { call_original(&mut o) };
    let wrong = unsafe { fonts_wrong(false) };
    if wrong.is_some() || !o.preserved_except(r, 0) {
        HOOK.mismatch(|| format!("the original's fonts: {wrong:?}"));
    }
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let entry = *r;
    unsafe { build_fonts(&entry, &mut Game) }?;
    Ok(())
}

// ---- self-test ----

/// The original up to its stores, and room for the `popa` and `retn` put after it.
const CALLS_LEN: usize = 0xF8;
const ENTRY_BYTES: [u8; 8] = [0x60, 0x83, 0x3D, 0xB0, 0xDE, 0x51, 0x00, 0x00];
const CALLS: [usize; 16] = [
    0x25, 0x30, 0x35, 0x49, 0x53, 0x5E, 0x73, 0x8B, 0x96, 0xA5, 0xB0, 0xBF, 0xCA, 0xD9, 0xE4, 0xF3,
];
/// Where the copy names `dword_51DEB0`.
const PALETTES_OPERANDS: [usize; 3] = [0x03, 0x4F, 0x6F];

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    if unsafe { game_read::<u32>(PALETTES_RVA) } == 0 {
        return t.skip("the fonts are not built");
    }
    // The fonts the game draws with, whichever version built them.
    let wrong = unsafe { fonts_wrong(true) };
    t.check(wrong.is_none(), || {
        format!("the fonts as they are: {wrong:?}")
    });

    let mut palettes = 0u32;
    let copy = unsafe {
        RedirectedCopy::new(
            game_addr(rva(0x415563)),
            CALLS_LEN + 4,
            &ENTRY_BYTES,
            &CALLS,
            &stand_in_for,
        )
    };
    let copy = match copy {
        Ok(copy) => copy,
        Err(why) => return t.skip(why),
    };
    let named = game_addr(PALETTES_RVA) as u32;
    for operand in PALETTES_OPERANDS {
        let there = unsafe { ((copy.addr() + operand) as *const u32).read_unaligned() };
        if there != named {
            return t.skip("the original is not what the test was made from");
        }
        unsafe { copy.patch_u32(operand, &mut palettes as *mut u32 as u32) };
    }
    // popa; retn
    unsafe { copy.patch_u32(CALLS_LEN, 0x9090_C361) };

    let variables: Vec<Region> = vec![
        at(BUILDER_FLAGS_RVA),
        at(FONT27_RVA),
        at(FONT23_RVA),
        at(FONT21_RVA),
        at(FONT13_RVA),
        (game_addr(PALETTE_COPY_RVA) as u32, PALETTE_BYTES),
    ];
    let saved = capture(&variables);
    for flags in [0u32, 0x33] {
        let before = Regs::SENTINEL;
        let mut run = |rust: bool| unsafe {
            restore(&variables, &vec![0xAA; saved.len()]);
            game_write(BUILDER_FLAGS_RVA, flags);
            palettes = 0;
            frame_test::begin(u32::MAX, 0);
            let done = if rust {
                tables(&before, &mut Noted(&PLACES), &mut palettes).is_ok()
            } else {
                let mut ro = before;
                call_game(&mut ro, copy.addr());
                ro.preserved_except(&before, 0)
            };
            (done, frame_test::notes(), palettes, capture(&variables))
        };
        let (theirs, mine) = (run(false), run(true));
        t.check(theirs == mine && theirs.0 && theirs.1.len() == 16, || {
            let call = theirs.1.iter().zip(&mine.1).position(|(a, b)| a != b);
            format!(
                "flags={flags:#x}: original done={} calls={} palettes={:#x}; rust done={} calls={} palettes={:#x}; first different call {call:?} ({:x?} / {:x?}); same variables={}",
                theirs.0,
                theirs.1.len(),
                theirs.2,
                mine.0,
                mine.1.len(),
                mine.2,
                call.map(|n| theirs.1[n]),
                call.map(|n| mine.1[n]),
                theirs.3 == mine.3
            )
        });
    }
    restore(&variables, &saved);
    t.finish()
}
