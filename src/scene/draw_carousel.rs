//! `scene::draw_carousel` (0x490649): the entries of the main menu's carousel that
//! can be seen.
//!
//! In:  ecx, passed on for a carousel of one entry
//! Out: ebx = the last entry drawn, ecx = its place in the row (for one entry: ebx = 0,
//!      edx = 0, ecx as it came); nothing changed but ebx for a carousel of none
//!
//! With one entry (`g_carouselCount` = 1): entry 0, the place whatever ecx holds.
//! With none or fewer: nothing. Else the entry before the front one
//! (`g_carouselFront`, going round) at place -1, the front one at 0, the next at 1,
//! and while the carousel turns (`g_carouselAngle` not 0) the one after at 2; each
//! through `scene::draw_carousel_entry`.
//! One caller, `scene::draw_main_menu_carousel`.
//! Shadow mode runs the original alone (each entry is compared in its own hook).
//! Self-test: a redirected copy whose calls note the entry and its place.

use crate::errors::Refused;
use crate::framework::{
    RawHook, Regs, SelfTest, call_game, game_addr, game_read, game_write, hook_asm, rva, shadow,
};
use crate::install::stand_ins::{act, set_handler, stand_in};
use crate::redirect::RedirectedCopy;
use crate::scene::draw_carousel_entry::draw_carousel_entry;
use crate::scene::draw_menu_object::draw_menu_object_in_game;
use crate::service::harness::{at, capture, restore};
use std::sync::Mutex;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("scene::draw_carousel", rva(0x490649), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

const COUNT_RVA: usize = rva(0x75AA78); // g_carouselCount
const FRONT_RVA: usize = rva(0x75AA7C); // g_carouselFront
const ANGLE_RVA: usize = rva(0x75AA80); // g_carouselAngle

// ---- Rust version ----

/// `entry`: what stands for `scene::draw_carousel_entry`, which keeps the registers.
pub unsafe fn draw_carousel(
    r: &mut Regs,
    entry: &mut dyn FnMut(&Regs) -> Result<(), Refused>,
) -> Result<(), Refused> {
    unsafe {
        let count: u32 = game_read(COUNT_RVA);
        r.ebx = count;
        if count == 1 {
            r.ebx = 0;
            r.edx = 0;
            return entry(r);
        }
        if (count.wrapping_sub(1) as i32) < 0 {
            return Ok(());
        }
        r.ebx = game_read(FRONT_RVA);
        if r.ebx == 0 {
            r.ebx = count;
        }
        r.ebx = r.ebx.wrapping_sub(1);
        r.ecx = u32::MAX;
        entry(r)?;
        let rows = if game_read::<u32>(ANGLE_RVA) != 0 {
            3
        } else {
            2
        };
        for row in 0..rows {
            r.ebx = r.ebx.wrapping_add(1);
            if r.ebx == count {
                r.ebx = 0;
            }
            r.ecx = row;
            entry(r)?;
        }
        Ok(())
    }
}

/// With the Rust versions of what it calls.
pub unsafe fn draw_carousel_in_game(r: &mut Regs) -> Result<(), Refused> {
    unsafe {
        draw_carousel(r, &mut |r| {
            draw_carousel_entry(r, &mut |r| draw_menu_object_in_game(r))
        })
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
    if let Err(e) = unsafe { draw_carousel_in_game(r) } {
        *r = entry;
        unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) }
    }
}

// ---- self-test ----

const LEN: usize = 0x8C;
const ENTRY_BYTES: [u8; 6] = [0x8B, 0x1D, 0x78, 0xAA, 0x75, 0x00];
const CALLS: [usize; 5] = [0x24, 0x3C, 0x54, 0x75, 0x85];

/// Each call: ebx, ecx, edx.
static CALLED: Mutex<Vec<[u32; 3]>> = Mutex::new(Vec::new());

fn handler(_: u32, r: &mut Regs) {
    CALLED.lock().unwrap().push([r.ebx, r.ecx, r.edx]);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let copy = unsafe {
        RedirectedCopy::new(
            game_addr(rva(0x490649)),
            LEN,
            &ENTRY_BYTES,
            &CALLS,
            &|callee| (callee == rva(0x4906D5)).then(|| stand_in(0)),
        )
    };
    let copy = match copy {
        Ok(copy) => copy,
        Err(why) => return t.skip(why),
    };
    let inputs = [at(COUNT_RVA), at(FRONT_RVA), at(ANGLE_RVA)];
    let saved = capture(&inputs);
    for count in [0u32, 1, 2, 3, 5, 7, 0xFFFF_FFFF, 0x8000_0000] {
        for front in [0u32, 1, 2, 4, 6, 9] {
            for angle in [0u32, 1, 0xFFFF_0000] {
                let run = |rust: bool| unsafe {
                    game_write(COUNT_RVA, count);
                    game_write(FRONT_RVA, front);
                    game_write(ANGLE_RVA, angle);
                    CALLED.lock().unwrap().clear();
                    set_handler(handler);
                    let mut r = Regs::SENTINEL;
                    if rust {
                        let done = draw_carousel(&mut r, &mut |r| {
                            act(0, &mut { *r });
                            Ok(())
                        });
                        if done.is_err() {
                            r = Regs::default();
                        }
                    } else {
                        call_game(&mut r, copy.addr());
                    }
                    (
                        [r.eax, r.ebx, r.ecx, r.edx, r.esi, r.edi, r.ebp],
                        CALLED.lock().unwrap().clone(),
                    )
                };
                let (theirs, mine) = (run(false), run(true));
                t.check(theirs == mine, || {
                    format!(
                        "{count:#x} entries, front {front}, angle {angle:#x}: original {theirs:x?}; rust {mine:x?}"
                    )
                });
            }
        }
    }
    restore(&inputs, &saved);
    t.finish()
}
