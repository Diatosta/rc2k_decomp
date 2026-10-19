//! `menu::inspection_frame` (0x46D019): one frame of the inspection area after its first
//! (`menu::inspection_area_loop`'s later frame callback): `menu::frame_callback`'s steps in
//! another order and without its notice: lock the surface; the background
//! (`copy_picture_to_surface`); the draw list (`g_drawList`); unlock; `dword_6365CC` 4;
//! begin the 3D scene; the scene list (`g_sceneList`, each between `pusha` and `popa`);
//! end the scene; lock again; `draw_menu`; the overlay list (`g_overlayList`); unlock;
//! show the frame. A set carry after either lock, the scene's beginning or the showing ends
//! it there (carry set), with what was locked left locked; else the carry is cleared.
//!
//! In:  edi = the menu; ebx, ecx as they come (`draw_menu` uses them)
//! Out: carry = the frame could not be made; every register kept (`pusha` .. `popa`)
//! The draw and overlay lists' functions get the registers as the one before left them.
//! The surface, the scene and the showing are the game's (DirectDraw and Direct3D),
//! reached through an `Env` like `frame_callback`'s. Shadow mode runs the original alone (a
//! frame cannot be shown twice). Manual patch with a private copy (0x7F bytes, nine calls
//! relocated): the first call starts at the second byte. Self-test: a copy whose calls go to
//! `frame_test`'s stand-ins, with lists of the test's, each call failing in turn.

use crate::framework::{
    RawHook, Regs, SelfTest, call_game, game_addr, game_read, game_write, hook_asm, rva, shadow,
};
use crate::menu::frame_callback::{COPY_PICTURE, DRAW_MENU, LOCK_SURFACE, UNLOCK_SURFACE};
use crate::menu::frame_test::{self, CHANGING, Noting};
use crate::menu::screen_loop::{Env, Real};
use crate::redirect::RedirectedCopy;
use crate::service::harness::{at, capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new("menu::inspection_frame", rva(0x46D019), thunk, &ORIGINAL)
    .private_copy(LEN)
    .relocate_calls(&CALLS)
    .manual_patch()
    .self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const BEGIN_SCENE: usize = rva(0x44E8A1); // scene::begin_scene
const END_SCENE: usize = rva(0x44E952); // scene::end_scene
const SHOW_FRAME: usize = rva(0x4057D1); // gfx::show_frame
const DRAW_LIST_RVA: usize = rva(0x7554F8); // g_drawList
const SCENE_LIST_RVA: usize = rva(0x755504); // g_sceneList
const OVERLAY_LIST_RVA: usize = rva(0x7554FC); // g_overlayList
const SCENE_STATE_RVA: usize = rva(0x6365CC); // dword_6365CC
const END: u32 = u32::MAX;
const LEN: usize = 0x7F;
const CALLS: [usize; 9] = [0x1, 0x8, 0x1F, 0x2E, 0x49, 0x4E, 0x55, 0x6C, 0x71];

/// The functions of the list esi points at; `each_alone` as `frame_callback`'s.
unsafe fn run_listed(g: &mut Regs, env: &mut dyn Env, each_alone: bool) {
    unsafe {
        while (g.esi as *const u32).read_unaligned() != END {
            let function = (g.esi as *const u32).read_unaligned() as usize;
            let rva = function.wrapping_sub(game_addr(0));
            if each_alone {
                let mut own = *g;
                env.call(rva, &mut own);
                g.eflags = own.eflags;
            } else {
                env.call(rva, g);
            }
            g.esi = g.esi.wrapping_add(4);
        }
    }
}

/// The Rust version: whether the frame was made.
pub unsafe fn inspection_frame(entry: &Regs, env: &mut dyn Env) -> bool {
    unsafe {
        let mut g = *entry;
        env.call(LOCK_SURFACE, &mut g);
        if g.carry() {
            return false;
        }
        env.call(COPY_PICTURE, &mut g);
        g.esi = game_read(DRAW_LIST_RVA);
        run_listed(&mut g, env, false);
        env.call(UNLOCK_SURFACE, &mut g);
        game_write(SCENE_STATE_RVA, 4u32);
        env.call(BEGIN_SCENE, &mut g);
        if g.carry() {
            return false;
        }
        g.esi = game_read(SCENE_LIST_RVA);
        run_listed(&mut g, env, true);
        env.call(END_SCENE, &mut g);
        env.call(LOCK_SURFACE, &mut g);
        if g.carry() {
            return false;
        }
        env.call(DRAW_MENU, &mut g);
        g.esi = game_read(OVERLAY_LIST_RVA);
        run_listed(&mut g, env, false);
        env.call(UNLOCK_SURFACE, &mut g);
        env.call(SHOW_FRAME, &mut g);
        !g.carry()
    }
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    let entry = *r;
    let made = unsafe { inspection_frame(&entry, &mut Real) };
    r.set_carry(!made);
}

// ---- self-test ----

const ENTRY_BYTES: [u8; 8] = [0x60, 0xE8, 0x8E, 0x88, 0xF9, 0xFF, 0x72, 0x74];
const PLACES: [(usize, u32); 7] = [
    (LOCK_SURFACE, 0),
    (COPY_PICTURE, 1),
    (DRAW_MENU, 2),
    (UNLOCK_SURFACE, 3),
    (BEGIN_SCENE, 4),
    (END_SCENE, 5),
    (SHOW_FRAME, 7),
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
            game_addr(rva(0x46D019)),
            LEN,
            &ENTRY_BYTES,
            &CALLS,
            &redirect,
        )
    } {
        Ok(copy) => copy,
        Err(why) => return t.skip(why),
    };
    let listed = |places: &[u32]| -> Vec<u32> {
        places
            .iter()
            .map(|&place| frame_test::stand_in(place) as u32)
            .chain([END])
            .collect()
    };
    let regions = [
        at(DRAW_LIST_RVA),
        at(SCENE_LIST_RVA),
        at(OVERLAY_LIST_RVA),
        at(SCENE_STATE_RVA),
    ];
    let saved = capture(&regions);
    // A game thread between two hooks when the pause began gets to its next one before the
    // lists are changed under it.
    std::thread::sleep(std::time::Duration::from_millis(30));
    let lists: [(&[u32], &[u32], &[u32]); 4] = [
        (&[], &[], &[]),
        (&[9], &[11], &[CHANGING]),
        (&[9, CHANGING, 11], &[CHANGING, 9], &[11, CHANGING, 9]),
        (&[CHANGING], &[9, 11, CHANGING], &[]),
    ];
    for (draw, scene, overlay) in lists {
        let (draw, scene, overlay) = (listed(draw), listed(scene), listed(overlay));
        let prepare = || unsafe {
            game_write(DRAW_LIST_RVA, draw.as_ptr() as u32);
            game_write(SCENE_LIST_RVA, scene.as_ptr() as u32);
            game_write(OVERLAY_LIST_RVA, overlay.as_ptr() as u32);
            game_write(SCENE_STATE_RVA, 0xAAAA_AAAAu32);
        };
        let observed = || {
            (frame_test::notes(), unsafe {
                game_read::<u32>(SCENE_STATE_RVA)
            })
        };
        let mut calls = u32::MAX;
        let mut failing = u32::MAX;
        loop {
            let before = Regs {
                ebx: 0x1234,
                ecx: 7,
                ..Regs::SENTINEL
            };
            prepare();
            frame_test::begin(failing, 0);
            let mut ro = before;
            unsafe { call_game(&mut ro, copy.addr()) };
            let theirs = (observed(), !ro.carry());
            prepare();
            frame_test::begin(failing, 0);
            let made = unsafe { inspection_frame(&before, &mut Noting { places: &PLACES }) };
            let mine = (observed(), made);
            let preserved = ro.preserved_except(&before, 0);
            t.check(theirs == mine && preserved, || {
                format!(
                    "lists of {} {} {} failing call {failing:#x}: preserved={preserved} original made={} calls={} state {:x}; rust made={} calls={} state {:x}",
                    draw.len() - 1,
                    scene.len() - 1,
                    overlay.len() - 1,
                    theirs.1,
                    theirs.0.0.len(),
                    theirs.0.1,
                    mine.1,
                    mine.0.0.len(),
                    mine.0.1
                )
            });
            if calls == u32::MAX {
                calls = theirs.0.0.len() as u32;
                failing = 0;
            } else {
                failing += 1;
            }
            if failing >= calls {
                break;
            }
        }
    }
    restore(&regions, &saved);
    t.finish()
}
