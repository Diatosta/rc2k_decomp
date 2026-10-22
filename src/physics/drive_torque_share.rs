//! `physics::drive_torque_share` (0x440D56): a share of the drive by speed. n =
//! `dword_613EEC` (8) with `g_carDriveKind` 1, else `dword_613EF0` (32), into
//! `dword_62400C`; s = |`g_carSpeed`| / `dword_613EF4` (`fidiv`) into `flt_62401C`, held to
//! at most `flt_5C5D38` (1.0; one below it, or unordered, is left); d = (n - `g_car`
//! +0x28, `unk_71A000`) / n (`fild` / `fidiv`) into `dword_624020`; `flt_614758` = d +
//! (1 - d) s, and `flt_614750` times it.
//!
//! In:  nothing
//! Out: eax n less the word (no `pusha`); the rest kept; no flags read (the callers return
//!      or load floats next)
//!
//! Callers: `sub_421848`, `sub_43CAAB`, `sub_43CB14`.
//! Self-test: a copy of the original (its variables the test's) against the converted
//! version: both drive kinds, speeds about the limit, words by lot, exact at double
//! precision.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, call_game, game_addr, game_read, hook_asm, rva};
use crate::math::x87;
use crate::math::x87_run::{
    TEST_CONTROL_WORDS, abs_fidiv, int_fidiv, one_less_times_plus, with_control_word,
};
use crate::physics::copy_hooks_test::check_floats_at;
use crate::scene::matrix3::test_floats;
use crate::screens::harness::Left;
use crate::service::floats::{FLOAT, Floats, dispatch_state_floats, floats_entry};
use crate::service::harness::capture;
use rc2k_core::physics::drive_forces::torque_share;
use std::cmp::Ordering;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "physics::drive_torque_share",
    rva(0x440D56),
    thunk,
    &ORIGINAL,
)
.self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The variables (IDA addresses), in the order the core takes them.
pub const VARIABLES: [usize; 8] = [
    0x62400C, 0x71A018, 0x71A184, 0x62401C, 0x624020, 0x71A000, 0x614758, 0x614750,
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

/// The core on the variables' addresses: eax as the original leaves it (n less the
/// word, its `sub eax, unk_71A000`).
pub unsafe fn drive_torque_share_at(v: &[u32]) -> u32 {
    unsafe {
        let n = if at(v[1]) == 1 {
            k(0x613EEC)
        } else {
            k(0x613EF0)
        };
        put(v[0], n);
        put(v[3], abs_fidiv(at(v[2]), k(0x613EF4) as i32));
        let one = k(0x5C5D38);
        let below = !matches!(
            f32::from_bits(at(v[3])).partial_cmp(&f32::from_bits(one)),
            Some(Ordering::Greater | Ordering::Equal)
        );
        if !below {
            put(v[3], x87::loaded(one));
        }
        let eax = at(v[0]).wrapping_sub(at(v[5]));
        put(v[4], eax);
        put(v[4], int_fidiv(at(v[4]) as i32, at(v[0]) as i32));
        put(v[6], one_less_times_plus(at(v[4]), at(v[3])));
        put(v[7], x87::mul(at(v[7]), at(v[6])));
        eax
    }
}

/// The converted version (`rc2k_core::physics::drive_forces`, plain floats) on the
/// variables' addresses: eax as the original leaves it.
// Converted: `rc2k_core::physics::drive_forces::torque_share`.
pub unsafe fn drive_torque_share_plain_at(v: &[u32]) -> u32 {
    unsafe {
        let s = torque_share(
            at(v[1]),
            [k(0x613EEC), k(0x613EF0)],
            f32::from_bits(at(v[2])),
            k(0x613EF4) as i32,
            f32::from_bits(k(0x5C5D38)),
            at(v[5]),
            f32::from_bits(at(v[7])),
        );
        put(v[0], s.n);
        put(v[3], s.speed_share.to_bits());
        put(v[4], s.drive_share.to_bits());
        put(v[6], s.share.to_bits());
        put(v[7], s.drive.to_bits());
        s.less
    }
}

/// The variables' addresses in the game (for `physics::drive_force`'s converted version).
pub fn drive_torque_share_places() -> Vec<u32> {
    game()
}

fn game() -> Vec<u32> {
    VARIABLES
        .iter()
        .map(|&x| game_addr(rva(x)) as u32)
        .collect()
}

/// The Rust version (the exact one, for `physics::drive_force` and
/// `physics::test_drive_run`): eax as the original leaves it.
pub unsafe fn drive_torque_share() -> u32 {
    unsafe { drive_torque_share_at(&game()) }
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    let eax = unsafe { drive_torque_share_plain_at(&game()) };
    Ok(Left::nothing().with(crate::framework::EAX, eax))
}

extern "C" fn dispatch(r: &mut Regs) {
    let v = game();
    let floats = Floats {
        memory: &[((v[3], 8), FLOAT), ((v[6], 4), FLOAT), ((v[7], 4), FLOAT)],
        regs: &[],
    };
    dispatch_state_floats(
        &HOOK,
        r,
        call_original,
        &[(v[0], 4), (v[3], 8), (v[6], 4), (v[7], 4)],
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
    let mut cells = vec![0u32; 8];
    let base = cells.as_mut_ptr() as u32;
    let v: Vec<u32> = (0..8).map(|n| base + 4 * n).collect();
    let vars: Vec<(usize, u32)> = VARIABLES.iter().copied().zip(v.iter().copied()).collect();
    let copy = match crate::audio::test_copy::make(
        &HOOK,
        &crate::audio::test_copy::Copy {
            address: 0x440D56,
            len: 0x9D,
            entry: &[0xA1, 0xEC, 0x3E, 0x61, 0x00],
            calls: &[],
            jumps_out: &[],
        },
        &|_| None,
        &vars,
        19,
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    let all = [(base, 32)];
    for word in TEST_CONTROL_WORDS {
        with_control_word(word, &mut || {
            for seed in 0..10u32 {
                let fill = || {
                    let mut f = [0u32; 3];
                    test_floats(seed * 5 + 9, &mut f);
                    let speed = match seed {
                        0 => 32.0f32.to_bits(),
                        1 => 0x7FC0_0000,
                        _ => (f32::from_bits(f[0]) % 60.0).to_bits(),
                    };
                    let w = [
                        0xAAAA_AAAA,
                        seed % 3,
                        speed,
                        0xAAAA_AAAA,
                        0xAAAA_AAAA,
                        seed * 3,
                        0xAAAA_AAAA,
                        f[1],
                    ];
                    unsafe { (base as *mut [u32; 8]).write(w) };
                };
                fill();
                let mut ro = Regs::SENTINEL;
                unsafe { call_game(&mut ro, copy.addr()) };
                let theirs = capture(&all);
                fill();
                let eax = unsafe { drive_torque_share_plain_at(&v) };
                let mine = capture(&all);
                let regs_ok =
                    ro.preserved_except(&Regs::SENTINEL, crate::framework::EAX) && ro.eax == eax;
                let floats = [((v[3], 8), FLOAT), ((v[6], 8), FLOAT)];
                check_floats_at(
                    &mut t,
                    "physics::drive_torque_share",
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
