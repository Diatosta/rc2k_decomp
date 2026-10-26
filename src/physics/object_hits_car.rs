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

use crate::errors::Refused;
use crate::framework::{EAX, EBX, ECX, EDX, RawHook, Regs, SelfTest, hook_asm, rva};
use crate::math::x87_run::{TEST_CONTROL_WORDS, ext_to_int, int_80, with_control_word};
use crate::physics::flow::{a, g, p};
use crate::physics::object_turn_matrix::{self, object_turn_matrix};
use crate::physics::point_hits_car::{self, point_hits_car};
use crate::scene::matrix3::test_floats;
use crate::scene::rotate_point::rotate_point;
use crate::screens::harness::Left;
use crate::service::harness::{Region, capture, check_state, dispatch_state, restore, state_entry};
use rc2k_core::physics::contacts;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("physics::object_hits_car", rva(0x45E755), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

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

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { object_hits_car(r) }
}

pub fn regions() -> Vec<Region> {
    let mut all = object_turn_matrix::regions();
    all.extend(point_hits_car::regions());
    all.extend([(a(0x61410C), 4), (a(0x6D3B40), 12)]);
    all
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let mut all = regions();
    all.extend(point_hits_car::inputs());
    all.extend([
        (a(OBJECT), 4),
        (a(FIRST), 0x400),
        (a(COUNT), 0x400),
        (a(POINTS), 4),
        (a(0x635038), 16), // the object's scale and turns
    ]);
    let saved = capture(&all);
    let mut points = [0u32; 3 * 16];
    let mut object = [0u32; 0x10];
    let (pts, ob) = (points.as_mut_ptr() as u32, object.as_mut_ptr() as u32);
    all.extend([(pts, 0xC0), (ob, 0x40)]);
    for word in TEST_CONTROL_WORDS {
        with_control_word(word, &mut || {
            for seed in 0..18u32 {
                let set = || {
                    point_hits_car::set_inputs(seed, [10u32, 5, 3, 1, 6, 0][(seed % 6) as usize]);
                    // Its own calls' counter (`dword_61410C`): the same start for both sides.
                    p(0x61410C, seed);
                    let n = (seed * 37) % 0x100;
                    p(OBJECT, n);
                    p(FIRST + 4 * n as usize, seed % 4);
                    p(COUNT + 4 * n as usize, seed % 6);
                    p(POINTS, pts);
                    let mut f = [0u32; 0x36];
                    test_floats(seed * 5 + 2, &mut f);
                    for (k, v) in f.iter().enumerate().take(0x30) {
                        let v = (*v as i32).wrapping_rem(0x18000) as u32;
                        unsafe { ((pts + 4 * k as u32) as *mut u32).write(v) };
                    }
                    for k in 0..3 {
                        let o = (f[0x30 + k] as i32).wrapping_rem(0x8000) as u32;
                        unsafe { ((ob + 0x30 + 4 * k as u32) as *mut u32).write(o) };
                    }
                    p(0x635038, 16 + seed % 8);
                    p(0x63503C, f[0x33] & 0xFFF);
                    p(0x635040, if seed % 3 == 0 { 0 } else { f[0x34] & 0xFFF });
                    p(0x635044, f[0x35] & 0xFFF);
                };
                check_state(
                    &mut t,
                    &format!("control word {word:#x}, seed {seed}"),
                    call_original,
                    Regs {
                        esi: pts + 12,
                        ebp: ob,
                        ..Regs::SENTINEL
                    },
                    &set,
                    &all,
                    &rust,
                );
            }
        });
    }
    all.truncate(all.len() - 2);
    restore(&all, &saved);
    t.finish()
}
