//! `physics::store_motion_length` (0x43BD2D): `dword_71A19C` = the length of the three
//! floats at `dword_71A170` (`physics::vector_length`, then `fstp`: rounded to a float).
//! Both are the work copy's (+0x198, +0x1C4); that the vector is the car's motion is a
//! reading (it sits next to `g_carSpeed`), not seen.
//!
//! In:  nothing
//! Out: every register kept
//!
//! Callers (2): `sub_43B635` (0x43B644, 0x43B94D), the car's physics step under
//! `sub_4454D2`.
//! Self-test: a copy of the original on test variables (its call to the
//! `vector_length` hook kept) against the converted version, vectors by lot, exact at
//! double precision.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, call_game, game_addr, hook_asm, rva};
use crate::math::x87_run::{TEST_CONTROL_WORDS, with_control_word};
use crate::physics::copy_hooks_test::check_floats_at;
use crate::physics::vector_length::vector_length;
use crate::scene::matrix3::{rounded, test_floats};
use crate::scene::vector3::read_floats;
use crate::screens::harness::Left;
use crate::service::floats::{FLOAT, Floats, dispatch_state_floats};
use rc2k_core::physics::motion::motion_length;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "physics::store_motion_length",
    rva(0x43BD2D),
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

const MOTION_RVA: usize = rva(0x71A170); // dword_71A170: three floats
const LENGTH_RVA: usize = rva(0x71A19C); // dword_71A19C

/// The Rust core on the vector at `v` and the length at `out`.
pub unsafe fn store_length(v: u32, out: u32) {
    unsafe { (out as *mut u32).write_unaligned(rounded(vector_length(v))) }
}

/// The converted version (`rc2k_core::physics::motion`, plain floats) on the vector at
/// `v` and the length at `out`.
// Converted: `rc2k_core::physics::motion::motion_length`.
pub unsafe fn store_length_plain(v: u32, out: u32) {
    unsafe { (out as *mut u32).write_unaligned(motion_length(read_floats(v)).to_bits()) }
}

/// The Rust version (the exact one, for `physics::car_physics_step` and
/// `physics::keep_motion_vectors`).
pub unsafe fn store_motion_length() {
    unsafe { store_length(game_addr(MOTION_RVA) as u32, game_addr(LENGTH_RVA) as u32) }
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { store_length_plain(game_addr(MOTION_RVA) as u32, game_addr(LENGTH_RVA) as u32) };
    Ok(Left::nothing())
}

extern "C" fn dispatch(r: &mut Regs) {
    let out = (game_addr(LENGTH_RVA) as u32, 4);
    let floats = Floats {
        memory: &[(out, FLOAT)],
        regs: &[],
    };
    dispatch_state_floats(&HOOK, r, call_original, &[out], &floats, &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let mut cells = vec![0u32; 4];
    let v = cells.as_mut_ptr();
    let out = unsafe { v.add(3) };
    let copy = match crate::audio::test_copy::make(
        &HOOK,
        &crate::audio::test_copy::Copy {
            address: 0x43BD2D,
            len: 0x14,
            entry: &[0x56, 0x8D, 0x35, 0x70, 0xA1, 0x71, 0x00],
            calls: &[7],
            jumps_out: &[],
        },
        &|_| None,
        &[(0x71A170, v as u32), (0x71A19C, out as u32)],
        2,
    ) {
        Ok(copy) => copy,
        Err(why) => return t.skip(&why),
    };
    let fill = |seed: u32| {
        let mut x = [0u32; 3];
        test_floats(seed * 3 + 2, &mut x);
        unsafe {
            (v as *mut [u32; 3]).write(x);
            out.write_volatile(0xAAAA_AAAA);
        }
    };
    for word in TEST_CONTROL_WORDS {
        with_control_word(word, &mut || {
            for seed in 1..=12u32 {
                fill(seed);
                let mut ro = Regs::SENTINEL;
                unsafe { call_game(&mut ro, copy.addr()) };
                let theirs = unsafe { (v as *const [u32; 4]).read() };
                fill(seed);
                unsafe { store_length_plain(v as u32, out as u32) };
                let mine = unsafe { (v as *const [u32; 4]).read() };
                let regs_ok = ro.preserved_except(&Regs::SENTINEL, 0);
                let bytes =
                    |w: [u32; 4]| w.iter().flat_map(|d| d.to_le_bytes()).collect::<Vec<u8>>();
                let floats = [((out as u32, 4), FLOAT)];
                check_floats_at(
                    &mut t,
                    "physics::store_motion_length",
                    word,
                    &[(v as u32, 16)],
                    &bytes(theirs),
                    &bytes(mine),
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
