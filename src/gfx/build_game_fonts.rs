//! `gfx::build_game_fonts` (0x41B178): builds the font with an alpha that a stage's loading
//! asks for, and hangs it on five font objects.
//!
//! In:  nothing
//! Out: every register preserved
//! ("LOADLEVEL_INIT : loadgamefonts" in the game's log; one caller, `sub_44426F`.)
//! Takes `dword_5BE894` * 0x400 * `dword_4F4958` bytes on the list `stru_5BE898` (fatal
//! error 0x80E48061 if there are none), `begin_glyph_tables`, the builder's flags = 0x21
//! (run-length coded, with alpha sheets), the pixel layout at `off_5ABACC` + 8, twice
//! 0x400 bytes off the list for two palettes, a new table, `gfx::build_glyphs` on
//! `files\d\fnt\fnt22_00.pcx` (0x80E48062 if that leaves no table), `alpha_palette`, the
//! palette's pixels to the first palette, `end_glyph_tables`. The fonts at 0x51E2E4 and
//! 0x51E304 get the first palette and the table. Then the sheet's palette is made
//! brighter where it lies (first byte + 0x40, second and third + 0x20, each stopping at
//! 0xFF) and its pixels are the second palette, which the fonts at 0x51E324, 0x51E344 and
//! 0x51E364 get with the same table.
//! Shadow mode runs the original alone (the memory is taken once) and checks the stores
//! and the second palette; the table is compared in `gfx::build_glyphs`' hook. Self-test:
//! a redirected copy whose ten calls note themselves and which writes the second
//! palette's pixels where the test says.

use crate::errors::{BadInput, FatalError, Refused};
use crate::framework::{
    RawHook, Regs, SelfTest, call_game, game_addr, game_read, game_write, hook_asm, rva, shadow,
};
use crate::gfx::build_glyphs::{BUILDER_FLAGS_RVA, FORMAT_RVA, TABLE_RVA, TABLE_SIZE_RVA};
use crate::gfx::components_to_pixel::components_to_pixel;
use crate::gfx::glyph_builder::{RUN_LENGTHS, WITH_ALPHA};
use crate::gfx::glyph_tables::{
    ALLOCATE, ALPHA_PALETTE, BEGIN_TABLE, BEGIN_TABLES, BUILD_GLYPHS, END_TABLES, Game, Noted,
    PALETTE_TO_PIXELS, PLACES, Parts, REWIND_LIST, TAKE_FROM_LIST, UNIT_FACTOR_RVA, stand_in_for,
};
use crate::gfx::pcx::PALETTE_RVA;
use crate::gfx::pixel_format::PixelFormat;
use crate::menu::frame_test;
use crate::redirect::RedirectedCopy;
use crate::service::harness::{Region, at, capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("gfx::build_game_fonts", rva(0x41B178), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

const LIST_RVA: usize = rva(0x5BE898); // stru_5BE898
const UNITS_RVA: usize = rva(0x5BE894); // dword_5BE894
/// off_5ABACC: something whose pixel layout is 8 bytes in (the textures', a reading).
const LAYOUT_OWNER_RVA: usize = rva(0x5ABACC);
const SHEETS_RVA: usize = rva(0x5BE8D8); // "files\d\fnt\fnt22_00.pcx"
const PALETTE_A_RVA: usize = rva(0x51E2D8); // dword_51E2D8: the first palette's place
const GLYPHS_RVA: usize = rva(0x51E2DC); // dword_51E2DC: the table
/// The five font objects, 0x20 bytes each: palette at +0x14, glyph table at +0x18.
const FONTS_RVA: usize = rva(0x51E2E4);
const FONT_PALETTE: usize = 0x14;
const FONT_GLYPHS: usize = 0x18;
/// The palette of the third font: where the second palette's place is kept.
const PALETTE_B_RVA: usize = FONTS_RVA + 2 * 0x20 + FONT_PALETTE;

// ---- Rust version ----

fn fatal(code: u32) -> Refused {
    FatalError { code, message: "" }.into()
}

/// `entry` are the registers at entry; `pixels_b` is where the place of the second
/// palette's pixels is read from when they are made.
pub unsafe fn build_game_fonts(
    entry: &Regs,
    parts: &mut dyn Parts,
    pixels_b: *const u32,
) -> Result<(), Refused> {
    unsafe {
        let mut g = *entry;
        g.eax = game_read::<u32>(UNITS_RVA)
            .wrapping_mul(0x400)
            .wrapping_mul(game_read(UNIT_FACTOR_RVA));
        g.ebx = game_addr(LIST_RVA) as u32;
        parts.call(ALLOCATE, &mut g)?;
        if g.carry() {
            return Err(fatal(0x80E4_8061));
        }
        parts.call(REWIND_LIST, &mut g)?;
        parts.call(BEGIN_TABLES, &mut g)?;
        game_write(BUILDER_FLAGS_RVA, RUN_LENGTHS | WITH_ALPHA);
        let format = game_read::<u32>(LAYOUT_OWNER_RVA).wrapping_add(8);
        game_write(FORMAT_RVA, format);
        g.eax = 0x400;
        parts.call(TAKE_FROM_LIST, &mut g)?;
        game_write(PALETTE_A_RVA, g.eax);
        g.eax = 0x400;
        parts.call(TAKE_FROM_LIST, &mut g)?;
        game_write(PALETTE_B_RVA, g.eax);
        parts.call(BEGIN_TABLE, &mut g)?;
        g.eax = game_addr(SHEETS_RVA) as u32;
        parts.call(BUILD_GLYPHS, &mut g)?;
        g.eax = game_read(TABLE_RVA);
        game_write(GLYPHS_RVA, g.eax);
        if game_read::<u32>(TABLE_SIZE_RVA) == 0 {
            return Err(fatal(0x80E4_8062));
        }
        parts.call(ALPHA_PALETTE, &mut g)?;
        g.edi = game_read(PALETTE_A_RVA);
        parts.call(PALETTE_TO_PIXELS, &mut g)?;
        parts.call(END_TABLES, &mut g)?;

        let (glyphs, palette_a): (u32, u32) = (game_read(GLYPHS_RVA), game_read(PALETTE_A_RVA));
        for font in 0..2 {
            game_write(FONTS_RVA + font * 0x20 + FONT_GLYPHS, glyphs);
            game_write(FONTS_RVA + font * 0x20 + FONT_PALETTE, palette_a);
        }

        // The sheet's palette made brighter where it lies, and its pixels.
        let to = pixels_b.read_unaligned() as *mut u32;
        let layout = game_read::<u32>(FORMAT_RVA) as *const PixelFormat;
        if to.is_null() || layout.is_null() {
            return Err(BadInput("no place for the second palette, or no pixel layout").into());
        }
        let layout = layout.read_unaligned();
        for n in 0..256usize {
            let place = game_addr(PALETTE_RVA + n * 4) as *mut u32;
            // Converted: `rc2k_core::gfx::pixels::brighter`.
            let brighter = rc2k_core::gfx::pixels::brighter(place.read_unaligned());
            place.write_unaligned(brighter);
            to.add(n)
                .write_unaligned(components_to_pixel(&layout, brighter));
        }

        let palette_b: u32 = game_read(PALETTE_B_RVA);
        game_write(FONTS_RVA + 2 * 0x20 + FONT_GLYPHS, glyphs);
        for font in 3..5 {
            game_write(FONTS_RVA + font * 0x20 + FONT_GLYPHS, glyphs);
            game_write(FONTS_RVA + font * 0x20 + FONT_PALETTE, palette_b);
        }
        Ok(())
    }
}

// ---- hook ----

/// What of the fonts is not as the Rust version leaves them, or None.
unsafe fn fonts_wrong() -> Option<String> {
    unsafe {
        let (glyphs, a, b): (u32, u32, u32) = (
            game_read(GLYPHS_RVA),
            game_read(PALETTE_A_RVA),
            game_read(PALETTE_B_RVA),
        );
        for (font, palette) in [a, a, b, b, b].into_iter().enumerate() {
            let (theirs, table): (u32, u32) = (
                game_read(FONTS_RVA + font * 0x20 + FONT_PALETTE),
                game_read(FONTS_RVA + font * 0x20 + FONT_GLYPHS),
            );
            if theirs != palette || table != glyphs {
                return Some(format!(
                    "font {font} has palette {theirs:#x} and table {table:#x}, not {palette:#x} and {glyphs:#x}"
                ));
            }
        }
        let layout = game_read::<u32>(FORMAT_RVA) as *const PixelFormat;
        if b == 0 || layout.is_null() || glyphs != game_read::<u32>(TABLE_RVA) {
            return Some("no second palette, no pixel layout, or another table".into());
        }
        let layout = layout.read_unaligned();
        for n in 0..256usize {
            let (theirs, mine) = (
                (b as *const u32).add(n).read_unaligned(),
                components_to_pixel(&layout, game_read(PALETTE_RVA + n * 4)),
            );
            if theirs != mine {
                return Some(format!(
                    "entry {n} of the second palette is {theirs:#x}, not {mine:#x}"
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
        let pixels_b = game_addr(PALETTE_B_RVA) as *const u32;
        if let Err(e) = unsafe { build_game_fonts(&entry, &mut Game, pixels_b) } {
            unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) };
        }
        return;
    }
    let mut o = *r;
    unsafe { call_original(&mut o) };
    let wrong = unsafe { fonts_wrong() };
    if wrong.is_some() || !o.preserved_except(r, 0) {
        HOOK.mismatch(|| format!("the original's fonts: {wrong:?}"));
    }
}

// ---- self-test ----

const LEN: usize = 0x144;
const ENTRY_BYTES: [u8; 8] = [0x60, 0x69, 0x05, 0x94, 0xE8, 0x5B, 0x00, 0x00];
/// The last one is `gfx::components_to_pixel`, which stays what it is.
const CALLS: [usize; 10] = [0x18, 0x23, 0x28, 0x49, 0x58, 0x62, 0x6D, 0x89, 0x94, 0x99];
const CONVERT_CALL: usize = 0xFF;
/// Where the copy names the third font's palette when it loads the place to write the
/// second palette's pixels.
const PIXELS_B_OPERAND: usize = 0xD4;

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    if unsafe { game_read::<u32>(LAYOUT_OWNER_RVA) } == 0 {
        return t.skip("no pixel layout to build with");
    }
    let mut calls = CALLS.to_vec();
    calls.push(CONVERT_CALL);
    let copy = unsafe {
        RedirectedCopy::new(
            game_addr(rva(0x41B178)),
            LEN,
            &ENTRY_BYTES,
            &calls,
            &stand_in_for,
        )
    };
    let copy = match copy {
        Ok(copy) => copy,
        Err(why) => return t.skip(why),
    };
    let mut pixels = [0u32; 256];
    let pixels_at: u32 = pixels.as_mut_ptr() as u32;
    let named = game_addr(PALETTE_B_RVA) as u32;
    if unsafe { ((copy.addr() + PIXELS_B_OPERAND) as *const u32).read_unaligned() } != named {
        return t.skip("the original is not what the test was made from");
    }
    unsafe { copy.patch_u32(PIXELS_B_OPERAND, &pixels_at as *const u32 as u32) };

    let variables: Vec<Region> = vec![
        at(BUILDER_FLAGS_RVA),
        at(FORMAT_RVA),
        at(TABLE_SIZE_RVA),
        (game_addr(PALETTE_A_RVA) as u32, 8 + 5 * 0x20),
        (game_addr(PALETTE_RVA) as u32, 0x400),
    ];
    let saved = capture(&variables);
    for seed in [0u32, 0x9E37_79B9] {
        let before = Regs::SENTINEL;
        let mut run = |rust: bool| unsafe {
            restore(&variables, &vec![0xAA; saved.len()]);
            game_write(TABLE_SIZE_RVA, 1u32);
            // A palette with entries that stop at 0xFF and entries that do not.
            for n in 0..256u32 {
                let entry = (n.wrapping_mul(0x0101_0101) ^ seed).wrapping_mul(2_654_435_761);
                game_write(PALETTE_RVA + n as usize * 4, entry);
            }
            pixels.fill(0xAAAA_AAAA);
            frame_test::begin(u32::MAX, 0);
            let done = if rust {
                build_game_fonts(&before, &mut Noted(&PLACES), &pixels_at).is_ok()
            } else {
                let mut ro = before;
                call_game(&mut ro, copy.addr());
                ro.preserved_except(&before, 0)
            };
            (done, frame_test::notes(), pixels, capture(&variables))
        };
        let (theirs, mine) = (run(false), run(true));
        t.check(theirs == mine && theirs.0 && theirs.1.len() == 10, || {
            let call = theirs.1.iter().zip(&mine.1).position(|(a, b)| a != b);
            format!(
                "seed={seed:#x}: original done={} calls={}; rust done={} calls={}; first different call {call:?} ({:x?} / {:x?}); same pixels={} same variables={}",
                theirs.0,
                theirs.1.len(),
                mine.0,
                mine.1.len(),
                call.map(|n| theirs.1[n]),
                call.map(|n| mine.1[n]),
                theirs.2 == mine.2,
                theirs.3 == mine.3
            )
        });
    }
    restore(&variables, &saved);
    t.finish()
}
