//! `marks::trail_particles` (0x4CFFE3): eight entries of the marks' particle ring
//! (`g_marksRingFree`'s +0, each while `g_marksRingUsed` is not 0xFFF) started along the
//! way the particle at esi came: for each, kind 4; `g_marksBurstRadius` = the marks'
//! random's low word times `flt_8EB969` times `flt_93C648`; the place (+0xC .. +0x14) the
//! particle's place moved that share toward its last place (+0x18 .. +0x20); the motion
//! (+0x28, +0x2C, +0x34) (last - place) times `flt_93C64C` / 54 / 5C plus the particle's
//! motion times `flt_93C650` / 58 / 60, moved by `marks::jitter_point` (spread
//! `flt_93C664`); +0x38 the particle's +0x2C times `flt_93C668` plus `flt_93C66C`
//! (`fistp`), +0x3C 0xFF000000, +0x44 300, +0x40 `flt_93C670` / 300, +0x24 the view's
//! +0x34.
//!
//! In:  esi the particle
//! Out: edi the particle; the rest kept (each entry's work inside `pusha` .. `popa`)
//!
//! Caller: the ring's update (code at 0x4D033F, kind 0 particles moving up). Its first
//! five bytes end inside a jump: private copy (its one call relocated), manual patch.
//! Refuses a null particle or ring (an entry to take), before anything. Self-test: a ring
//! of the test's, the original and the Rust version at several fillings and randoms.

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva};
use crate::marks::blocks::{RING_ENTRY, RING_FREE_RVA, RING_USED_RVA};
use crate::marks::jitter_point::jitter_point;
use crate::marks::setup::SEED_RVA;
use crate::math::x87;
use crate::race::stage_prefs::left_of;
use crate::scene::matrix3::{extended, rounded};
use crate::screens::harness::Left;
use crate::service::harness::{
    Region, at, capture, check_state, dispatch_state, restore, state_entry,
};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new("marks::trail_particles", rva(0x4CFFE3), thunk, &ORIGINAL)
    .private_copy(0x163)
    .relocate_calls(&[0x100])
    .manual_patch()
    .self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const FULL: u32 = 0xFFF;
const COUNT: u32 = 8;
const RADIUS_RVA: usize = rva(0x93A5E4); // g_marksBurstRadius
const RANDOM_K: [usize; 2] = [rva(0x8EB969), rva(0x93C648)];
/// Per axis (place offset, last offset, motion offset, its two factors).
const AXES: [(u32, u32, u32, usize, usize); 3] = [
    (0xC, 0x18, 0x28, rva(0x93C64C), rva(0x93C650)),
    (0x10, 0x1C, 0x2C, rva(0x93C654), rva(0x93C658)),
    (0x14, 0x20, 0x34, rva(0x93C65C), rva(0x93C660)),
];
const SPREAD_RVA: usize = rva(0x93C664); // flt_93C664
const SIZE_K: [usize; 2] = [rva(0x93C668), rva(0x93C66C)];
const LIFE_K_RVA: usize = rva(0x93C670); // flt_93C670
const VIEW_RVA: usize = rva(0x5EA3CC); // flt_5EA3CC: the view's place at +0x30
const LIFE: u32 = 300;

unsafe fn get(a: u32) -> u32 {
    unsafe { (a as *const u32).read_unaligned() }
}

/// The entries it takes, from `g_marksRingFree` along +0.
unsafe fn entries_taken() -> Vec<u32> {
    unsafe {
        let used: u32 = game_read(RING_USED_RVA);
        let n = if used <= FULL {
            (FULL - used).min(COUNT)
        } else {
            COUNT
        };
        let mut e: u32 = game_read(RING_FREE_RVA);
        let mut out = Vec::new();
        for _ in 0..n {
            if e == 0 {
                break;
            }
            e = get(e);
            out.push(e);
        }
        out
    }
}

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn trail_particles(entry: &Regs) -> Result<Regs, Refused> {
    let mut g = *entry;
    unsafe {
        if game_read::<u32>(RING_USED_RVA) != FULL {
            if g.esi == 0 {
                return Err(BadInput("a null particle").into());
            }
            if game_read::<u32>(RING_FREE_RVA) == 0 || entries_taken().contains(&0) {
                return Err(BadInput("a null particle ring entry").into());
            }
        }
        g.edi = g.esi;
        let p = g.edi;
        for _ in 0..COUNT {
            let used: u32 = game_read(RING_USED_RVA);
            if used == FULL {
                continue;
            }
            let e = get(game_read(RING_FREE_RVA));
            game_write(RING_FREE_RVA, e);
            game_write(RING_USED_RVA, used.wrapping_add(1));
            let put = |o: u32, v: u32| ((e + o) as *mut u32).write_unaligned(v);
            put(8, 4);
            let s = game_read::<u32>(SEED_RVA)
                .wrapping_mul(0x9D)
                .wrapping_add(0x47);
            game_write(SEED_RVA, s);
            let [k1, k2] = RANDOM_K.map(|r| game_read::<u32>(r));
            game_write(
                RADIUS_RVA,
                x87::int_times_times((s & 0xFFFF) as i32, k1, k2),
            );
            let radius: u32 = game_read(RADIUS_RVA);
            for (place, last, _, _, _) in AXES {
                put(place, x87::toward(get(p + place), get(p + last), radius));
            }
            for (place, last, motion, k, m) in AXES {
                put(
                    motion,
                    x87::diff_times_plus_times(
                        get(p + last),
                        get(p + place),
                        game_read(k),
                        get(p + motion),
                        game_read(m),
                    ),
                );
            }
            let (moved, _) = jitter_point([
                extended(game_read(SPREAD_RVA)),
                extended(get(e + 0x34)),
                extended(get(e + 0x2C)),
                extended(get(e + 0x28)),
            ]);
            put(0x34, rounded(moved[0]));
            put(0x2C, rounded(moved[1]));
            put(0x28, rounded(moved[2]));
            let [a, b] = SIZE_K.map(|r| game_read::<u32>(r));
            put(0x38, x87::times_plus_to_int(a, get(p + 0x2C), b) as u32);
            put(0x3C, 0xFF00_0000);
            put(0x44, LIFE);
            put(0x40, x87::over_int(game_read(LIFE_K_RVA), LIFE as i32));
            put(0x24, x87::loaded(game_read(VIEW_RVA + 0x34)));
        }
    }
    Ok(g)
}

/// What it may write.
fn regions() -> Vec<Region> {
    let mut r: Vec<Region> = [RING_FREE_RVA, RING_USED_RVA, SEED_RVA, RADIUS_RVA]
        .map(at)
        .to_vec();
    r.extend(
        unsafe { entries_taken() }
            .into_iter()
            .filter(|&e| e != 0)
            .map(|e| (e, RING_ENTRY as usize)),
    );
    r
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { trail_particles(r) }?))
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
    const TEST_ENTRIES: u32 = 0x10;
    let mut ring = vec![0u8; (TEST_ENTRIES * RING_ENTRY) as usize];
    let base = ring.as_mut_ptr() as u32;
    let mut particle = [0u32; 0x13];
    for (n, v) in particle.iter_mut().enumerate() {
        *v = (n as f32 * 1.75 - 6.0).to_bits();
    }
    let pa = particle.as_ptr() as u32;
    let mut game: Vec<Region> = [
        RING_FREE_RVA,
        RING_USED_RVA,
        SEED_RVA,
        RADIUS_RVA,
        SPREAD_RVA,
        LIFE_K_RVA,
    ]
    .map(at)
    .to_vec();
    game.extend(RANDOM_K.map(at));
    game.extend(SIZE_K.map(at));
    for (_, _, _, k, m) in AXES {
        game.extend([at(k), at(m)]);
    }
    game.push((game_addr(VIEW_RVA) as u32 + 0x34, 4));
    let saved = capture(&game);
    let mut regions: Vec<Region> = [RING_FREE_RVA, RING_USED_RVA, SEED_RVA, RADIUS_RVA]
        .map(at)
        .to_vec();
    regions.push((base, ring.len()));
    for (case, (used, seed)) in [
        (0u32, 0x1234_5678u32),
        (0xFFA, 0xDEAD_BEEF),
        (FULL, 7),
        (0x1000, 3),
        (0x20, 0xFFFF_FFFF),
    ]
    .into_iter()
    .enumerate()
    {
        check_state(
            &mut t,
            &format!("used {used:#x}, seed {seed:#x}"),
            call_original,
            Regs {
                esi: pa,
                ..Regs::SENTINEL
            },
            &|| unsafe {
                restore(&game, &saved);
                std::ptr::write_bytes(base as *mut u8, 0xAA, ring.len());
                for n in 0..TEST_ENTRIES {
                    let e = (base + n * RING_ENTRY) as *mut u32;
                    e.write_unaligned(base + ((n + 1) % TEST_ENTRIES) * RING_ENTRY);
                }
                game_write(RING_FREE_RVA, base + 3 * RING_ENTRY);
                game_write(RING_USED_RVA, used);
                game_write(SEED_RVA, seed);
                let k = case as f32;
                game_write(RANDOM_K[0], (1.0f32 / 65536.0).to_bits());
                game_write(RANDOM_K[1], (0.8f32 + k * 0.1).to_bits());
                for (n, (_, _, _, a, b)) in AXES.into_iter().enumerate() {
                    game_write(a, (0.3f32 + n as f32 * 0.07).to_bits());
                    game_write(b, (0.9f32 - n as f32 * 0.11 + k).to_bits());
                }
                game_write(SPREAD_RVA, (0.25f32 + k).to_bits());
                game_write(SIZE_K[0], (300.0f32 + k).to_bits());
                game_write(SIZE_K[1], 12000.5f32.to_bits());
                game_write(LIFE_K_RVA, 0.75f32.to_bits());
                game_write(VIEW_RVA + 0x34, (2.5f32 * k).to_bits());
            },
            &regions,
            &rust,
        );
    }
    restore(&game, &saved);
    drop(ring);
    t.finish()
}
