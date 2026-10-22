//! `race::draw_lamp_flare` (0x4551F4): lamp ebx (`g_lamps`) as a flare: its point, size
//! (+4 x 8) and colour (+8), object `dword_621508`, placed by `race::place_object_matrix`
//! and drawn (`scene::draw_scene_object`), see `race/frame_draw.rs`.
//!
//! In:  ebx = the lamp
//! Out: every register preserved (`pusha` .. `popa`)
//!
//! Caller: `race::draw_list_entry` (lamps 0x3000000 ..). Shadow mode runs the original
//! alone (it draws). Self-test: a copy whose drawing call goes to a stand-in, against the
//! Rust version on the same; test lamps on points of the test's; all put back.

use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_write, hook_asm, rva, shadow};
use crate::race::draw_flare::{compare_draws, copy_with_draw, flare_draw_regions};
use crate::race::frame_draw::*;
use crate::race::level_init::note;
use crate::race::place_object_matrix::{object_inputs, set_object_inputs};
use crate::service::harness::{at, capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::draw_lamp_flare", rva(0x4551F4), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    if let Err(e) = unsafe { draw_lamp_flare(r, &mut entry_part) } {
        unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) };
    }
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    unsafe { draw_lamp_flare(r, &mut entry_part) }?;
    Ok(())
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let copy = match copy_with_draw(
        &HOOK,
        0x4551F4,
        0x6F,
        &[0x60, 0xC1, 0xE3, 0x04, 0x8D, 0xBB, 0x5C, 0xE0],
        &[0x59, 0x68],
    ) {
        Ok(c) => c,
        Err(e) => return t.skip(&format!("copy: {e}")),
    };
    let regions = flare_draw_regions();
    let material_at = game_addr(rva(0x97DF30)) as u32 + 0x20;
    let mut kept = regions.clone();
    kept.extend(object_inputs());
    kept.extend([
        (game_addr(rva(0x63E05C)) as u32, 0x100),
        at(rva(0x5CCCC0)),
        at(rva(0x621508)),
        (material_at, 1),
    ]);
    let saved = capture(&kept);
    let points: Vec<u32> = (0..48u32)
        .map(|k| k.wrapping_mul(0x2F1D3) % 0x30_0000)
        .collect();
    let mut seed = 0x1A3F_0001u32;
    for (lamp, way) in [(0u32, 3u32), (7, 5), (15, 4)] {
        let s = seed;
        seed = seed.wrapping_mul(0x9E37_79B9).wrapping_add(1);
        let prepare = || unsafe {
            restore(&kept, &saved);
            let mut s2 = s;
            set_object_inputs(&mut s2, way, material_at);
            game_write(rva(0x5CCCC0), points.as_ptr() as u32);
            game_write(rva(0x621508), 1u32);
            let l = game_addr(rva(0x63E05C)) as u32 + 16 * lamp;
            (l as *mut u32).write_unaligned(lamp % 16);
            ((l + 4) as *mut u32).write_unaligned(0x40 + lamp);
            ((l + 8) as *mut u32).write_unaligned(0x00C0_8040);
        };
        compare_draws(
            &mut t,
            &format!("lamp {lamp} way {way}"),
            &copy,
            Regs {
                ebx: lamp,
                ..Regs::SENTINEL
            },
            &prepare,
            &regions,
            &|r| unsafe {
                draw_lamp_flare(r, &mut |p, r| {
                    note(p, r);
                    Ok(())
                })
            },
        );
    }
    restore(&kept, &saved);
    drop(points);
    t.finish()
}
