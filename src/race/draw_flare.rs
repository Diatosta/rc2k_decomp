//! `race::draw_flare` (0x455189) and the shared test of `race::draw_lamp_flare`: a flare
//! from the flare tables (or a lamp) placed by `race::place_object_matrix` and drawn
//! (`scene::draw_scene_object`), see `race/frame_draw.rs`.
//!
//! In:  ebx = the flare
//! Out: every register preserved (`pusha` .. `popa`)
//!
//! Caller: `race::draw_list_entry` (flares 0x1000000 ..). Shadow mode runs the original
//! alone (it draws). Self-test: a copy whose drawing call goes to a stand-in that notes
//! eax .. ecx (its matrix call real), against the Rust version on the same; flares of the
//! test's in the tables; all put back.

use crate::framework::{
    RawHook, Regs, SelfTest, call_game, game_addr, game_write, hook_asm, rva, shadow,
};
use crate::race::frame_draw::*;
use crate::race::level_init::{note, take_called};
use crate::race::place_object_matrix::{object_inputs, object_matrix_regions, set_object_inputs};
use crate::redirect::RedirectedCopy;
use crate::service::harness::{Region, at, capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::draw_flare", rva(0x455189), thunk, &ORIGINAL).self_test(self_test);

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
    if let Err(e) = unsafe { draw_flare(r, &mut entry_part) } {
        unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) };
    }
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    unsafe { draw_flare(r, &mut entry_part) }?;
    Ok(())
}

/// A copy of `address` whose drawing call (at `call`) goes to stand-in 6.
pub fn copy_with_draw(
    hook: &RawHook,
    address: usize,
    len: usize,
    entry: &[u8],
    calls: &[usize],
) -> Result<RedirectedCopy, String> {
    crate::install::stand_ins::set_handler(note);
    crate::audio::test_copy::make(
        hook,
        &crate::audio::test_copy::Copy {
            address,
            len,
            entry,
            calls,
            jumps_out: &[],
        },
        &|c| (c == rva(0x451E3E)).then(|| crate::install::stand_ins::stand_in(6)),
        &[],
        0,
    )
}

/// What a flare's or lamp's drawing writes besides the matrices.
pub fn flare_draw_regions() -> Vec<Region> {
    let mut v = object_matrix_regions();
    v.extend([
        (game_addr(rva(0x63501C)) as u32, 0xC),
        at(rva(0x635038)),
        at(rva(0x635054)),
        at(rva(0x6214FC)),
        at(rva(0x6365C8)),
        at(rva(0x62DAF4)),
    ]);
    v
}

/// Runs a copy and the Rust version (`rust`) on the same, `prepare` before each.
pub fn compare_draws(
    t: &mut SelfTest,
    what: &str,
    copy: &RedirectedCopy,
    entry: Regs,
    prepare: &dyn Fn(),
    regions: &[Region],
    rust: &dyn Fn(&Regs) -> Result<(), crate::errors::Refused>,
) {
    let regs = |r: &Regs| [r.eax, r.ebx, r.ecx, r.edx, r.esi, r.edi, r.ebp];
    prepare();
    take_called();
    let mut ro = entry;
    unsafe { call_game(&mut ro, copy.addr()) };
    let theirs = (regs(&ro), take_called(), capture(regions));
    prepare();
    let done = rust(&entry);
    let mine = (regs(&entry), take_called(), capture(regions));
    t.check(done.is_ok() && theirs == mine, || {
        let diff = theirs.2.iter().zip(&mine.2).position(|(a, b)| a != b);
        format!(
            "{what}: original {:x?} {:x?}; rust {:x?} {:x?} {done:?}; memory first differs at {diff:x?}",
            theirs.0, theirs.1, mine.0, mine.1
        )
    });
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let copy = match copy_with_draw(
        &HOOK,
        0x455189,
        0x6B,
        &[0x60, 0x8B, 0x04, 0x9D, 0x8C, 0xF1, 0x63, 0x00],
        &[0x53, 0x64],
    ) {
        Ok(c) => c,
        Err(e) => return t.skip(&format!("copy: {e}")),
    };
    let regions = flare_draw_regions();
    let material_at = game_addr(rva(0x97DF30)) as u32 + 0x20;
    let mut kept = regions.clone();
    kept.extend(object_inputs());
    kept.extend([(game_addr(rva(0x63F18C)) as u32, 0x2404), (material_at, 1)]);
    let saved = capture(&kept);
    let mut seed = 0xF1A2_0001u32;
    for (flare, way) in [(0u32, 3u32), (5, 1), (0x3FF, 4)] {
        let s = seed;
        seed = seed.wrapping_mul(0x9E37_79B9).wrapping_add(1);
        let prepare = || unsafe {
            restore(&kept, &saved);
            let mut s2 = s;
            set_object_inputs(&mut s2, way, material_at);
            for (k, table) in [0x63F18Cusize, 0x63F58C, 0x63F98C, 0x64018C, 0x63FD8C]
                .iter()
                .enumerate()
            {
                game_write(
                    rva(table + 4 * flare as usize),
                    [0x10_0000u32, 0x2_0000, 0x18_0000, 0x200, 0x60_6060][k],
                );
            }
            game_write(rva(0x64058C + 4 * flare as usize), 1u32);
        };
        compare_draws(
            &mut t,
            &format!("flare {flare:#x} way {way}"),
            &copy,
            Regs {
                ebx: flare,
                ..Regs::SENTINEL
            },
            &prepare,
            &regions,
            &|r| unsafe {
                draw_flare(r, &mut |p, r| {
                    note(p, r);
                    Ok(())
                })
            },
        );
    }
    restore(&kept, &saved);
    t.finish()
}
