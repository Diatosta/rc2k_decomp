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
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::{Regs, game_addr, rva};
use crate::race::frame_draw::*;
use crate::race::start_kept_reflections::kept_regions;
use crate::service::harness::{Region, at};

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
