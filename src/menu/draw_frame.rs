//! `menu::draw_frame` (0x47C420): draws a menu once, without the 3D scene and without
//! showing the frame: what a screen loop does when it has set a screen up.
//!
//! In:  edi = the menu; ebx, ecx as they come (`draw_menu` uses them)
//! Out: every register preserved
//! Makes the menu the current one (`dword_75558D`). Unless the game is quitting:
//! `copy_picture_to_surface` (the background), lock the surface, and if that worked
//! `draw_menu`, `run_draw_list` and unlock.
//! 89 callers, the screen loops. The lock and the unlock are the game's, through an `Env`.
//! Shadow mode runs the original alone (it draws on the real surface).

use crate::framework::{
    RawHook, Regs, SelfTest, call_game, game_addr, game_read, game_write, hook_asm, rva, shadow,
};
use crate::menu::frame_callback::{COPY_PICTURE, DRAW_MENU, LOCK_SURFACE, UNLOCK_SURFACE};
use crate::menu::frame_test::{self, Noting, QUITTING};
use crate::menu::item_at::CURRENT_MENU_RVA;
use crate::menu::screen_loop::{Env, Real};
use crate::redirect::RedirectedCopy;
use crate::service::harness::{at, capture, restore};
use std::sync::atomic::{AtomicU32, AtomicUsize};

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("menu::draw_frame", rva(0x47C420), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const RUN_DRAW_LIST: usize = rva(0x4681ED); // menu::run_draw_list

// ---- Rust version ----

/// `entry` are the registers at entry.
pub unsafe fn draw_frame(entry: &Regs, env: &mut dyn Env) {
    unsafe {
        game_write(CURRENT_MENU_RVA, entry.edi);
        if env.quitting() {
            return;
        }
        let mut g = *entry;
        env.call(COPY_PICTURE, &mut g);
        env.call(LOCK_SURFACE, &mut g);
        if g.carry() {
            return;
        }
        env.call(DRAW_MENU, &mut g);
        env.call(RUN_DRAW_LIST, &mut g);
        env.call(UNLOCK_SURFACE, &mut g);
    }
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    let entry = *r;
    unsafe { draw_frame(&entry, &mut Real) };
}

// ---- self-test: a copy of the original whose five calls go to stand-ins that note them,
// and which reads the quit switch from a test variable ----

const LEN: usize = 0x2D;
const ENTRY_BYTES: [u8; 8] = [0x60, 0x89, 0x3D, 0x8D, 0x55, 0x75, 0x00, 0x83];
const CALLS: [usize; 5] = [0x10, 0x15, 0x1C, 0x21, 0x26];
/// Where the copy names the quit switch.
const QUIT_OPERAND: usize = 0x09;
const PLACES: [(usize, u32); 5] = [
    (LOCK_SURFACE, 0),
    (COPY_PICTURE, 1),
    (DRAW_MENU, 2),
    (UNLOCK_SURFACE, 3),
    (RUN_DRAW_LIST, 8),
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
            game_addr(rva(0x47C420)),
            LEN,
            &ENTRY_BYTES,
            &CALLS,
            &redirect,
        )
    } {
        Ok(copy) => copy,
        Err(why) => return t.skip(why),
    };
    unsafe { copy.patch_u32(QUIT_OPERAND, &QUITTING as *const AtomicU32 as u32) };
    let regions = [at(CURRENT_MENU_RVA)];
    let saved = capture(&regions);
    for quitting in [0u32, 1] {
        // No call failing, then each of the five in turn.
        for failing in [u32::MAX, 0, 1, 2, 3, 4] {
            let before = Regs {
                ebx: 0x1234,
                ecx: 7,
                edi: 0x0074_8CB8,
                ..Regs::SENTINEL
            };
            let observed = || {
                (frame_test::notes(), unsafe {
                    game_read::<u32>(CURRENT_MENU_RVA)
                })
            };
            unsafe { game_write(CURRENT_MENU_RVA, 0xAAAA_AAAAu32) };
            frame_test::begin(failing, quitting);
            let mut ro = before;
            unsafe { call_game(&mut ro, copy.addr()) };
            let theirs = observed();
            unsafe { game_write(CURRENT_MENU_RVA, 0xAAAA_AAAAu32) };
            frame_test::begin(failing, quitting);
            unsafe { draw_frame(&before, &mut Noting { places: &PLACES }) };
            let mine = observed();
            let preserved = ro.preserved_except(&before, 0);
            t.check(theirs == mine && preserved, || {
                format!(
                    "quitting={quitting} failing call {failing:#x}: preserved={preserved} original {theirs:x?}; rust {mine:x?}"
                )
            });
        }
    }
    restore(&regions, &saved);
    t.finish()
}
