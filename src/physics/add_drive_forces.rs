//! `physics::add_drive_forces` (0x440C7B): a factor (`flt_613EE4`, or with
//! `g_carDriveKind` set `flt_613EE8`, the first then stored in `dword_62400C`) times the
//! sum of the four floats at `flt_614284` (80 bits) into `flt_6140F4`; then
//! `flt_614248` += `flt_614234` x it x `flt_6140B8` and `flt_61424C` += `flt_614238` x it
//! x `flt_6140B8`.
//!
//! In:  nothing
//! Out: every register kept; no flags read (the caller's next is a call)
//!
//! Caller: `sub_43DFB1` (0x43E51E).
//! Self-test: a copy of the original (its variables the test's) against the converted
//! version, both drive kinds, values by lot, exact at double precision.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, call_game, game_addr, game_read, hook_asm, rva};
use crate::math::x87;
use crate::math::x87_run::{TEST_CONTROL_WORDS, mul2_plus, with_control_word};
use crate::physics::copy_hooks_test::check_floats_at;
use crate::scene::matrix3::test_floats;
use crate::screens::harness::Left;
use crate::service::floats::{FLOAT, Floats, dispatch_state_floats, floats_entry};
use crate::service::harness::capture;
use rc2k_core::physics::drive_forces::add_drive_forces as add_drive_forces_core;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("physics::add_drive_forces", rva(0x440C7B), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The variables (IDA addresses), in the order the core takes them.
pub const VARIABLES: [usize; 11] = [
    0x62400C, 0x71A018, 0x614284, 0x614288, 0x61428C, 0x614290, 0x6140F4, 0x614234, 0x614238,
    0x614248, 0x61424C,
];

unsafe fn at(a: u32) -> u32 {
    unsafe { (a as *const u32).read_unaligned() }
}

unsafe fn put(a: u32, v: u32) {
    unsafe { (a as *mut u32).write_unaligned(v) }
}

fn k(a: usize) -> u32 {
    unsafe { game_read(rva(a)) }
}

/// The core on the variables' addresses.
pub unsafe fn add_drive_forces_at(v: &[u32]) {
    unsafe {
        let mut factor = k(0x613EE4);
        if at(v[1]) != 0 {
            put(v[0], x87::loaded(factor));
            factor = k(0x613EE8);
        }
        put(
            v[6],
            x87::sum4_times(at(v[2]), at(v[3]), at(v[4]), at(v[5]), factor),
        );
        let scale = k(0x6140B8);
        put(v[9], mul2_plus(at(v[7]), at(v[6]), scale, at(v[9])));
        put(v[10], mul2_plus(at(v[8]), at(v[6]), scale, at(v[10])));
    }
}

/// The converted version (`rc2k_core::physics::drive_forces`, plain floats) on the
/// variables' addresses.
// Converted: `rc2k_core::physics::drive_forces::add_drive_forces`.
pub unsafe fn add_drive_forces_plain_at(v: &[u32]) {
    unsafe {
        let f = |n: usize| f32::from_bits(at(v[n]));
        let kind = at(v[1]);
        let factors = [f32::from_bits(k(0x613EE4)), f32::from_bits(k(0x613EE8))];
        let d = add_drive_forces_core(
            kind,
            factors,
            [f(2), f(3), f(4), f(5)],
            [f(7), f(8)],
            f32::from_bits(k(0x6140B8)),
            [f(9), f(10)],
        );
        if let Some(kept) = d.kept {
            put(v[0], kept.to_bits());
        }
        put(v[6], d.drive.to_bits());
        put(v[9], d.forces[0].to_bits());
        put(v[10], d.forces[1].to_bits());
    }
}

fn game() -> Vec<u32> {
    VARIABLES
        .iter()
        .map(|&x| game_addr(rva(x)) as u32)
        .collect()
}

/// The Rust version (the exact one, for `physics::drive_wheels_step`).
pub unsafe fn add_drive_forces() {
    unsafe { add_drive_forces_at(&game()) }
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { add_drive_forces_plain_at(&game()) };
    Ok(Left::nothing())
}

extern "C" fn dispatch(r: &mut Regs) {
    let v = game();
    let floats = Floats {
        memory: &[((v[6], 4), FLOAT), ((v[9], 8), FLOAT)],
        regs: &[],
    };
    dispatch_state_floats(
        &HOOK,
        r,
        call_original,
        &[(v[0], 4), (v[6], 4), (v[9], 8)],
        &floats,
        &rust,
    );
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    floats_entry(r, &rust)
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let mut cells = vec![0u32; 11];
    let base = cells.as_mut_ptr() as u32;
    let v: Vec<u32> = (0..11).map(|n| base + 4 * n).collect();
    let vars: Vec<(usize, u32)> = VARIABLES.iter().copied().zip(v.iter().copied()).collect();
    let copy = match crate::audio::test_copy::make(
        &HOOK,
        &crate::audio::test_copy::Copy {
            address: 0x440C7B,
            len: 0x78,
            entry: &[0xD9, 0x05, 0xE4, 0x3E, 0x61, 0x00],
            calls: &[],
            jumps_out: &[],
        },
        &|_| None,
        &vars,
        15,
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    let all = [(base, 44)];
    for word in TEST_CONTROL_WORDS {
        with_control_word(word, &mut || {
            for seed in 0..8u32 {
                let fill = || {
                    let mut f = [0u32; 11];
                    test_floats(seed * 7 + 2, &mut f);
                    f[0] = 0xAAAA_AAAA;
                    f[1] = seed % 2;
                    f[6] = 0xAAAA_AAAA;
                    unsafe { (base as *mut [u32; 11]).write(f) };
                };
                fill();
                let mut ro = Regs::SENTINEL;
                unsafe { call_game(&mut ro, copy.addr()) };
                let theirs = capture(&all);
                fill();
                unsafe { add_drive_forces_plain_at(&v) };
                let mine = capture(&all);
                let regs_ok = ro.preserved_except(&Regs::SENTINEL, 0);
                let floats = [((v[6], 4), FLOAT), ((v[9], 8), FLOAT)];
                check_floats_at(
                    &mut t,
                    "physics::add_drive_forces",
                    word,
                    &all,
                    &theirs,
                    &mine,
                    &floats,
                    regs_ok,
                    || {
                        format!(
                            "control word {word:#x}, seed {seed}: regs_ok={regs_ok} original {theirs:x?}; rust {mine:x?}"
                        )
                    },
                );
            }
        });
    }
    drop((copy, cells));
    t.finish()
}
