//! `physics::scale_vector` (0x421297): the three floats at esi each times st(0)
//! (`fld; fmul st, st(1); fstp`, x, y, z), st(0) left as it was.
//!
//! In:  esi the vector, st(0) the factor
//! Out: the vector scaled; st(0) the factor still; every register kept
//!
//! The thunk (here, not a shared macro) pops the factor onto its stack, and inside the
//! gate (as `hook_asm!`'s) copies it into `FACTOR`, runs `dispatch` and copies `KEPT`
//! (what the original or the Rust version left in st(0)) back for loading; `original_with_factor` loads the factor, runs the original and pops its
//! st(0) into `KEPT`, so shadow mode compares that too.
//! Callers (12): the cars' physics under `sub_4454D2` (`sub_43BDD1`, `sub_43DEF4`,
//! `sub_43F7AC`, `sub_45F17E`, `sub_45F7B1`, `sub_45FAE1`, two in unowned code at
//! 0x43F5A0, 0x43F647).
//! Self-test: the original and the Rust version on vectors and factors by lot, under three
//! precisions.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, rva};
use crate::math::x87_run::{TEST_CONTROL_WORDS, scale_by_80, with_control_word};
use crate::race::stage_prefs::left_of;
use crate::scene::matrix3::{extended, test_floats};
use crate::scene::vector3::Result80;
use crate::screens::harness::Left;
use crate::service::harness::{check_state, dispatch_state};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);
/// The factor the caller left in st(0).
static FACTOR: Result80 = Result80::new();
/// What goes back in st(0).
static KEPT: Result80 = Result80::new();

pub static HOOK: RawHook =
    RawHook::new("physics::scale_vector", rva(0x421297), thunk, &ORIGINAL).self_test(self_test);

#[unsafe(naked)]
pub(crate) unsafe extern "C" fn thunk() {
    core::arch::naked_asm!(
        "lea esp, [esp - 12]",
        "fstp tbyte ptr [esp]", // the factor; no flags touched
        "pushfd",
        "pushad",
        "cld",
        "call {enter}",
        "push esp",             // arg: *mut Regs
        "lea eax, [esp + 40]",
        "push eax",
        "call {keep}",          // the factor into FACTOR, once the gate is passed
        "add esp, 4",
        "call {dispatch}",
        "add esp, 4",
        "lea eax, [esp + 36]",
        "push eax",
        "call {give}",          // KEPT onto the stack, before the gate is left
        "add esp, 4",
        "call {leave}",
        "popad",
        "popfd",
        "fld tbyte ptr [esp]",  // no flags touched from here
        "lea esp, [esp + 12]",
        "ret",
        enter = sym crate::framework::hook_enter,
        leave = sym crate::framework::hook_leave,
        keep = sym keep,
        give = sym give,
        dispatch = sym dispatch,
    )
}

extern "C" fn keep(at: *const [u8; 10]) {
    FACTOR.set(unsafe { at.read_unaligned() });
}

extern "C" fn give(at: *mut [u8; 10]) {
    unsafe { at.write_unaligned(*KEPT.0.get()) };
}

/// The original with the factor in st(0); its st(0) taken off into `KEPT`.
#[unsafe(naked)]
pub(crate) unsafe extern "C" fn original_with_factor(_regs: *mut Regs) {
    core::arch::naked_asm!(
        "push dword ptr [esp + 4]",
        "fld tbyte ptr [{f}]",
        "call {co}",
        "add esp, 4",
        "fstp tbyte ptr [{k}]",
        "ret",
        co = sym call_original,
        f = sym FACTOR,
        k = sym KEPT,
    )
}

crate::framework::hook_asm!(@call_original call_original, ORIGINAL);

/// The Rust version: the vector at `v` times the 80-bit `factor`.
pub unsafe fn scale_vector(v: u32, factor: [u8; 10]) {
    unsafe { scale_by_80(v as *mut u32, factor) }
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    let factor = unsafe { *FACTOR.0.get() };
    unsafe { scale_vector(r.esi, factor) };
    KEPT.set(factor);
    Ok(left_of(r))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(
        &HOOK,
        r,
        original_with_factor,
        &[KEPT.region(), (r.esi, 12)],
        &rust,
    );
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let saved = unsafe { *FACTOR.0.get() };
    let mut v = [0u32; 4];
    let p = v.as_mut_ptr() as u32;
    for word in TEST_CONTROL_WORDS {
        with_control_word(word, &mut || {
            for seed in 1..=12u32 {
                check_state(
                    &mut t,
                    &format!("control word {word:#x}, seed {seed}"),
                    original_with_factor,
                    Regs {
                        esi: p,
                        ..Regs::SENTINEL
                    },
                    &|| {
                        let mut x = [0u32; 4];
                        test_floats(seed, &mut x);
                        unsafe { (p as *mut [u32; 3]).write([x[0], x[1], x[2]]) };
                        // An 80-bit factor that is not a float: a float times 1 + 2^-40.
                        let mut f = extended(x[3]);
                        f[0] ^= (seed as u8) | 1;
                        FACTOR.set(f);
                        KEPT.set([0xAA; 10]);
                    },
                    &[KEPT.region(), (p, 12)],
                    &rust,
                );
            }
        });
    }
    FACTOR.set(saved);
    t.finish()
}
