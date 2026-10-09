//! `gfx::blit_marker_pair` (0x46D098): draws glyphs 0x9C and 0x9D of the sprite table at the
//! two points stored in an object.
//!
//! In:  edi = object (first point at +0x44/+0x48, second at +0x4C/+0x50), ebx = palette
//! Out: every register preserved
//! Each point is also stored in the globals `g_markerX`/`g_markerY`, which are left holding
//! the second one. What the object and the two glyphs are is not known yet.

use crate::errors::FatalError;
use crate::framework::{RawHook, Regs, SelfTest, game_read, game_write, hook_asm, rva, shadow};
use crate::gfx::blit_glyph::{
    BlitCall, blit_glyph_on_surface, glyph_record, record_calls, recording_works,
};
use crate::gfx::surface::surface;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("gfx::blit_marker_pair", rva(0x46D098), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

const MARKER_X_RVA: usize = rva(0x755A18); // g_markerX
const MARKER_Y_RVA: usize = rva(0x755A1C); // g_markerY
const SPRITE_TABLE_PTR_RVA: usize = rva(0x755581); // g_spriteTable: pointer to a glyph table

const FIRST_GLYPH: u32 = 0x9C;
const SECOND_GLYPH: u32 = 0x9D;

fn sprite_table() -> u32 {
    unsafe { game_read(SPRITE_TABLE_PTR_RVA) }
}

// ---- Rust version ----

/// The two blitter calls the original makes for `object`.
pub unsafe fn marker_calls(object: *const u8, palette: u32, table: u32) -> [BlitCall; 2] {
    let coord = |at: usize| unsafe { object.add(at).cast::<i32>().read_unaligned() };
    [
        BlitCall {
            glyph: FIRST_GLYPH,
            x: coord(0x44),
            y: coord(0x48),
            palette,
            table,
        },
        BlitCall {
            glyph: SECOND_GLYPH,
            x: coord(0x4C),
            y: coord(0x50),
            palette,
            table,
        },
    ]
}

/// Draws both markers on the game's surface.
pub unsafe fn blit_marker_pair(object: *const u8, palette: u32) -> Result<(), FatalError> {
    for call in unsafe { marker_calls(object, palette, sprite_table()) } {
        unsafe {
            game_write(MARKER_X_RVA, call.x);
            game_write(MARKER_Y_RVA, call.y);
            blit_glyph_on_surface(
                call.glyph,
                call.table as *const u8,
                call.x,
                call.y,
                call.palette as *const u32,
            )?;
        }
    }
    Ok(())
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    let (object, palette) = (r.edi as *const u8, r.ebx);

    if !shadow() {
        // A fatal error in the blitter: the original, run from the start, meets the same one.
        if unsafe { blit_marker_pair(object, palette) }.is_err() {
            unsafe { call_original(r) };
        }
        return;
    }

    let table = sprite_table();
    let mine = unsafe { marker_calls(object, palette, table) };
    let surface = surface();
    if mine
        .iter()
        .any(|c| unsafe { glyph_record(&surface, c.table as *const u8, c.glyph) }.is_err())
    {
        unsafe { call_original(r) };
        return;
    }

    let mut o = *r;
    let calls = record_calls(false, || unsafe { call_original(&mut o) });
    let (x, y): (i32, i32) = unsafe { (game_read(MARKER_X_RVA), game_read(MARKER_Y_RVA)) };
    let ok = o.preserved_except(r, 0)
        && (x, y) == (mine[1].x, mine[1].y)
        && (!recording_works() || calls == mine);
    if !ok {
        HOOK.mismatch(|| {
            format!(
                "object={:#x}: original calls={calls:?} x={x} y={y} rust calls={mine:?}",
                r.edi
            )
        });
    }
}

// ---- self-test: the blitter's hook swallows the calls, so nothing is drawn ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    if !recording_works() {
        return t.skip("the gfx::blit_glyph hook is off, so the original would really draw");
    }
    let saved: (i32, i32) = unsafe { (game_read(MARKER_X_RVA), game_read(MARKER_Y_RVA)) };
    let table = sprite_table();
    let points = [
        [0i32, 0, 0, 0],
        [10, 20, 30, 40],
        [-5, -6, 700, 500],
        [i32::MAX, i32::MIN, -1, 1],
    ];

    for p in points {
        let mut object = [0xAAu8; 0x54];
        for (i, v) in p.iter().enumerate() {
            object[0x44 + i * 4..0x48 + i * 4].copy_from_slice(&v.to_le_bytes());
        }
        let before = Regs {
            edi: object.as_ptr() as u32,
            ..Regs::SENTINEL
        };
        let mut ro = before;
        let calls = record_calls(true, || unsafe { call_original(&mut ro) });
        let (x, y): (i32, i32) = unsafe { (game_read(MARKER_X_RVA), game_read(MARKER_Y_RVA)) };

        let mine = unsafe { marker_calls(object.as_ptr(), before.ebx, table) };
        let preserved = ro.preserved_except(&before, 0);
        t.check(
            calls == mine && (x, y) == (p[2], p[3]) && preserved,
            || {
                format!(
                    "points={p:?}: original calls={calls:?} x={x} y={y} preserved={preserved}; rust calls={mine:?}"
                )
            },
        );
    }

    unsafe {
        game_write(MARKER_X_RVA, saved.0);
        game_write(MARKER_Y_RVA, saved.1);
    }
    t.finish()
}
