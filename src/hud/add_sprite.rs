//! `hud::add_sprite` (0x4B70EE): a sprite from the texture's rectangle at esi (left, top,
//! width, height) put at (ebx, ecx): the quad record's texture places (<< 16), its corners
//! (the width and height made positive), then `hud::add_quad`.
//!
//! In:  esi the rectangle, ebx x, ecx y
//! Out: all kept (`pusha` .. `popa`)
//!
//! Callers: the HUD's drawers (code at 0x4B1E42, 0x4B1E62, ...). The quad through `part`
//! (1), in the game its Rust version. Shadow mode runs the original alone (it draws).
//! Self-test: a copy whose call goes to a stand-in, against the Rust version, rectangles
//! by lot (negative sizes among them).

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_write, hook_asm, rva, shadow};
use crate::hud::add_quad::QUAD_RVA;
use crate::service::harness::{Region, capture, restore};
use rc2k_core::hud::quads;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("hud::add_sprite", rva(0x4B70EE), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// A call by place: 1 `add_quad`.
pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;

/// The quad record's texture places from the rectangle at `s` (shared with
/// `hud::add_scaled_sprite`): the rectangle.
pub unsafe fn put_places(s: u32) -> [u32; 4] {
    unsafe {
        let rect: [u32; 4] =
            std::array::from_fn(|n| ((s + 4 * n as u32) as *const u32).read_unaligned());
        for (n, v) in quads::sprite_places(rect) {
            game_write(QUAD_RVA + 4 * n, v);
        }
        rect
    }
}

/// The quad record's corners (see `quads::sprite_corners`).
pub unsafe fn put_corners(x: u32, y: u32, w: u32, h: u32) {
    for (n, v) in quads::sprite_corners(x, y, w, h) {
        unsafe { game_write(QUAD_RVA + 4 * n, v) };
    }
}

/// The Rust version (see the header): the registers it leaves.
// Converted: `rc2k_core::hud::quads::{sprite_places, sprite_corners}`
pub unsafe fn add_sprite(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    unsafe {
        let [_, _, w, h] = put_places(entry.esi);
        put_corners(entry.ebx, entry.ecx, w, h);
    }
    let mut r = Regs {
        eax: unsafe { crate::framework::game_read(QUAD_RVA + 4 * quads::BOTTOM) },
        ..*entry
    };
    part(1, &mut r)?;
    Ok(*entry)
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    match unsafe { add_sprite(r, &mut crate::hud::add_triangle::game_part_quad) } {
        Ok(left) => *r = Regs { esp: r.esp, ..left },
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

fn self_test() -> (usize, usize) {
    use crate::race::level_init::{note, take_called};
    use crate::race::mud::copy;
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // pusha; mov eax, [esi]; shl eax, 10h
    let entry: &[u8] = &[0x60, 0x8B, 0x06, 0xC1, 0xE0, 0x10];
    let copy = match copy(0x4B70EE, 0x7C, entry, &[0x75], &[(0x4B72D1, 1)]) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let regions: Vec<Region> = vec![(game_addr(QUAD_RVA) as u32, 0x38)];
    let saved = capture(&regions);
    let regs = |r: &Regs| [r.eax, r.ebx, r.ecx, r.edx, r.esi, r.edi, r.ebp];
    for case in 0..4u32 {
        let rect: [i32; 4] = [
            case as i32 * 8,
            16,
            if case == 2 { -24 } else { 24 },
            if case == 3 { -9 } else { 12 },
        ];
        let before = Regs {
            esi: rect.as_ptr() as u32,
            ebx: 100 + case,
            ecx: 50 * case,
            ..Regs::SENTINEL
        };
        let prepare =
            || unsafe { std::ptr::write_bytes(game_addr(QUAD_RVA) as *mut u8, 0xAA, 0x38) };
        prepare();
        take_called();
        let mut ro = before;
        unsafe { crate::framework::call_game(&mut ro, copy.addr()) };
        let theirs = (Some(regs(&ro)), take_called(), capture(&regions));
        prepare();
        let mine = unsafe {
            add_sprite(&before, &mut |p, r| {
                note(p, r);
                Ok(())
            })
        };
        let mine = (
            mine.as_ref().ok().map(regs),
            take_called(),
            capture(&regions),
        );
        t.check(theirs == mine, || {
            format!(
                "case {case}: original {:x?} {:x?}; rust {:x?} {:x?}; state same {}",
                theirs.0,
                theirs.1,
                mine.0,
                mine.1,
                theirs.2 == mine.2
            )
        });
    }
    restore(&regions, &saved);
    t.finish()
}
