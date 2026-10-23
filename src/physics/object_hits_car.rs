//! `physics::object_hits_car` (0x45E755): whether any point of the object hit
//! (`dword_6D321C`) lies inside the work car's box. `dword_61410C` + 1;
//! `physics::object_turn_matrix` (in `pusha` / `popa`); then for each of the object's
//! points (`g_objectFirstPoint[n]` on, `g_objectPointCount2[n]` of them, in
//! `g_objectPoints`, three whole numbers each): `fild`ed, turned by `flt_5EA444`
//! (`scene::rotate_point`) and `fistp`ed into the offset `dword_6D3B40` .., then
//! `physics::point_hits_car` (esi, ebp the caller's); the first inside ends it.
//!
//! In:  esi, ebp passed to `point_hits_car` (ebp the object's record)
//! Out: CF set when a point is inside; eax the point's number, ecx the points left (0 when
//!      none was inside), edx its address (kept with no points), ebx `dword_6D321C`; the
//!      rest kept
//!
//! Caller: `sub_45E3BF` (0x45E51B, `jb`).
//! Refuses (before anything) an object past 0xFF (the tables' room; `object_turn_matrix`
//! refuses it too).
//! Self-test: the original and the Rust version on the game's variables (saved and put
//! back): test points (`g_objectPoints` pointed at them for the test), the object's turn
//! and the car's figures by lot, counts 0 .. 5, under three precisions.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{EAX, EBX, ECX, EDX, Regs};
use crate::math::x87_run::{ext_to_int, int_80};
use crate::physics::flow::{a, g, p};
use crate::physics::object_turn_matrix::object_turn_matrix;
use crate::physics::point_hits_car::point_hits_car;
use crate::scene::rotate_point::rotate_point;
use crate::screens::harness::Left;
use rc2k_core::physics::contacts;

const OBJECT: usize = 0x6D321C; // dword_6D321C
const FIRST: usize = 0x631F70; // g_objectFirstPoint (0x100 dwords)
const COUNT: usize = 0x632370; // g_objectPointCount2 (0x100 dwords)
const POINTS: usize = 0x5C6FA4; // g_objectPoints (a pointer)
const TURN: usize = 0x5EA444; // flt_5EA444

/// The Rust version.
// Converted: `rc2k_core::physics::contacts::{check_object, walk_points}` (the turns stay
// x87).
pub unsafe fn object_hits_car(entry: &Regs) -> Result<Left, Refused> {
    let object = g(OBJECT);
    contacts::check_object(object)?;
    unsafe { object_turn_matrix(entry) }?;
    p(0x61410C, g(0x61410C).wrapping_add(1));
    let count = g(COUNT + 4 * object as usize);
    // With no points the original skips to `clc` before it sets edx (0x45E779); the
    // points' table is read only when there are some.
    let points = if count == 0 { 0 } else { g(POINTS) };
    let walk = contacts::walk_points(
        g(FIRST + 4 * object as usize),
        count,
        points,
        entry.edx,
        |edx| {
            let at =
                |k: u32| unsafe { int_80(((edx + 4 * k) as *const i32).read_unaligned(), false) };
            let turned = unsafe { rotate_point(a(TURN), [at(0), at(1), at(2)]) };
            for (k, e) in turned.iter().enumerate() {
                p(0x6D3B40 + 4 * k, ext_to_int(*e) as u32);
            }
            unsafe { point_hits_car(entry.esi, entry.ebp) }
        },
    );
    Ok(Left::carry(walk.inside)
        .with(EAX, walk.number)
        .with(EBX, object)
        .with(ECX, walk.left)
        .with(EDX, walk.address))
}
