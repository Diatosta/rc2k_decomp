//! `scene::draw_main_menu_objects` (0x490106): the 3D part of a frame of the menus
//! with a carousel.
//!
//! In:  ecx, passed on to the carousel
//! Out: every register preserved
//!
//! `scene::clear_depth_buffer`; `scene::assign_kind_textures`;
//! `scene::choose_carousel_pictures`; `scene::assign_menu_textures`;
//! `scene::map_texture_places`; `scene::load_slot_textures`;
//! `scene::draw_main_menu_carousel`. Each is entered with the registers the one
//! before left.
//! Reached through the menus' scene lists.
//! Shadow mode runs the original alone. Manual patch with a private copy: a call in
//! the first bytes.
//! Self-test: a redirected copy whose seven calls note themselves.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, call_game, game_addr, hook_asm, rva, shadow};
use crate::install::stand_ins::{act, set_handler, stand_in};
use crate::redirect::RedirectedCopy;
use crate::scene::assign_kind_textures::assign_kind_textures;
use crate::scene::assign_menu_textures::assign_menu_textures;
use crate::scene::choose_carousel_pictures::choose_carousel_pictures;
use crate::scene::clear_depth_buffer::clear_depth_buffer_in_game;
use crate::scene::draw_carousel::draw_carousel_in_game;
use crate::scene::draw_main_menu_carousel::draw_main_menu_carousel;
use crate::scene::load_slot_texture::load_slot_texture_in_game;
use crate::scene::load_slot_textures::load_slot_textures;
use crate::scene::map_texture_places::map_texture_places;
use std::sync::Mutex;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "scene::draw_main_menu_objects",
    rva(0x490106),
    thunk,
    &ORIGINAL,
)
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

const STEPS: [usize; 7] = [
    rva(0x44E967),
    rva(0x4256D8),
    rva(0x4901B9),
    rva(0x490DF3),
    rva(0x42570D),
    rva(0x425778),
    rva(0x49012C),
];

// ---- Rust version ----

/// `step`: what stands for the seven calls, by their order.
pub unsafe fn draw_main_menu_objects(
    r: &Regs,
    step: &mut dyn FnMut(usize, &mut Regs) -> Result<(), Refused>,
) -> Result<(), Refused> {
    let mut inner = *r;
    for n in 0..STEPS.len() {
        step(n, &mut inner)?;
    }
    Ok(())
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    let done = unsafe {
        draw_main_menu_objects(r, &mut |n, r| {
            match n {
                0 => clear_depth_buffer_in_game(r)?,
                1 => assign_kind_textures()?.apply(r),
                2 => choose_carousel_pictures()?.apply(r),
                3 => assign_menu_textures()?,
                4 => map_texture_places().apply(r),
                5 => {
                    let entry = *r;
                    load_slot_textures(&mut |slot| load_slot_texture_in_game(slot, &entry))?
                        .apply(r)
                }
                _ => draw_main_menu_carousel(r, &mut |r| draw_carousel_in_game(r))?,
            }
            Ok(())
        })
    };
    if let Err(e) = done {
        unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) }
    }
}

// ---- self-test ----

const LEN: usize = 0x26;
const ENTRY_BYTES: [u8; 6] = [0x60, 0xE8, 0x5B, 0xE8, 0xFB, 0xFF];
const CALLS: [usize; 7] = [0x01, 0x06, 0x0B, 0x10, 0x15, 0x1A, 0x1F];

/// Each call: its place and eax, ecx as it was entered with.
static CALLED: Mutex<Vec<[u32; 3]>> = Mutex::new(Vec::new());

fn handler(place: u32, r: &mut Regs) {
    CALLED.lock().unwrap().push([place, r.eax, r.ecx]);
    r.eax = r.eax.wrapping_mul(7).wrapping_add(place);
    r.ecx ^= 1 << place;
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let copy = unsafe {
        RedirectedCopy::new(
            game_addr(rva(0x490106)),
            LEN,
            &ENTRY_BYTES,
            &CALLS,
            &|callee| {
                STEPS
                    .iter()
                    .position(|&step| step == callee)
                    .map(|place| stand_in(place as u32))
            },
        )
    };
    let copy = match copy {
        Ok(copy) => copy,
        Err(why) => return t.skip(why),
    };
    for ecx in [0u32, 0xFFFF_FFFF, 0x1234_5678] {
        let run = |rust: bool| unsafe {
            CALLED.lock().unwrap().clear();
            set_handler(handler);
            let before = Regs {
                ecx,
                ..Regs::SENTINEL
            };
            let kept = if rust {
                draw_main_menu_objects(&before, &mut |n, r| {
                    act(n as u32, r);
                    Ok(())
                })
                .is_ok()
            } else {
                let mut r = before;
                call_game(&mut r, copy.addr());
                r.preserved_except(&before, 0)
            };
            (kept, CALLED.lock().unwrap().clone())
        };
        let (theirs, mine) = (run(false), run(true));
        t.check(theirs == mine && theirs.0 && theirs.1.len() == 7, || {
            format!("ecx {ecx:#x}: original {theirs:x?}; rust {mine:x?}")
        });
    }
    t.finish()
}
