//! `drive::drive_computer_car` (0x462811): the computer's driving of the work car, in
//! `pusha` / `popa`. Its controls (0x71BF88 .. 0x71BFA0: steering, throttle 0x100, brake
//! 0xFF / 0x100, and three switches) start at 0; nothing more below a step count
//! (`dword_719FEC`) of 800 or with `flt_71A0A0` below `flt_71BF70`. Else, by the
//! checkpoint the car is at (`dword_71A070`, n) and its speed (`fistp` of the root of
//! `flt_71A170`² + `flt_71A178`², doubled, `dword_71C00C`):
//! - the looked-at checkpoint's face's two bytes (+0xA, +0xB) and the line's turn ahead
//!   (`dword_9ED808[n + k]`, k by the speed and `dword_71A8F8`: the difference of two of
//!   them >> 16) note a bend (0x71C010, 0x71C038, 0x71C088 by slot `dword_71C0CC`, 0);
//! - a wanted speed from the tables at 0x71BF1C / 0x71BF38 / 0x71BF54 by the bend's turn
//!   (n's, `>> 25`), 25 / 40 / 50 near a bend, eased by `flt_71BFA4` above 50; throttle or
//!   brake by the speed against it;
//! - the point aimed at: a checkpoint face's first point (n + 2 .. n + 5 by the speed and
//!   the bend) moved sideways by `flt_71BE9C[(n + slot) & 0x1F]` along the line's
//!   normal (`scene::cross_product` with `flt_71BF74`, `race::normalise_work_vector`);
//! - the steering: the car's forward (row 0's and row 2's z of `g_crashMatrix`) crossed
//!   with the way to that point, its y times 256 `fist`'d, its sign by the product's
//!   (and turned round in reverse) into `dword_71BF88`;
//! - a reverse count (`dword_71BFD0`), and every 200 steps (`dword_71BFA8`) a check that
//!   the car moved 0x8000 (else reversing for 250 steps, or the reverse tries counted
//!   in `dword_71C060`, the fourth setting `dword_71BFA0`).
//!
//! The checkpoint tables' indices are held to `g_checkpointCount` - 1 (unsigned) as the
//! original holds them. What each value is, is a reading at best.
//!
//! In:  nothing
//! Out: every register kept; no flags read (callers go on with `push`)
//!
//! Callers: `sub_43A35A` (0x43A497, 0x43A4F7).
//! Refuses (before anything) no checkpoints when the tables are to be read.
//! Self-test: in a race only: a copy of the original (its calls the game's) against the
//! Rust version on the game's stage, with the work car's checkpoint, speed, gear, step
//! count and the slot's counters by lot; everything it writes saved and put back.

use crate::errors::{BadInput, Refused};
use crate::framework::{
    RawHook, Regs, SelfTest, call_game, game_addr, game_read, game_write, hook_asm, rva,
};
use crate::math::x87;
use crate::math::x87_run::{
    compare_full, float_hypot_to_int, int_toward_to_int, mul2_to_int, times_fist_compare,
};
use crate::race::norms::normalise_work_vector;
use crate::scene::view_matrix::cross_product;
use crate::screens::harness::Left;
use crate::service::harness::{Region, capture, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("drive::drive_computer_car", rva(0x462811), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn g(x: u32) -> u32 {
    unsafe { game_read(rva(x as usize)) }
}

fn p(x: u32, v: u32) {
    unsafe { game_write(rva(x as usize), v) }
}

fn a(x: u32) -> u32 {
    game_addr(rva(x as usize)) as u32
}

unsafe fn at(addr: u32) -> u32 {
    unsafe { (addr as *const u32).read_unaligned() }
}

unsafe fn put(addr: u32, v: u32) {
    unsafe { (addr as *mut u32).write_unaligned(v) }
}

const CHECKPOINT_FACES: u32 = 0x9E3738; // g_checkpointFaces
const LINE: u32 = 0x9ED808; // dword_9ED808, by checkpoint
const COUNT: u32 = 0x623640; // g_checkpointCount
const GEAR: u32 = 0x71A1AC; // g_carGear
const SPEED: u32 = 0x71C00C;
const SLOT: u32 = 0x71C0CC;

/// `table[n + k]`, the index held to the count less one (unsigned) as the original holds
/// it (through `lea`, `sub` and `shr 2`).
fn held(table: u32, n: u32, k: u32) -> u32 {
    let mut i = (n.wrapping_add(k).wrapping_mul(4)) >> 2;
    let count = g(COUNT);
    if i >= count {
        i = count.wrapping_sub(1);
    }
    g(table.wrapping_add(i.wrapping_mul(4)))
}

/// A checkpoint face's first point: (x, z).
unsafe fn face_point(face: u32) -> (u32, u32) {
    unsafe {
        let f = g(0x5CCCBC).wrapping_add(face << 6);
        let pt = g(0x5CCCC0).wrapping_add(at(f + 4).wrapping_mul(12));
        (at(pt), at(pt + 8))
    }
}

/// `sub_463296` then `sub_4632AE`: (to - from) in x and z, y 0, as floats, at `out`.
fn flat_way(from: u32, to: u32, out: u32) {
    unsafe {
        put(out, at(to).wrapping_sub(at(from)));
        put(out + 8, at(to + 8).wrapping_sub(at(from + 8)));
        put(out + 4, 0);
        for o in [0, 4, 8] {
            put(out + o, x87::from_int(at(out + o) as i32));
        }
    }
}

/// Through `g_workVector` and `race::normalise_work_vector`: the vector at `from` made one
/// long into `to`.
unsafe fn normalised(from: u32, to: u32) {
    unsafe {
        let w = a(0x5C5F60);
        for o in [0, 4, 8] {
            put(w + o, at(from + o));
        }
        normalise_work_vector();
        for o in [0, 4, 8] {
            put(to + o, at(w + o));
        }
    }
}

/// The Rust version.
pub unsafe fn drive_computer_car() -> Result<(), Refused> {
    unsafe {
        let reaches = g(0x719FEC) >= 0x320 && compare_full(g(0x71A0A0), g(0x71BF70)) & 0x100 == 0;
        if reaches && g(COUNT) == 0 {
            return Err(BadInput("no checkpoints (the tables' indices would be -1)").into());
        }
        for v in [
            0x71BF88, 0x71BF8C, 0x71BF90, 0x71BF94, 0x71BF98, 0x71BF9C, 0x71C0D0, 0x71BFA0,
            0x71C0E4, 0x71C0BC, SLOT,
        ] {
            p(v, 0);
        }
        if !reaches {
            return Ok(());
        }
        let slot = g(SLOT);
        p(0x71C0DC, a(0x71C0E8).wrapping_add(slot << 4));
        p(0x71C0E0, a(0x71C188).wrapping_add(slot << 4));
        let n = g(0x71A070);
        let f = g(0x5CCCBC).wrapping_add(held(CHECKPOINT_FACES, n, 2) << 6);
        let (b1, b2) = (
            u32::from((f as *const u8).add(0xA).read()),
            u32::from((f as *const u8).add(0xB).read()),
        );
        p(0x71C0D4, b1);
        p(0x71C0D8, b2);
        if (b1.wrapping_add(b2).wrapping_sub(0x12) as i32) < 0 {
            p(0x71C0BC, 1);
            p(0x71C0D0, 1);
        }
        p(0x71C0D0, 1);
        let speed = (float_hypot_to_int(g(0x71A170), g(0x71A178)) as u32) << 1;
        p(SPEED, speed);
        let edx = g(SLOT);
        let (first, second, ahead) = if g(0x71C0D0) == 1 {
            if g(0x71A8F8) >= 3 {
                if speed >= 0x46 {
                    (0xF, 0x10, 0xF)
                } else {
                    (3, 4, 3)
                }
            } else if speed >= 0x5A {
                (0xE, 0xF, 0xE)
            } else {
                (4, 5, 4)
            }
        } else if speed >= 0x6E {
            (0xF, 0x10, 0xF)
        } else {
            (4, 5, 4)
        };
        let turn =
            (held(LINE, n, second).wrapping_sub(held(LINE, n, first)) as i32 >> 16).unsigned_abs();
        p(0x71BFFC, turn);
        let limit = if g(0x71C0D0) == 1 { 0x780 } else { 0x980 };
        if (turn.wrapping_sub(limit) as i32) >= 0 {
            if (turn.wrapping_sub(0xC00) as i32) >= 0 {
                p(0x71C038 + 4 * g(SLOT), 1);
            }
            let to = n.wrapping_add(ahead);
            if (turn.wrapping_sub(g(0x71C088 + 4 * edx)) as i32) >= 0 {
                p(0x71C088 + 4 * edx, turn);
                p(0x71C010 + 4 * edx, to);
            }
        }
        let edx = g(SLOT);
        if (n.wrapping_sub(g(0x71C010 + 4 * edx)) as i32) >= 0 {
            p(0x71C010 + 4 * edx, 0);
            p(0x71C088 + 4 * edx, 0);
        }
        let mut bend =
            ((held(LINE, n, 4).wrapping_sub(held(LINE, n, 0)) as i32) >> 25).unsigned_abs() + 10;
        if bend >= 0x18 {
            bend = 0x18;
        }
        if bend >= 0x10 {
            p(0x71C0E4, 1);
        }
        let mut table = 0x71BF1C;
        if g(0x71C0D0) == 1 {
            table = 0x71BF38;
        }
        if g(0x71A8F8) >= 3 {
            table = 0x71BF54;
        }
        let mut want = u32::from((a(table) as *const u8).add(bend as usize).read());
        let s = g(SLOT);
        if g(0x71C010 + 4 * s) != 0 {
            want = if g(0x71C038 + 4 * s) != 0 {
                0x19
            } else if (speed.wrapping_sub(0x28) as i32) < 0 {
                0x32
            } else {
                0x28
            };
        } else {
            p(0x71C038 + 4 * s, 0);
        }
        p(0x71C008, want);
        if want >= 0x32 {
            p(
                0x71C008,
                int_toward_to_int(want as i32, g(0x604EC8), g(0x71BFA4)) as u32,
            );
        }
        let want = g(0x71C008);
        if speed < want {
            p(0x71BF8C, 0x100);
        } else if speed >= want.wrapping_add(5) {
            if speed >= want.wrapping_add(0xF) {
                p(0x71BF94, 1);
            }
            p(0x71BF90, 0xFF);
        }
        let k = n.wrapping_add(g(SLOT)) & 0x1F;
        p(0x71C0C8, x87::loaded(g(0x71BE9C + 4 * k)));
        let (x, z) = face_point(held(CHECKPOINT_FACES, n, 0));
        p(0x71C228, x);
        p(0x71C230, z);
        p(0x71C22C, 0);
        let aim = if g(0x71C038 + 4 * g(SLOT)) != 0 {
            if speed >= 0x46 {
                5
            } else if speed >= 0x28 {
                4
            } else {
                2
            }
        } else {
            let mut k = 2;
            if g(0x71C0E4) == 1 && speed >= 0x46 {
                k = 4;
            }
            if speed >= 0x5A {
                k = 5;
            } else if speed >= 0x3C {
                k = 3;
            }
            k
        };
        let (x, z) = face_point(held(CHECKPOINT_FACES, n, aim));
        p(0x71C238, x);
        p(0x71C240, z);
        p(0x71C23C, 0);
        flat_way(a(0x71C228), a(0x71C238), a(0x71C298));
        normalised(a(0x71C298), a(0x71C2A8));
        cross_product(
            a(0x71C2A8) as usize,
            a(0x71BF74) as usize,
            a(0x71C278) as usize,
        );
        normalised(a(0x71C278), a(0x71C278));
        let (k1, k2) = (g(0x71C0C8), g(0x5C5D98));
        p(0x71C0C0, mul2_to_int(g(0x71C278), k1, k2) as u32);
        p(0x71C0C4, mul2_to_int(g(0x71C280), k1, k2) as u32);
        p(0x71C238, g(0x71C238).wrapping_add(g(0x71C0C0)));
        p(0x71C240, g(0x71C240).wrapping_add(g(0x71C0C4)));
        let here = g(0x71C0DC);
        put(here, g(0x71A10C));
        put(here + 8, g(0x71A114));
        put(here + 4, 0);
        flat_way(here, a(0x71C238), a(0x71C258));
        normalised(a(0x71C258), a(0x71C268));
        let m = a(0x71A08C);
        p(0x71C2C0, x87::loaded(at(m + 0x28)));
        p(0x71C2B8, x87::negated(at(m + 8)));
        p(0x71C2BC, 0);
        cross_product(
            a(0x71C2B8) as usize,
            a(0x71C268) as usize,
            a(0x71C2C8) as usize,
        );
        let (i, status) = times_fist_compare(g(0x71C2CC), g(0x5C5E2C), g(0x5C5D90));
        p(0x71C000, i as u32);
        let mut steer = sign_abs(i as u32);
        if status & 0x100 != 0 {
            steer = steer.wrapping_neg();
        }
        if g(GEAR) == u32::MAX {
            steer = steer.wrapping_neg();
        }
        p(0x71BF88, steer);
        let s = g(SLOT);
        let back = g(0x71BFD0 + 4 * s);
        if back != 0 {
            p(0x71BFD0 + 4 * s, back - 1);
            if g(GEAR) != u32::MAX {
                p(0x71BF9C, 1);
                p(0x71BF90, 0x100);
            }
        } else if (g(GEAR).wrapping_sub(1) as i32) < 0 {
            p(0x71BF98, 1);
        }
        let s = g(SLOT);
        let steps = g(0x71BFA8 + 4 * s).wrapping_add(1);
        p(0x71BFA8 + 4 * s, steps);
        if (steps.wrapping_sub(0xC8) as i32) < 0 {
            return Ok(());
        }
        p(0x71BFA8 + 4 * s, 0);
        let (now, then) = (g(0x71C0DC), g(0x71C0E0));
        let (ax, az, cx, cz) = (at(now), at(now + 8), at(then), at(then + 8));
        put(then, ax);
        put(then + 8, az);
        let mut dx = ax.wrapping_sub(cx);
        if (dx as i32) < 0 {
            dx = dx.wrapping_neg();
        }
        let mut dz = az.wrapping_sub(cz);
        if (dz as i32) < 0 {
            dz = dz.wrapping_neg();
        }
        if (dx.wrapping_sub(dz) as i32) < 0 {
            dz >>= 1;
        } else {
            dx >>= 1;
        }
        let moved = dx.wrapping_add(dz);
        p(0x71BFF8, moved);
        if (moved.wrapping_sub(0x8000) as i32) >= 0 {
            return Ok(());
        }
        if g(GEAR) != u32::MAX {
            p(0x71BFD0 + 4 * g(SLOT), 0xFA);
            return Ok(());
        }
        let s = g(SLOT);
        p(0x71BFD0 + 4 * s, 0);
        let mut tries = g(0x71C060 + 4 * s).wrapping_add(1);
        if tries >= 4 {
            tries = 0;
            p(0x71BFA0, 1);
        }
        p(0x71C060 + 4 * s, tries);
        Ok(())
    }
}

/// `cmp 0; jns; neg`.
fn sign_abs(v: u32) -> u32 {
    if (v as i32) < 0 { v.wrapping_neg() } else { v }
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { drive_computer_car() }?;
    Ok(Left::nothing())
}

/// Everything it writes.
pub fn regions() -> Vec<Region> {
    vec![
        (a(0x71BF88), 0x71C2D4 - 0x71BF88),
        (a(0x5C5F60), 12),
        (a(0x62400C), 4),
    ]
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    if !crate::physics::move_slot::in_race() || g(COUNT) < 0x20 {
        return t.skip("not in a race (no stage's checkpoints)");
    }
    let copy = match crate::audio::test_copy::make(
        &HOOK,
        &crate::audio::test_copy::Copy {
            address: 0x462811,
            len: 0xA85,
            entry: &[0x60, 0x33, 0xC0, 0xA3, 0x88, 0xBF, 0x71, 0x00],
            calls: &[
                0x78F, 0x794, 0x7B9, 0x7F0, 0x815, 0x8B4, 0x8B9, 0x8DE, 0x935,
            ],
            jumps_out: &[],
        },
        &|_| None,
        &[],
        0,
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    let inputs: Vec<u32> = vec![
        0x719FEC, 0x71A0A0, 0x71A070, 0x71A170, 0x71A178, 0x71A8F8, GEAR,
    ];
    let mut all = regions();
    all.extend(inputs.iter().map(|&v| (a(v), 4)));
    let saved = capture(&all);
    let count = g(COUNT);
    for seed in 1..=20u32 {
        let mut x = seed.wrapping_mul(0x9E37_79B9) | 1;
        let mut next = move || {
            x = x.wrapping_mul(1_103_515_245).wrapping_add(12345);
            x >> 8
        };
        p(
            0x719FEC,
            if seed % 7 == 0 {
                0x100
            } else {
                0x400 + next() % 0x400
            },
        );
        p(0x71A070, next() % count);
        p(0x71A170, ((next() % 80) as f32 - 40.0).to_bits());
        p(0x71A178, ((next() % 80) as f32 - 40.0).to_bits());
        p(0x71A8F8, next() % 5);
        p(GEAR, [u32::MAX, 0, 1, 3][(next() % 4) as usize]);
        p(0x71BFA8, if seed % 3 == 0 { 0xC7 } else { next() % 0x40 });
        p(0x71BFD0, if seed % 4 == 0 { 3 } else { 0 });
        let input = capture(&all);
        let mut ro = Regs::SENTINEL;
        unsafe { call_game(&mut ro, copy.addr()) };
        let theirs = capture(&all);
        restore(&all, &input);
        let mine = unsafe { drive_computer_car() };
        let state = capture(&all);
        restore(&all, &saved);
        let regs_ok = mine.is_ok() && ro.preserved_except(&Regs::SENTINEL, 0);
        t.check(regs_ok && theirs == state, || {
            let first = theirs.iter().zip(&state).position(|(a, b)| a != b);
            format!("seed {seed}: regs_ok={regs_ok} {mine:?}, first difference at byte {first:?}")
        });
    }
    drop(copy);
    t.finish()
}
