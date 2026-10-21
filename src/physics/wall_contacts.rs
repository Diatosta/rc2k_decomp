//! `physics::wall_contacts` (0x45D7B6): the probe points (`unk_71A238` of them) against
//! walls. For each point k, its face now (`unk_71ADC8[k]`, into `dword_6D3174`) and before
//! (`dword_6D2814[k]`, into `dword_6D3170` when they differ): their words +2 or-ed
//! together; when they differ and the pair's has both 0x400 and 0x800, the wall met:
//! `dword_6D3214` = k, the point's three floats from `unk_71AAC8[k]` .. into
//! `unk_71A1FC` .., `physics::wall_edge_contact`, `unk_71A208` raised to |`flt_6D3108`|,
//! `unk_71A20C` = 1, `unk_71A1CC` = 0x32, `physics::restore_saved_car`,
//! `physics::feel_ground`. None met: `dword_6D3174` = `dword_6D3170`, the faces' words
//! or-ed & 0xC00 into `dword_6D3038`; at 0xC00, the car-frame speed (`unk_71A17C` ..)
//! into `unk_71A1FC` .., `physics::wall_push_back`, the same two words,
//! `restore_saved_car`, `feel_ground`.
//!
//! In:  nothing
//! Out: the registers as the steps and calls leave them (ebp the point, or -1 after the
//!      loop); no flags read (callers go on)
//!
//! Callers: `sub_43B635` (every physics step), `sub_43BDD1`.
//! Refuses (before anything) a count of 0 or past 0x40 (the original's loop runs on, or
//! past the tables), and what its calls refuse (what was written put back).
//! Self-test (`test_walls`, in a race; skipped otherwise): the original and the Rust
//! version on the stage's own faces (everything written saved and put back): probe
//! lists of faces chosen by their wall bits, none met, a wall met, the 0xC00 fallback.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::math::x87_run::{TEST_CONTROL_WORDS, with_control_word};
use crate::physics::contact_force;
use crate::physics::flow::{a, g, guard, p};
use crate::physics::touch_face::FACES;
use crate::race::norms::norm_regions;
use crate::race::stage_prefs::left_of;
use crate::scene::matrix3::test_floats;
use crate::screens::harness::{CallOriginal, Left};
use crate::service::harness::{Region, Rust, capture, check_state, dispatch_state, restore};
use rc2k_core::physics::walls;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("physics::wall_contacts", rva(0x45D7B6), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// `g_workVector`.
pub const WORK: usize = 0x5C5F60;
/// Contact record 4 (`g_contactRecords` + 0x80).
pub const WALL_RECORD: usize = 0x614AD0;

fn word(face: u32) -> u32 {
    let at = (face << 6).wrapping_add(g(FACES)) + 2;
    unsafe { (at as *const u16).read_unaligned() as u32 }
}

fn after(r: &mut Regs) -> Result<(), Refused> {
    p(0x71A20C, 1);
    p(0x71A1CC, 0x32);
    unsafe { crate::physics::restore_saved_car::restore_saved_car() }?.apply(r);
    unsafe { crate::physics::feel_ground::feel_ground(r) }
}

/// The steps, not guarded (`check` before).
// Converted: `rc2k_core::physics::walls::walk`.
pub fn steps(r: &mut Regs) -> Result<(), Refused> {
    r.edx = 0;
    r.ebp = 0;
    let walked = walls::walk(g(0x71A238), |k| {
        r.ebp = k;
        let k = k as usize;
        r.ebx = g(0x71ADC8 + 4 * k);
        p(0x6D3174, r.ebx);
        r.esi = (r.ebx << 6).wrapping_add(g(FACES));
        let now_word = word(r.ebx);
        r.eax = now_word;
        let before = g(0x6D2814 + 4 * k);
        let before_word = (r.ebx != before).then(|| {
            p(0x6D3170, before);
            r.esi = (before << 6).wrapping_add(g(FACES));
            r.ebx = word(before);
            r.eax = (now_word | r.ebx) & 0xC00;
            r.ebx
        });
        Ok::<_, Refused>(walls::ProbeFaces {
            now_word,
            before_word,
        })
    })?;
    match walked {
        walls::Walk::Met { probe, words } => {
            // 0x45D83E: a wall met.
            r.edx = words;
            let k = probe as usize;
            p(0x6D3214, r.ebp);
            p(0x71A1FC, g(0x71AAC8 + 4 * k));
            p(0x71A200, g(0x71ABC8 + 4 * k));
            p(0x71A204, g(0x71ACC8 + 4 * k));
            crate::physics::wall_edge_contact::steps(r)?;
            if let Some(hit) = rc2k_core::physics::walls::raised_hit(
                f32::from_bits(g(0x6D3108)),
                f32::from_bits(g(0x71A208)),
            ) {
                p(0x71A208, hit.to_bits());
            }
            after(r)
        }
        walls::Walk::Clear { bits } => {
            r.ebp = u32::MAX;
            r.eax = g(0x6D3170);
            p(0x6D3174, r.eax);
            r.edx = bits;
            p(0x6D3038, r.edx);
            if r.edx != walls::WALL_BITS {
                return Ok(());
            }
            p(0x71A1FC, g(0x71A17C));
            p(0x71A200, g(0x71A180));
            p(0x71A204, g(0x71A184));
            crate::physics::wall_push_back::steps(r)?;
            after(r)
        }
    }
}

/// The probe count checked: 0 or past 0x40 refused.
pub fn check() -> Result<(), Refused> {
    Ok(walls::check_probe_count(g(0x71A238))?)
}

/// The Rust version.
pub unsafe fn wall_contacts(entry: &Regs) -> Result<Left, Refused> {
    check()?;
    guard(&regions(), || {
        let mut r = *entry;
        steps(&mut r)?;
        Ok(left_of(&r))
    })
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { wall_contacts(r) }
}

/// Everything the three wall hooks and their calls write.
pub fn regions() -> Vec<Region> {
    let mut all = contact_force::regions();
    all.extend(norm_regions());
    all.extend(crate::physics::feel_ground::regions());
    all.extend(crate::physics::crash_contact::sound_regions());
    all.extend([
        (a(WALL_RECORD), 0x18),
        (a(contact_force::RECORD), 4),
        (a(0x6141AC), 0x20), // the edge's ends .. dword_6141C8
        (a(0x6D3170), 0x10),
        (a(0x6D3214), 4),
        (a(0x6D3038), 4),
        (a(0x62400C), 4),
        (a(0x71A1FC), 0x14),
        (a(0x71A1CC), 4),
        crate::math::turn_scratch_xz::scratch(),
        // `restore_saved_car`'s
        (a(0x614738), 4),
        (a(0x71A10C), 12),
        (a(0x71A08C), 0x40),
        (a(0x71ADC8), 0x100),
    ]);
    all
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

/// Faces of the stage by their wall bits (word +2 & 0xC00: 0, 0x400, 0x800, 0xC00), a
/// few of each.
fn faces_by_bits() -> [Vec<u32>; 4] {
    let mut out: [Vec<u32>; 4] = Default::default();
    for k in 0..g(0x5CCD1C) {
        let bits = (word(k) >> 10 & 3) as usize;
        if out[bits].len() < 6 {
            out[bits].push(k);
        }
    }
    out
}

/// The tests of the three wall hooks (`hook`, its `call_original` and Rust version).
/// `wall_push_back` and `wall_edge_contact` run on test faces and points (any time);
/// `wall_contacts`, whose every contact ends in `feel_ground`, on the stage in a race.
pub fn test_walls(hook: &RawHook, call_original: CallOriginal, rust: Rust) -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(hook) else {
        return (0, 0);
    };
    let whole = std::ptr::eq(hook, &HOOK);
    if whole && (!crate::physics::move_slot::in_race() || !(1..=0x40).contains(&g(0x71A238))) {
        return t.skip("not in a race (no stage)");
    }
    let car = crate::physics::car_sounds::CURRENT;
    let mut all = regions();
    all.extend(contact_force::inputs());
    all.extend([
        (a(car), 4),
        (a(crate::physics::car_sounds::LISTED), 0x20),
        (a(FACES), 4),
        (a(crate::physics::scene_point_hits_car::SCENE_POINTS), 4),
        (a(0x71A238), 4),
        (a(0x6D2214), 0x300),
        (a(0x6D2814), 0x100),
        (a(0x71AAC8), 0x300),
        (a(0x71A15C), 0x2C),
        (a(0x71A208), 4),
        (a(0x614148), 8),
    ]);
    let saved = capture(&all);
    let mut faces = [0u32; 0x10 * 4];
    let mut points = [0u32; 3 * 8];
    let (fs, ps) = (faces.as_mut_ptr() as u32, points.as_mut_ptr() as u32);
    if !whole {
        all.extend([(fs, 0x100), (ps, 0x60)]);
    }
    let bits = if whole {
        faces_by_bits()
    } else {
        Default::default()
    };
    let count = g(0x71A238);
    for word in TEST_CONTROL_WORDS {
        with_control_word(word, &mut || {
            for seed in 0..12u32 {
                let set = || {
                    restore(&all[..all.len() - if whole { 0 } else { 2 }], &saved);
                    let mut f = vec![0u32; 0x400];
                    test_floats(seed * 23 + 1, &mut f);
                    if !whole {
                        // The stage's own figures in a race; else by lot.
                        contact_force::set_inputs(seed);
                        p(0x614148, 0);
                        let small = |v: u32| (f32::from_bits(v) % 8.0).to_bits();
                        for k in 0..0xC0 {
                            p(0x6D2214 + 4 * k, small(f[k]));
                            p(0x71AAC8 + 4 * k, small(f[k + 0xC0]));
                        }
                        for k in 0..11 {
                            p(0x71A15C + 4 * k, small(f[k + 0x180]));
                        }
                        p(0x71A208, small(f[0x18B]));
                    }
                    let c = seed % 8;
                    p(car, c);
                    p(
                        crate::physics::car_sounds::LISTED + 4 * c as usize,
                        u32::MAX,
                    );
                    if seed % 6 == 5 && !whole {
                        // Standing still.
                        for k in 0..3 {
                            p(0x71A170 + 4 * k, 0);
                        }
                    }
                    if whole {
                        // Probe lists from the stage's faces: none met, a wall met, the
                        // 0xC00 fallback.
                        let pick = |b: usize, i: u32| {
                            let v = &bits[b];
                            if v.is_empty() {
                                0
                            } else {
                                v[i as usize % v.len()]
                            }
                        };
                        for k in 0..count {
                            let (now, then) = match seed % 3 {
                                0 => (pick(0, k), pick(0, k)),
                                1 => (pick(1, k), if k == 3 { pick(2, k) } else { pick(1, k) }),
                                _ => (pick(3, k), pick(3, k)),
                            };
                            p(0x71ADC8 + 4 * k as usize, now);
                            p(0x6D2814 + 4 * k as usize, then);
                        }
                    } else {
                        p(FACES, fs);
                        p(crate::physics::scene_point_hits_car::SCENE_POINTS, ps);
                        for k in 0..0x18 {
                            unsafe {
                                ((ps + 4 * k) as *mut u32).write(f[0x200 + k as usize] % 0x40000)
                            };
                        }
                        // Two faces of corners 0 .. 7; the second shares an edge (by
                        // seed) with the first, or none.
                        let wall = [0u32, 1, 2, 3];
                        let shared: [[u32; 4]; 5] = [
                            [1, 0, 5, 6],
                            [5, 2, 6, 0],
                            [5, 6, 3, 2],
                            [4, 7, 1, 3],
                            [4, 5, 6, 7],
                        ];
                        let other = shared[(seed % 5) as usize];
                        for k in 0..4u32 {
                            unsafe {
                                ((fs + 4 + 4 * k) as *mut u32).write(wall[k as usize]);
                                ((fs + 0x44 + 4 * k) as *mut u32).write(other[k as usize]);
                            }
                        }
                        p(0x6D3170, 0);
                        p(0x6D3174, 1);
                        p(0x6D3214, seed % 0x40);
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
    if !whole {
        all.truncate(all.len() - 2);
    }
    restore(&all, &saved);
    t.finish()
}

fn self_test() -> (usize, usize) {
    test_walls(&HOOK, call_original, &rust)
}
