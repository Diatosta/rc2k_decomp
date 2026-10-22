//! `menu::inspection_screen` (0x4A1BBF): the inspection's screen shown: `dword_623FB0` /
//! `dword_623FB4` 0 and `dword_624080` 0x100, the menu scene prepared
//! (`scene::prepare_menu_scene`), the clip's screen edges 0 and 0x320 (`dword_6241DC` /
//! `dword_6241E0` and, as floats, `g_clipScreenLeft` / `Right`), the background loaded
//! (`service::load_background`, 0x470F55; its carry not read), copied to the surface and
//! the frame shown.
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Callers: `sub_48E093`, `sub_4A15E3`, `sub_4A8556`. Its four calls through an `Env` (the
//! copy and the showing are the platform's; the others are reached through their
//! addresses, Rust where hooked). Shadow mode runs the original alone (it shows a frame).
//! Self-test: a copy whose calls go to `menu::frame_test`'s stand-ins, against the Rust
//! version on the same: the calls and the variables compared.

use crate::framework::{
    RawHook, Regs, SelfTest, call_game, game_addr, game_write, hook_asm, rva, shadow,
};
use crate::menu::frame_test::{self, Noting};
use crate::menu::plain_frame::{COPY_PICTURE, SHOW_FRAME};
use crate::menu::screen_loop::{Env, Real};
use crate::redirect::RedirectedCopy;
use crate::service::harness::{Region, at, capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("menu::inspection_screen", rva(0x4A1BBF), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const PREPARE: usize = rva(0x46F7FD); // scene::prepare_menu_scene
const LOAD_BACKGROUND: usize = rva(0x470F55); // service::load_background
const ZEROED_RVA: [usize; 2] = [rva(0x623FB0), rva(0x623FB4)]; // dword_623FB0, dword_623FB4
const SCALE_RVA: usize = rva(0x624080); // dword_624080
const EDGES_RVA: [usize; 2] = [rva(0x6241DC), rva(0x6241E0)]; // dword_6241DC, dword_6241E0
const CLIP_RVA: [usize; 2] = [rva(0x6241EC), rva(0x6241F0)]; // g_clipScreenLeft, g_clipScreenRight

/// The Rust version (see the header).
pub unsafe fn inspection_screen(entry: &Regs, env: &mut dyn Env) {
    let mut g = *entry;
    unsafe {
        game_write(ZEROED_RVA[0], 0u32);
        game_write(ZEROED_RVA[1], 0u32);
        game_write(SCALE_RVA, 0x100u32);
        env.call(PREPARE, &mut g);
        game_write(EDGES_RVA[0], 0u32);
        game_write(EDGES_RVA[1], 0x320u32);
        game_write(CLIP_RVA[0], 0.0f32.to_bits());
        game_write(CLIP_RVA[1], 800.0f32.to_bits());
        for function in [LOAD_BACKGROUND, COPY_PICTURE, SHOW_FRAME] {
            env.call(function, &mut g);
        }
    }
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    unsafe { inspection_screen(r, &mut Real) };
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    unsafe { inspection_screen(r, &mut Real) };
    Ok(())
}

const PLACES: [(usize, u32); 4] = [
    (PREPARE, 0),
    (LOAD_BACKGROUND, 1),
    (COPY_PICTURE, 2),
    (SHOW_FRAME, 3),
];

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let redirect = |target: usize| {
        PLACES
            .iter()
            .find(|&&(function, _)| function == target)
            .map(|&(_, place)| frame_test::stand_in(place))
    };
    let copy = match unsafe {
        RedirectedCopy::new(
            game_addr(rva(0x4A1BBF)),
            0x61,
            // pusha; mov dword_623FB0, 0
            &[
                0x60, 0xC7, 0x05, 0xB0, 0x3F, 0x62, 0x00, 0x00, 0x00, 0x00, 0x00,
            ],
            &[0x1F, 0x50, 0x55, 0x5A],
            &redirect,
        )
    } {
        Ok(copy) => copy,
        Err(why) => return t.skip(why),
    };
    let regions: Vec<Region> = [
        ZEROED_RVA[0],
        ZEROED_RVA[1],
        SCALE_RVA,
        EDGES_RVA[0],
        EDGES_RVA[1],
        CLIP_RVA[0],
        CLIP_RVA[1],
    ]
    .map(at)
    .to_vec();
    let saved = capture(&regions);
    let before = Regs {
        ebx: 0x4321,
        ..Regs::SENTINEL
    };
    let poison = || {
        for &(p, n) in &regions {
            unsafe { std::ptr::write_bytes(p as *mut u8, 0xAA, n) };
        }
    };
    poison();
    frame_test::begin(u32::MAX, 0);
    let mut ro = before;
    unsafe { call_game(&mut ro, copy.addr()) };
    let theirs = (frame_test::notes(), capture(&regions));
    poison();
    frame_test::begin(u32::MAX, 0);
    unsafe { inspection_screen(&before, &mut Noting { places: &PLACES }) };
    let mine = (frame_test::notes(), capture(&regions));
    let preserved = ro.preserved_except(&before, 0);
    t.check(theirs == mine && preserved, || {
        format!(
            "preserved={preserved}; calls {} / {}, same {}; state same {}",
            theirs.0.len(),
            mine.0.len(),
            theirs.0 == mine.0,
            theirs.1 == mine.1
        )
    });
    restore(&regions, &saved);
    t.finish()
}
