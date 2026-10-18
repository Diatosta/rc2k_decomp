//! `race::draw_sky` (0x455540): the sky and the sun for a view (see `race/frame_draw.rs`):
//! `race::fetch_view_value`; unless the scene's mode is 1 or 2, the sky (object 1 on
//! `race::sky_matrix`, full light) drawn; then the sun's direction turned by the sky's
//! matrix (`scene::rotate_point`) and, in front, its place and size on the screen for the
//! flare (`dword_6415C4` .. `dword_6415DC`).
//!
//! In:  nothing
//! Out: the registers as its parts leave them (a `call` follows)
//!
//! It begins with its call: private copy (0x26B, its seven calls relocated), manual
//! patch. In the game the render states (`sub_44E44D`) and the batch's flush
//! (`sub_45FD1D`) are called in the game (section 15); the drawing by
//! `scene::draw_scene_object_in_game` and the depth buffer by `scene::depth_buffer_on`'s
//! Rust versions. Shadow mode runs the original
//! alone (it draws). Self-test: a copy whose four drawing calls go to stand-ins that note
//! eax .. ecx, against the Rust version on the same; the sky's and the sun's inputs set
//! (the sun in front and behind, flat and not, on and off the screen); all put back.

use crate::framework::{
    RawHook, Regs, SelfTest, call_game, game_addr, game_write, hook_asm, rva, shadow,
};
use crate::race::frame_draw::*;
use crate::race::level_init::{note, take_called};
use crate::race::sky_matrix::sky_regions;
use crate::service::harness::{Region, at, capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

const CALLS: [usize; 7] = [0x00, 0x29, 0x91, 0xA0, 0xA5, 0xAA, 0x163];

pub static HOOK: RawHook = RawHook::new("race::draw_sky", rva(0x455540), thunk, &ORIGINAL)
    .private_copy(0x26B)
    .relocate_calls(&CALLS)
    .manual_patch()
    .self_test(self_test);

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
    match unsafe { draw_sky(r, &mut sky_part) } {
        Ok(after) => {
            let esp = r.esp;
            *r = after;
            r.esp = esp;
        }
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

fn regions() -> Vec<Region> {
    let g = |a: usize, len: usize| (game_addr(rva(a)) as u32, len);
    let mut v = sky_regions();
    v.extend([
        at(rva(0x6415D8)),
        at(rva(0x5CCCD4)),
        g(0x5EA34C, 0x40),
        at(rva(0x6214FC)),
        at(rva(0x6365C8)),
        g(0x635084, 0xC),
        g(0x635094, 8),
        at(rva(0x621510)),
        at(rva(0x62DAF4)),
        g(0x6415AC, 0x34),
    ]);
    v
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let blocks: Vec<u32> = crate::race::frame::VIEW_BLOCKS
        .iter()
        .map(|&a| unsafe { crate::framework::game_read::<u32>(a) })
        .collect();
    if blocks.contains(&0) {
        return t.skip("no view blocks (no race loaded)");
    }
    crate::install::stand_ins::set_handler(note);
    let copy = match crate::audio::test_copy::make(
        &HOOK,
        &crate::audio::test_copy::Copy {
            address: 0x455540,
            len: 0x26B,
            entry: &[0xE8, 0xC9, 0xFF, 0xFF, 0xFF, 0xC7, 0x05, 0xD4],
            calls: &CALLS,
            jumps_out: &[],
        },
        &|c| {
            SKY_CALLEES
                .iter()
                .find(|&&(_, k)| rva(k) == c)
                .map(|&(place, _)| crate::install::stand_ins::stand_in(place))
        },
        &[],
        0,
    ) {
        Ok(c) => c,
        Err(e) => return t.skip(&format!("copy: {e}")),
    };
    let regions = regions();
    let mut kept = regions.clone();
    for a in [
        0x6365C4usize,
        0x609310,
        0x609360,
        0x609334,
        0x609338,
        0x60933C,
        0x60932C,
        0x609318,
        0x6241D0,
        0x5CCD80,
        0x63DB9C,
    ] {
        kept.push(at(rva(a)));
    }
    kept.push((game_addr(rva(0x5EA24C)) as u32, 0x40));
    kept.extend(blocks.iter().map(|&b| (b + 0xFBC, 4)));
    let saved = capture(&kept);
    let mut seed = 0x5CA1_0001u32;
    for (mode, sun, angle, height, flat) in [
        (0u32, 0u32, 0u32, 0u32, 0u32),
        (2, 1, 0, 0, 0),
        (0, 1, 0x40, 0x20, 0),
        (0, 1, 0xC0, 0x08, 0),
        (3, 1, 0x10, 0x30, 1),
        (0, 1, 0x80, 0xF0, 0),
    ] {
        let view = crate::scene::matrix::test_floats(&mut seed);
        let prepare = || unsafe {
            restore(&kept, &saved);
            game_write(rva(0x6365C4), mode);
            game_write(rva(0x609310), sun);
            game_write(rva(0x609360), 0u32);
            game_write(rva(0x609334), angle);
            game_write(rva(0x609338), 0x11u32);
            game_write(rva(0x60933C), height);
            game_write(rva(0x60932C), flat);
            game_write(rva(0x609318), 0x40u32);
            game_write(rva(0x6241D0), 0x2000u32);
            game_write(rva(0x5CCD80), 0x100u32);
            game_write(rva(0x63DB9C), 0u32);
            // A plain turn: the identity with a little of the test's floats.
            let mut m = [0f32; 16];
            for k in 0..4 {
                m[k * 5] = 1.0;
            }
            m[1] = f32::from_bits(view[1]).fract() * 0.1;
            game_write(rva(0x5EA24C), m.map(f32::to_bits));
            take_called();
        };
        let regs = |r: &Regs| [r.eax, r.ebx, r.ecx, r.edx, r.esi, r.edi, r.ebp];
        prepare();
        let mut ro = Regs::SENTINEL;
        unsafe { call_game(&mut ro, copy.addr()) };
        let theirs = (Some(regs(&ro)), take_called(), capture(&regions));
        prepare();
        let mine = unsafe {
            draw_sky(&Regs::SENTINEL, &mut |p, r| {
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
            let diff = theirs.2.iter().zip(&mine.2).position(|(a, b)| a != b);
            format!(
                "mode {mode} sun {sun} angle {angle:#x} height {height:#x} flat {flat}: original {:x?} {:x?}; rust {:x?} {:x?}; memory first differs at {diff:x?}",
                theirs.0, theirs.1, mine.0, mine.1
            )
        });
    }
    restore(&kept, &saved);
    t.finish()
}
