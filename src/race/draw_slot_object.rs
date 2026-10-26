//! `race::draw_slot_object` (0x4626D1): a race slot's object drawn (see
//! `race/frame_draw.rs`): its number to `g_objectNumber`; outside one player and modes 0
//! and 2 only the drawn view's car; the light set, the slot's turn and place (+0x10, +4 ..
//! with +0x5C on y) moved by the view into `g_madeObjectMatrix`, then
//! `scene::draw_scene_object` with the reflections kept around it.
//!
//! In:  esi = the slot (0x78 bytes of `g_raceSlots`)
//! Out: the registers as the drawing leaves them (its caller calls it inside `pusha`)
//!
//! In the game the drawing is `scene::draw_scene_object_in_game`. Refuses (before
//! anything) a car's object without its block. Shadow mode runs the original alone (it
//! draws). Self-test: a copy whose drawing call goes to a stand-in that notes eax .. ecx,
//! against the Rust version on the same; test slots (objects, places, turns), players,
//! modes and views; all put back.

use crate::framework::{
    RawHook, Regs, SelfTest, call_game, game_addr, game_write, hook_asm, rva, shadow,
};
use crate::race::frame_draw::*;
use crate::race::level_init::{note, take_called};
use crate::race::start_kept_reflections::kept_regions;
use crate::service::harness::{Region, at, capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

const CALLS: [usize; 7] = [0xE6, 0xFD, 0x114, 0x12B, 0x130, 0x135, 0x13A];

pub static HOOK: RawHook =
    RawHook::new("race::draw_slot_object", rva(0x4626D1), thunk, &ORIGINAL).self_test(self_test);

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
    match unsafe { draw_slot_object(r, &mut slot_part) } {
        Ok(after) => {
            let esp = r.esp;
            *r = after;
            r.esp = esp;
        }
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    {
        let after = unsafe { draw_slot_object(r, &mut slot_part) }?;
        let esp = r.esp;
        *r = after;
        r.esp = esp;
    }
    Ok(())
}

pub fn slot_regions() -> Vec<Region> {
    let g = |a: usize, len: usize| (game_addr(rva(a)) as u32, len);
    let mut v = kept_regions();
    v.extend([
        g(0x5EA444, 0x40),
        g(0x5EA484, 0x40),
        g(0x5EA04C, 0x40),
        g(0x5EA10C, 0x40),
        g(0x5EA30C, 0x40),
        g(0x63501C, 0xC),
        at(rva(0x6214FC)),
        at(rva(0x635054)),
        g(0x635084, 0x18),
        at(rva(0x6365C8)),
        at(rva(0x62DAF4)),
        at(rva(0x62400C)),
    ]);
    v
}

/// The inputs a slot test sets.
pub fn slot_inputs() -> Vec<Region> {
    let g = |a: usize, len: usize| (game_addr(rva(a)) as u32, len);
    vec![
        at(rva(0x63DB94)),
        at(rva(0x62389C)),
        at(rva(0x63DB9C)),
        g(0x641680, 0x20),
        g(0x5EA1CC, 0x40),
        g(0x5EA24C, 0x40),
    ]
}

/// A test slot: object `object`, a place and a turn by `seed`.
pub fn fill_slot(slot: &mut [u32; 0x1E], object: u32, seed: &mut u32) {
    slot[0] = object;
    slot[1] = 0x10_0000 ^ (*seed % 0x1000);
    slot[2] = 0x2_0000;
    slot[3] = 0x18_0000u32.wrapping_sub(*seed % 0x3000);
    let turn = crate::scene::matrix::test_floats(seed);
    slot[4..20].copy_from_slice(&turn);
    slot[0x17] = 0x800;
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    crate::install::stand_ins::set_handler(note);
    let copy = match crate::audio::test_copy::make(
        &HOOK,
        &crate::audio::test_copy::Copy {
            address: 0x4626D1,
            len: 0x140,
            entry: &[0x8B, 0x06, 0xA3, 0xFC, 0x14, 0x62, 0x00, 0x83],
            calls: &CALLS,
            jumps_out: &[],
        },
        &|c| {
            SLOT_CALLEES
                .iter()
                .find(|&&(_, k)| rva(k) == c)
                .map(|&(p, _)| crate::install::stand_ins::stand_in(p))
        },
        &[],
        0,
    ) {
        Ok(c) => c,
        Err(e) => return t.skip(&format!("copy: {e}")),
    };
    let regions = slot_regions();
    let mut kept = regions.clone();
    kept.extend(slot_inputs());
    let saved = capture(&kept);
    let mut seed = 0x5107_0001u32;
    let mut slot = Box::new([0u32; 0x1E]);
    for (object, players, mode, view, shown) in [
        (3u32, 1u32, 1u32, 0u32, 0u32),
        (0x10, 2, 0, 1, 0),
        (0x30, 2, 1, 1, 1),
        (0x30, 2, 1, 0, 1),
        (0x20, 2, 3, 1, 0),
    ] {
        fill_slot(&mut slot, object, &mut seed);
        let at_slot = slot.as_ptr() as u32;
        let prepare = || unsafe {
            restore(&kept, &saved);
            game_write(rva(0x63DB94), players);
            game_write(rva(0x62389C), mode);
            game_write(rva(0x63DB9C), view);
            for k in 0..8u32 {
                game_write(
                    rva(0x641680 + 4 * k as usize),
                    if k == 3 { shown } else { 7 },
                );
            }
            take_called();
        };
        let regs = |r: &Regs| [r.eax, r.ebx, r.ecx, r.edx, r.esi, r.edi, r.ebp];
        prepare();
        let mut ro = Regs {
            esi: at_slot,
            ..Regs::SENTINEL
        };
        unsafe { call_game(&mut ro, copy.addr()) };
        let theirs = (Some(regs(&ro)), take_called(), capture(&regions));
        prepare();
        let mine = unsafe {
            draw_slot_object(
                &Regs {
                    esi: at_slot,
                    ..Regs::SENTINEL
                },
                &mut |p, r| {
                    note(p, r);
                    Ok(())
                },
            )
        };
        let mine = (
            mine.as_ref().ok().map(regs),
            take_called(),
            capture(&regions),
        );
        t.check(theirs == mine, || {
            let diff = theirs.2.iter().zip(&mine.2).position(|(a, b)| a != b);
            format!(
                "object {object:#x} players {players} mode {mode} view {view} shown {shown}: original {:x?} {:x?}; rust {:x?} {:x?}; memory first differs at {diff:x?}",
                theirs.0, theirs.1, mine.0, mine.1
            )
        });
    }
    restore(&kept, &saved);
    t.finish()
}
