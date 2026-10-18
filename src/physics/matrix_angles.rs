//! `physics::matrix_angles` (0x4212C3): three angles of the turn matrix at esi. The
//! matrix turns (0, 0, 1) (`scene::rotate_point`, stored as floats in `flt_5C70E0`,
//! `E4`, `E8`): `flt_5C70D8` = `fpatan` of E0 over E8; the same turn again, and
//! `flt_5C70D4` = minus `physics::clamped_asin` of E4; then (1, 0, 0) turned, and
//! `flt_5C70DC` = `clamped_asin` of E4 over the cosine of D4 (80 bits). A reading: the
//! car's heading (D8), pitch (D4) and roll (DC).
//!
//! In:  esi the matrix
//! Out: every register kept; the x87 stack as it was; no flags read
//!
//! Callers: `physics::car_angles` (`g_crashMatrix`).
//! Self-test: a copy of the original (its six floats the test's, its call to
//! `clamped_asin` going to that one's test copy) on turn matrices and matrices by lot,
//! under three precisions.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, call_game, game_addr, hook_asm, rva};
use crate::math::x87_run::{
    TEST_CONTROL_WORDS, neg_rounded, over_cos_80, patan, with_control_word,
};
use crate::physics::clamped_asin::clamped_asin_at;
use crate::redirect::RedirectedCopy;
use crate::scene::matrix3::{extended, rounded, test_floats};
use crate::scene::rotate_point::rotate_point;
use crate::screens::harness::Left;
use crate::service::harness::{Region, capture, dispatch_state};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("physics::matrix_angles", rva(0x4212C3), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const TURNED_RVA: usize = rva(0x5C70E0); // flt_5C70E0, E4, E8
const ANGLES_RVA: usize = rva(0x5C70D4); // flt_5C70D4 (pitch), D8 (heading), DC (roll)
const SCRATCH_RVA: usize = rva(0x62400C); // dword_62400C (clamped_asin's)
const LIMIT_RVA: usize = rva(0x5C5D38); // flt_5C5D38 (clamped_asin's)

/// Where the function's variables are: the game's or a test's.
#[derive(Clone, Copy)]
pub struct AnglePlaces {
    /// The turned unit vector (three floats).
    pub turned: u32,
    /// The three angles.
    pub angles: u32,
    pub scratch: u32,
    pub limit: u32,
}

impl AnglePlaces {
    pub fn game() -> Self {
        Self {
            turned: game_addr(TURNED_RVA) as u32,
            angles: game_addr(ANGLES_RVA) as u32,
            scratch: game_addr(SCRATCH_RVA) as u32,
            limit: game_addr(LIMIT_RVA) as u32,
        }
    }

    /// What the function writes (the limit is put back).
    pub fn regions(&self) -> Vec<Region> {
        vec![(self.turned, 12), (self.angles, 12), (self.scratch, 4)]
    }
}

unsafe fn at(a: u32) -> u32 {
    unsafe { (a as *const u32).read_unaligned() }
}

unsafe fn put(a: u32, v: u32) {
    unsafe { (a as *mut u32).write_unaligned(v) }
}

/// `rotate_point` of the unit vector `unit` by `m`, stored as floats at `p.turned`.
unsafe fn turn_unit(m: u32, p: &AnglePlaces, unit: [u32; 3]) {
    unsafe {
        let out = rotate_point(m, unit.map(extended));
        for (k, e) in out.iter().enumerate() {
            put(p.turned + 4 * k as u32, rounded(*e));
        }
    }
}

/// The Rust core: the matrix at `m`, the variables at `p`.
pub unsafe fn matrix_angles_at(m: u32, p: &AnglePlaces) {
    const ONE: u32 = 0x3F80_0000;
    unsafe {
        turn_unit(m, p, [0, 0, ONE]);
        put(p.angles + 4, patan(at(p.turned), at(p.turned + 8)));
        turn_unit(m, p, [0, 0, ONE]);
        let pitch = clamped_asin_at(extended(at(p.turned + 4)), p.scratch, p.limit);
        put(p.angles, neg_rounded(pitch));
        turn_unit(m, p, [ONE, 0, 0]);
        let sine = over_cos_80(at(p.turned + 4), at(p.angles));
        let roll = clamped_asin_at(sine, p.scratch, p.limit);
        put(p.angles + 8, rounded(roll));
    }
}

/// The Rust version.
pub unsafe fn matrix_angles(m: u32) {
    unsafe { matrix_angles_at(m, &AnglePlaces::game()) }
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { matrix_angles(r.esi) };
    Ok(Left::nothing())
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(
        &HOOK,
        r,
        call_original,
        &AnglePlaces::game().regions(),
        &rust,
    );
}

/// Test memory for this function and its callers: a matrix, the places, three ints
/// (`car_angles`') and two switches (`update_car_angles`'), with the copies of
/// `clamped_asin` and this function made on them.
pub struct AngleTest {
    _cells: Vec<u32>,
    pub m: u32,
    pub places: AnglePlaces,
    pub ints: u32,
    pub switches: u32,
    /// This function's copy (its call to the copy of `clamped_asin`).
    pub entry: usize,
    _copies: Vec<RedirectedCopy>,
}

impl AngleTest {
    pub fn new() -> Result<Self, String> {
        let mut cells = vec![0u32; 16 + 3 + 3 + 2 + 3 + 2];
        let m = cells.as_mut_ptr() as u32;
        let places = AnglePlaces {
            turned: m + 0x40,
            angles: m + 0x4C,
            scratch: m + 0x58,
            limit: m + 0x5C,
        };
        let inner = crate::physics::clamped_asin::test_copy(places.scratch, places.limit)?;
        let target = inner.addr();
        let own = crate::audio::test_copy::make(
            &HOOK,
            &crate::audio::test_copy::Copy {
                address: 0x4212C3,
                len: 0x9A,
                entry: &[0xD9, 0xE8, 0xD9, 0xEE, 0xD9, 0xEE],
                calls: &[6, 0x37, 0x54, 0x67, 0x8E],
                jumps_out: &[],
            },
            &|r| (r == rva(0x42120B)).then_some(target),
            &[
                (0x5C70E0, places.turned),
                (0x5C70E4, places.turned + 4),
                (0x5C70E8, places.turned + 8),
                (0x5C70D4, places.angles),
                (0x5C70D8, places.angles + 4),
                (0x5C70DC, places.angles + 8),
            ],
            17,
        )?;
        Ok(Self {
            entry: own.addr(),
            _cells: cells,
            m,
            places,
            ints: m + 0x60,
            switches: m + 0x6C,
            _copies: vec![own, inner],
        })
    }

    /// The matrix by `seed` (a turn by three angles when even, floats by lot when odd),
    /// the limit 1.0, everything else 0xAA.
    pub fn fill(&self, seed: u32) {
        let mut f = [0u32; 16];
        if seed & 1 == 0 {
            let (a, b, c) = (
                seed as f32 * 0.37,
                seed as f32 * 0.11 - 0.5,
                seed as f32 * 0.23,
            );
            let (sa, ca) = a.sin_cos();
            let (sb, cb) = b.sin_cos();
            let (sc, cc) = c.sin_cos();
            let rows = [
                [ca * cc + sa * sb * sc, -ca * sc + sa * sb * cc, sa * cb],
                [cb * sc, cb * cc, -sb],
                [-sa * cc + ca * sb * sc, sa * sc + ca * sb * cc, ca * cb],
            ];
            for (r, row) in rows.iter().enumerate() {
                for (k, v) in row.iter().enumerate() {
                    f[4 * r + k] = v.to_bits();
                }
            }
            for (k, v) in f.iter_mut().enumerate().skip(12) {
                *v = (k as f32 - seed as f32).to_bits();
            }
        } else {
            test_floats(seed * 3 + 7, &mut f);
        }
        unsafe {
            (self.m as *mut [u32; 16]).write(f);
            for k in 0..8u32 {
                put(self.places.turned + 4 * k, 0xAAAA_AAAA);
            }
            put(self.places.limit, 0x3F80_0000);
            for k in 0..5u32 {
                put(self.ints + 4 * k, 0xAAAA_AAAA);
            }
        }
    }

    /// All of the test memory.
    pub fn all(&self) -> [Region; 1] {
        [(self.m, 4 * self._cells.len())]
    }
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let test = match AngleTest::new() {
        Ok(x) => x,
        Err(why) => return t.skip(&why),
    };
    for word in TEST_CONTROL_WORDS {
        with_control_word(word, &mut || {
            for seed in 0..12u32 {
                test.fill(seed);
                let before = Regs {
                    esi: test.m,
                    ..Regs::SENTINEL
                };
                let mut ro = before;
                unsafe { call_game(&mut ro, test.entry) };
                let theirs = capture(&test.all());
                test.fill(seed);
                unsafe { matrix_angles_at(test.m, &test.places) };
                let mine = capture(&test.all());
                let regs_ok = ro.preserved_except(&before, 0);
                t.check(regs_ok && theirs == mine, || {
                    let first = theirs.iter().zip(&mine).position(|(a, b)| a != b);
                    format!("control word {word:#x}, seed {seed}: regs_ok={regs_ok}, first difference at byte {first:?}")
                });
            }
        });
    }
    drop(test);
    t.finish()
}
