//! `gfx::set_clip` (0x406988): sets the clip rectangle from a rectangle given as a corner
//! and a size.
//!
//! In:  ebx = four dwords: x, y, width, height
//! Out: every register preserved
//! `g_clipLeft` = x, `g_clipTop` = y, `g_clipRight` = x + width - 1, `g_clipBottom` = y +
//! height - 1. Three callers, each with the surface's own rectangle (`g_surface`).
//! Shadow mode runs the original and checks the four variables against the Rust
//! arithmetic. Refuses a null rectangle. The name is a reading.

use crate::errors::BadInput;
use crate::framework::{
    RawHook, Regs, SelfTest, call_game, game_addr, game_read, game_write, hook_asm, rva, shadow,
};
use crate::gfx::surface::{CLIP_BOTTOM_RVA, CLIP_LEFT_RVA, CLIP_RIGHT_RVA, CLIP_TOP_RVA};
use crate::redirect::RedirectedCopy;
use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("gfx::set_clip", rva(0x406988), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- Rust version ----

/// (left, top, right, bottom) for a rectangle of x, y, width, height. Converted:
/// `rc2k_core::gfx::raster::clip_of`.
pub use rc2k_core::gfx::raster::clip_of;

unsafe fn clip_now() -> [u32; 4] {
    unsafe {
        [
            game_read(CLIP_LEFT_RVA),
            game_read(CLIP_TOP_RVA),
            game_read(CLIP_RIGHT_RVA),
            game_read(CLIP_BOTTOM_RVA),
        ]
    }
}

pub unsafe fn set_clip(rectangle: *const [u32; 4]) -> Result<(), BadInput> {
    if rectangle.is_null() {
        return Err(BadInput("no rectangle to clip to"));
    }
    unsafe {
        let [left, top, right, bottom] = clip_of(rectangle.read_unaligned());
        game_write(CLIP_LEFT_RVA, left);
        game_write(CLIP_TOP_RVA, top);
        game_write(CLIP_RIGHT_RVA, right);
        game_write(CLIP_BOTTOM_RVA, bottom);
    }
    Ok(())
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    let rectangle = r.ebx as *const [u32; 4];
    if !shadow() {
        if let Err(e) = unsafe { set_clip(rectangle) } {
            unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) };
        }
        return;
    }
    let mine = (!rectangle.is_null()).then(|| clip_of(unsafe { rectangle.read_unaligned() }));
    let mut o = *r;
    unsafe { call_original(&mut o) };
    let theirs = unsafe { clip_now() };
    if mine.is_some_and(|mine| mine != theirs) || !o.preserved_except(r, 0) {
        HOOK.mismatch(|| format!("original left {theirs:?}, rust {mine:?}"));
    }
}

// ---- self-test: a copy of the original that writes four variables of the test's instead
// of the clip rectangle, which the game's drawing reads all the time ----

static T_CLIP: [AtomicU32; 4] = [const { AtomicU32::new(0) }; 4];

const LEN: usize = 0x2A;
const ENTRY_BYTES: [u8; 8] = [0x51, 0x52, 0x8B, 0x0B, 0x8B, 0x53, 0x04, 0x89];
/// Where the copy names left, top, right, bottom.
const OPERANDS: [usize; 4] = [0x09, 0x0F, 0x1D, 0x23];

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let copy = match unsafe {
        RedirectedCopy::new(game_addr(rva(0x406988)), LEN, &ENTRY_BYTES, &[], &|_| None)
    } {
        Ok(copy) => copy,
        Err(why) => return t.skip(why),
    };
    for (operand, variable) in OPERANDS.into_iter().zip(&T_CLIP) {
        unsafe { copy.patch_u32(operand, variable as *const AtomicU32 as u32) };
    }
    for rectangle in [
        [0u32, 0, 800, 600],
        [10, 20, 30, 40],
        [0, 0, 0, 0],
        [5, 5, 1, 1],
        [0xFFFF_FFF0, 7, 0x20, 0xFFFF_FFFF],
    ] {
        for variable in &T_CLIP {
            variable.store(0xAAAA_AAAA, Ordering::SeqCst);
        }
        let before = Regs {
            ebx: rectangle.as_ptr() as u32,
            ..Regs::SENTINEL
        };
        let mut ro = before;
        unsafe { call_game(&mut ro, copy.addr()) };
        let theirs: Vec<u32> = T_CLIP.iter().map(|v| v.load(Ordering::SeqCst)).collect();
        let mine = clip_of(rectangle);
        let preserved = ro.preserved_except(&before, 0);
        t.check(theirs == mine && preserved, || {
            format!("{rectangle:x?}: original {theirs:x?} preserved={preserved}; rust {mine:x?}")
        });
    }
    t.finish()
}
