//! `physics::crossed_edge` (0x45D737): when a probe point crossed a face's edge
//! (`dword_6D3B84` not -1): its three floats from `unk_71AAC8[n]` .. into `unk_71A1FC`
//! ..; `physics::restore_saved_car`; `physics::crossed_edge_contact`; `unk_71A208`
//! raised to |`flt_6D3108`|, `unk_71A20C` = 1, `unk_71A1CC` = 0x32;
//! `physics::feel_ground`.
//!
//! In:  nothing
//! Out: the registers as the calls leave them; no flags read (callers go on)
//!
//! Callers: `sub_43B635` (every physics step), `sub_43BDD1`.
//! Refuses (before anything) a probe point past 0x3F (the tables' room), and what its
//! calls refuse (what was written put back).
//! Self-test (`test_crossed`; in a race, its `feel_ground` needs the stage): the
//! original and the Rust version on the game's variables (saved and put back) with a
//! test face and scene points, paths across each edge, none crossed, under three
//! precisions.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::math::x87_run::{TEST_CONTROL_WORDS, abs_compare, abs_of, with_control_word};
use crate::physics::contact_force;
use crate::physics::crossed_edge_contact::{CROSSED, check_probe};
use crate::physics::flow::{a, g, guard, p};
use crate::physics::touch_face::FACES;
use crate::race::stage_prefs::left_of;
use crate::scene::matrix3::test_floats;
use crate::screens::harness::{CallOriginal, Left};
use crate::service::harness::{Region, Rust, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("physics::crossed_edge", rva(0x45D737), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The steps, not guarded (the probe checked before).
pub fn steps(r: &mut Regs) -> Result<(), Refused> {
    let n = g(CROSSED);
    if n == u32::MAX {
        return Ok(());
    }
    r.ebx = n;
    let k = n as usize;
    p(0x71A1FC, g(0x71AAC8 + 4 * k));
    p(0x71A200, g(0x71ABC8 + 4 * k));
    p(0x71A204, g(0x71ACC8 + 4 * k));
    unsafe { crate::physics::restore_saved_car::restore_saved_car() }?.apply(r);
    crate::physics::crossed_edge_contact::steps(r)?;
    if abs_compare(g(0x6D3108), g(0x71A208)) & 0x100 == 0 {
        p(0x71A208, abs_of(g(0x6D3108)));
    }
    p(0x71A20C, 1);
    p(0x71A1CC, 0x32);
    unsafe { crate::physics::feel_ground::feel_ground(r) }
}

/// The probe checked (-1 is none).
pub fn check() -> Result<(), Refused> {
    if g(CROSSED) == u32::MAX {
        return Ok(());
    }
    check_probe()
}

/// The Rust version.
pub unsafe fn crossed_edge(entry: &Regs) -> Result<Left, Refused> {
    check()?;
    guard(&regions(), || {
        let mut r = *entry;
        steps(&mut r)?;
        Ok(left_of(&r))
    })
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { crossed_edge(r) }
}

/// Everything the two and their calls write.
pub fn regions() -> Vec<Region> {
    let mut all = crate::physics::wall_contacts::regions();
    all.extend([(a(0x6D3180), 0x38), (a(0x620460), 8), (a(0x614A70), 0x18)]);
    all
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

/// The tests of the two (`hook`, its `call_original` and Rust version): a test face (its
/// corners scene points 0 .. 3 of a test square) and probe paths from inside across each
/// side or not; `crossed_edge` (whose `feel_ground` needs the stage) only in a race.
pub fn test_crossed(hook: &RawHook, call_original: CallOriginal, rust: Rust) -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(hook) else {
        return (0, 0);
    };
    let whole = std::ptr::eq(hook, &HOOK);
    if whole && (!crate::physics::move_slot::in_race() || !(1..=0x40).contains(&g(0x71A238))) {
        return t.skip("not in a race (no stage)");
    }
    let car = crate::physics::car_sounds::CURRENT;
    let sp = crate::physics::scene_point_hits_car::SCENE_POINTS;
    let mut all = regions();
    if !whole {
        all.extend(contact_force::inputs());
    }
    all.extend([
        (a(car), 4),
        (a(crate::physics::car_sounds::LISTED), 0x20),
        (a(FACES), 4),
        (a(sp), 4),
        (a(CROSSED), 4),
        (a(0x6D1E14), 0x1200), // the probe tables, 0x6D1E14 .. 0x6D3014
        (a(0x71AAC8), 0x300),
        (a(0x614148), 8),
    ]);
    let saved = capture(&all);
    let mut face = [0u32; 0x10];
    let mut points = [0u32; 3 * 4];
    let (fs, ps) = (face.as_mut_ptr() as u32, points.as_mut_ptr() as u32);
    all.extend([(fs, 0x40), (ps, 0x30)]);
    for word in TEST_CONTROL_WORDS {
        with_control_word(word, &mut || {
            for seed in 0..16u32 {
                // No crossing (-1) is for the whole; `crossed_edge_contact` refuses it
                // on purpose (the original would index its tables with it).
                if !whole && seed % 8 == 7 {
                    continue;
                }
                let set = || {
                    restore(&all[..all.len() - 2], &saved);
                    let mut f = vec![0u32; 0x100];
                    test_floats(seed * 31 + 5, &mut f);
                    if !whole {
                        contact_force::set_inputs(seed);
                        p(0x614148, 0);
                    }
                    let c = seed % 8;
                    p(car, c);
                    p(
                        crate::physics::car_sounds::LISTED + 4 * c as usize,
                        u32::MAX,
                    );
                    // The face: in a race one of the stage's (its real corners, as
                    // `feel_ground` reads the stage after); else a test square of
                    // corners p1 (-1, -1), p2 (1, -1), p3 (-1, 1), p4 (1, 1) (x 0x10000).
                    let number = if whole { 10 + seed * 7 } else { 0 };
                    if !whole {
                        p(FACES, fs);
                        p(sp, ps);
                        let corners = [(-1i32, -1i32), (1, -1), (-1, 1), (1, 1)];
                        for (j, (x, z)) in corners.iter().enumerate() {
                            unsafe {
                                ((fs + 4 + 4 * j as u32) as *mut u32).write(j as u32);
                                ((ps + 12 * j as u32) as *mut u32).write((x * 0x10000) as u32);
                                ((ps + 12 * j as u32 + 8) as *mut u32).write((z * 0x10000) as u32);
                            }
                        }
                    }
                    let at = (number << 6).wrapping_add(g(FACES));
                    let rd = |x: u32| unsafe { (x as *const u32).read_unaligned() };
                    let corner = |j: u32| {
                        let pt = rd(at + 4 + 4 * j).wrapping_mul(12).wrapping_add(g(sp));
                        (rd(pt) as i32 as i64, rd(pt + 8) as i32 as i64)
                    };
                    let (mut cx, mut cz, mut span) = (0i64, 0i64, 0i64);
                    for j in 0..4 {
                        let (x, z) = corner(j);
                        cx += x / 4;
                        cz += z / 4;
                        span = span
                            .max((x - corner(0).0).abs())
                            .max((z - corner(0).1).abs());
                    }
                    let n = 3 + seed % 4;
                    p(CROSSED, if seed % 8 == 7 { u32::MAX } else { n });
                    let k = n as usize;
                    // From near the middle out across a side (by seed) or not.
                    let out = [(0i64, -1i64), (1, 0), (0, 1), (-1, 0), (1, 1), (0, 0)];
                    let (ox, oz) = out[(seed % 6) as usize];
                    let jitter = |v: u32| i64::from(v as i32 % 0x1000);
                    p(0x6D2C14 + 4 * k, (cx + jitter(f[0])) as u32);
                    p(0x6D2E14 + 4 * k, (cz + jitter(f[1])) as u32);
                    p(0x6D1E14 + 4 * k, (cx + ox * span + jitter(f[2])) as u32);
                    p(0x6D2014 + 4 * k, (cz + oz * span + jitter(f[3])) as u32);
                    p(0x6D2F14 + 4 * k, number);
                    if !whole {
                        let small = |v: u32| (f32::from_bits(v) % 8.0).to_bits();
                        for j in 0..3 {
                            p(0x6D2214 + 0x100 * j + 4 * k, small(f[8 + j]));
                        }
                    }
                };
                check_state(
                    &mut t,
                    &format!("control word {word:#x}, seed {seed}"),
                    call_original,
                    Regs::SENTINEL,
                    &set,
                    &all,
                    rust,
                );
            }
        });
    }
    all.truncate(all.len() - 2);
    restore(&all, &saved);
    t.finish()
}

fn self_test() -> (usize, usize) {
    test_crossed(&HOOK, call_original, &rust)
}
