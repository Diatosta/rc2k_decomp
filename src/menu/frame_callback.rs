//! `menu::frame_callback` (0x474D4B): one frame of a menu screen, the callback nearly every
//! screen loop hands to `screens::run_frame`.
//!
//! In:  edi = the menu; ebx, ecx as they come (`draw_menu` uses them)
//! Out: carry = the frame could not be made; every register preserved
//! In this order: lock the surface; `copy_picture_to_surface` (the background);
//! `draw_menu`; every function of the draw list (`g_drawList`); unlock; `dword_6365CC` =
//! 4; begin the 3D scene (`sub_44E8A1`); every function of the scene list (`g_sceneList`),
//! each between `pusha` and `popa`; end the scene (`sub_44E952`); lock again; every
//! function of the overlay list (`g_overlayList`); if `byte_4E07A0` is 2 and
//! `dword_7585D8` is 1, clear that and draw string 0x1000 at (800, 587) with the font
//! 0x51E404; unlock; show the frame (`sub_4057D1`). A set carry after either lock, the
//! scene's beginning or the showing ends it there, with what was locked left locked.
//! The functions of the draw and overlay lists get the registers as the one before left
//! them; a list ends with -1.
//! The surface's lock and unlock, the 3D scene and the showing of the frame are the game's
//! (DirectDraw and Direct3D calls), reached like everything else here through an `Env`:
//! that is where a build without the game puts its own.
//! Shadow mode runs the original alone (a frame cannot be shown twice). Manual patch with
//! a private copy (0xB9 bytes, ten calls relocated): the first call starts at the second
//! byte, inside what the patch overwrites.

use crate::framework::{
    RawHook, Regs, SelfTest, call_game, game_addr, game_read, game_write, hook_asm, rva, shadow,
};
use crate::menu::frame_test::{self, CHANGING, Noting};
use crate::menu::screen_loop::{Env, Real};
use crate::redirect::RedirectedCopy;
use crate::service::harness::{at, capture, restore};
use rc2k_core::menu::notice;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new("menu::frame_callback", rva(0x474D4B), thunk, &ORIGINAL)
    .private_copy(0xB9)
    .relocate_calls(&[0x01, 0x0C, 0x11, 0x28, 0x37, 0x52, 0x57, 0xA1, 0xA6, 0xAB])
    .manual_patch()
    .self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

pub const LOCK_SURFACE: usize = rva(0x4058AD); // gfx::lock_surface
pub const UNLOCK_SURFACE: usize = rva(0x405949); // gfx::unlock_surface
pub const COPY_PICTURE: usize = rva(0x4060E3); // gfx::copy_picture_to_surface
pub const DRAW_MENU: usize = rva(0x4746B7); // menu::draw_menu
const BEGIN_SCENE: usize = rva(0x44E8A1); // sub_44E8A1
const END_SCENE: usize = rva(0x44E952); // sub_44E952
const SHOW_FRAME: usize = rva(0x4057D1); // sub_4057D1
const DRAW_STRING_BY_ID: usize = rva(0x40234A); // text::draw_string_by_id
const DRAW_LIST_RVA: usize = rva(0x7554F8); // g_drawList
const SCENE_LIST_RVA: usize = rva(0x755504); // g_sceneList
const OVERLAY_LIST_RVA: usize = rva(0x7554FC); // g_overlayList
const SCENE_STATE_RVA: usize = rva(0x6365CC); // dword_6365CC
const NOTICE_KIND_RVA: usize = rva(0x4E07A0); // byte_4E07A0: 2 = the notice below may be drawn
const NOTICE_DUE_RVA: usize = rva(0x7585D8); // dword_7585D8: set to 1 by the main menu before a frame
const NOTICE_FONT_RVA: usize = rva(0x51E404); // stru_51E404
const END: u32 = u32::MAX;

// ---- Rust version ----

/// Calls the functions of the list esi points at, one after the other; esi moves along.
/// `each_alone`: every function gets the registers as they are before it (the scene list's
/// `pusha` and `popa`), not as the one before left them.
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

/// `entry` are the registers at entry. Returns whether the frame was made.
pub unsafe fn frame_callback(entry: &Regs, env: &mut dyn Env) -> bool {
    unsafe {
        let mut g = *entry;
        env.call(LOCK_SURFACE, &mut g);
        if g.carry() {
            return false;
        }
        env.call(COPY_PICTURE, &mut g);
        env.call(DRAW_MENU, &mut g);
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
        g.esi = game_read(OVERLAY_LIST_RVA);
        run_listed(&mut g, env, false);
        // Converted: `rc2k_core::menu::notice::drawn_now`.
        if notice::drawn_now(game_read(NOTICE_KIND_RVA), || game_read(NOTICE_DUE_RVA)) {
            game_write(NOTICE_DUE_RVA, 0u32);
            g.eax = notice::STRING;
            g.ebx = game_addr(NOTICE_FONT_RVA) as u32;
            (g.ecx, g.edx) = notice::AT;
            env.call(DRAW_STRING_BY_ID, &mut g);
        }
        env.call(UNLOCK_SURFACE, &mut g);
        env.call(SHOW_FRAME, &mut g);
        !g.carry()
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
    let made = unsafe { frame_callback(&entry, &mut Real) };
    r.set_carry(!made);
}

// ---- self-test: a copy of the original whose ten calls go to stand-ins that note them,
// with lists of the test's made of more such stand-ins ----

const LEN: usize = 0xB9;
const ENTRY_BYTES: [u8; 8] = [0x60, 0xE8, 0x5C, 0x0B, 0xF9, 0xFF, 0x0F, 0x82];
const CALLS: [usize; 10] = [0x01, 0x0C, 0x11, 0x28, 0x37, 0x52, 0x57, 0xA1, 0xA6, 0xAB];
/// The place each game function has among the stand-ins; 9 to 11 are the lists' own.
const PLACES: [(usize, u32); 8] = [
    (LOCK_SURFACE, 0),
    (COPY_PICTURE, 1),
    (DRAW_MENU, 2),
    (UNLOCK_SURFACE, 3),
    (BEGIN_SCENE, 4),
    (END_SCENE, 5),
    (DRAW_STRING_BY_ID, 6),
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
            game_addr(rva(0x474D4B)),
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
        at(NOTICE_KIND_RVA),
        at(NOTICE_DUE_RVA),
    ];
    let saved = capture(&regions);
    // A game thread that was between two hooks when the pause began gets to its next one
    // before the lists are changed under it.
    std::thread::sleep(std::time::Duration::from_millis(30));

    // (draw list, scene list, overlay list) as places, (notice kind, notice due)
    let lists: [(&[u32], &[u32], &[u32]); 4] = [
        (&[], &[], &[]),
        (&[9], &[11], &[CHANGING]),
        (&[9, CHANGING, 11], &[CHANGING, 9], &[11, CHANGING, 9]),
        (&[CHANGING], &[9, 11, CHANGING], &[]),
    ];
    for (draw, scene, overlay) in lists {
        let (draw, scene, overlay) = (listed(draw), listed(scene), listed(overlay));
        for (kind, due) in [(2u8, 1u32), (2, 0), (1, 1), (0, 0)] {
            let prepare = || unsafe {
                game_write(DRAW_LIST_RVA, draw.as_ptr() as u32);
                game_write(SCENE_LIST_RVA, scene.as_ptr() as u32);
                game_write(OVERLAY_LIST_RVA, overlay.as_ptr() as u32);
                game_write(SCENE_STATE_RVA, 0xAAAA_AAAAu32);
                game_write(NOTICE_KIND_RVA, kind);
                game_write(NOTICE_DUE_RVA, due);
            };
            let observed = || {
                (frame_test::notes(), unsafe {
                    (
                        game_read::<u32>(SCENE_STATE_RVA),
                        game_read::<u32>(NOTICE_DUE_RVA),
                    )
                })
            };
            // First with every call working, which says how many calls there are; then with
            // each of them failing in turn.
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
                let made = unsafe { frame_callback(&before, &mut Noting { places: &PLACES }) };
                let mine = (observed(), made);
                let preserved = ro.preserved_except(&before, 0);
                t.check(theirs == mine && preserved, || {
                    let first = theirs.0.0.iter().zip(&mine.0.0).position(|(a, b)| a != b);
                    format!(
                        "lists of {} {} {} kind={kind} due={due} failing call {failing:#x}: preserved={preserved} original made={} calls={} left {:x?}; rust made={} calls={} left {:x?}; first different call {first:?}: {:x?} / {:x?}",
                        draw.len() - 1,
                        scene.len() - 1,
                        overlay.len() - 1,
                        theirs.1,
                        theirs.0.0.len(),
                        theirs.0.1,
                        mine.1,
                        mine.0.0.len(),
                        mine.0.1,
                        first.map(|n| theirs.0.0[n]),
                        first.map(|n| mine.0.0[n])
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
    }
    restore(&regions, &saved);
    t.finish()
}
