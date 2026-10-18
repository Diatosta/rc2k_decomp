//! `hud::add_triangle` (0x4B6DA7): `g_polygonVertexCount` 3 and `hud::add_polygon`.
//!
//! In:  nothing
//! Out: all kept (`pusha` .. `popa`)
//!
//! Caller: `sub_4B6D10` (0x4B6D03). The polygon through `part` (1), in the game its Rust
//! version. Shadow mode runs the original alone (it draws). Self-test: a copy whose call
//! goes to a stand-in, against the Rust version.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_write, hook_asm, rva, shadow};
use crate::hud::batch::POLYGON_COUNT_RVA;
use crate::service::harness::{Region, at, capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("hud::add_triangle", rva(0x4B6DA7), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// A call by place: 1 `add_polygon`.
pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn add_triangle(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    unsafe { game_write(POLYGON_COUNT_RVA, 3u32) };
    let mut r = *entry;
    part(1, &mut r)?;
    Ok(*entry)
}

/// In the game, for the quads' callers: `add_quad`'s Rust version.
pub fn game_part_quad(_place: u32, r: &mut Regs) -> Result<(), Refused> {
    use crate::hud::add_quad as aq;
    *r = unsafe { aq::add_quad(r, &mut aq::game_part) }?;
    Ok(())
}

/// In the game, for `draw_disc`: this function's Rust version.
pub fn game_part_triangle(_place: u32, r: &mut Regs) -> Result<(), Refused> {
    *r = unsafe { add_triangle(r, &mut crate::hud::add_quad::game_part) }?;
    Ok(())
}

/// In the game, for the dials' set-ups: `draw_disc`'s Rust version.
pub fn game_part_disc(_place: u32, r: &mut Regs) -> Result<(), Refused> {
    *r = unsafe { crate::hud::draw_disc::draw_disc(r, &mut game_part_triangle) }?;
    Ok(())
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    match unsafe { add_triangle(r, &mut crate::hud::add_quad::game_part) } {
        Ok(left) => *r = Regs { esp: r.esp, ..left },
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

fn self_test() -> (usize, usize) {
    use crate::race::mud::{copy, run_both, test_part};
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // pusha; mov g_polygonVertexCount, 3
    let entry: &[u8] = &[
        0x60, 0xC7, 0x05, 0xF8, 0x40, 0x62, 0x00, 0x03, 0x00, 0x00, 0x00,
    ];
    let copy = match copy(0x4B6DA7, 0x12, entry, &[0xB], &[(0x4B7413, 1)]) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let regions: Vec<Region> = vec![at(POLYGON_COUNT_RVA)];
    let saved = capture(&regions);
    run_both(
        &mut t,
        "once",
        &copy,
        &regions,
        &|| unsafe { game_write(POLYGON_COUNT_RVA, 9u32) },
        &|r| unsafe { add_triangle(r, &mut test_part) },
    );
    restore(&regions, &saved);
    t.finish()
}
