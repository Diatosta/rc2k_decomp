//! `physics::clamped_asin` (0x42120B): the angle whose sine is st(0). The value is stored
//! as a float in `dword_62400C`, held there to -`flt_5C5D38` ..= `flt_5C5D38` (1.0; the
//! limit is negated in place for the second compare and put back), and the result is
//! `fpatan` of st(0) (unheld, 80 bits) over the root of 1 less the held float's square.
//! A value past the limit gives +-pi/2.
//!
//! In:  st(0) the sine
//! Out: st(0) the angle (in its place); every register kept; flags changed (no caller
//!      reads them)
//!
//! The thunk (as `physics::scale_vector`'s) pops st(0) into `INPUT`, runs `dispatch`
//! inside the gate and loads `RESULT`; `original_with_input` runs the original on
//! `INPUT` and pops its result into `RESULT`, so shadow mode compares it.
//! Callers: `physics::matrix_angles` (twice), unowned code at 0x42119E.
//! Self-test: a copy of the original (its scratch and limit the test's) on sines by lot
//! (80-bit ones, +-1, past the limit, NaN) and three limits, under three precisions.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, rva};
use crate::math::x87_run::{TEST_CONTROL_WORDS, clamped_asin_80, with_control_word};
use crate::scene::matrix3::{extended, test_floats};
use crate::scene::vector3::Result80;
use crate::screens::harness::Left;
use crate::service::harness::dispatch_state;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);
/// The value the caller left in st(0).
static INPUT: Result80 = Result80::new();
/// What goes back in st(0).
static RESULT: Result80 = Result80::new();

pub static HOOK: RawHook =
    RawHook::new("physics::clamped_asin", rva(0x42120B), thunk, &ORIGINAL).self_test(self_test);

#[unsafe(naked)]
pub(crate) unsafe extern "C" fn thunk() {
    core::arch::naked_asm!(
        "lea esp, [esp - 12]",
        "fstp tbyte ptr [esp]", // the input; no flags touched
        "pushfd",
        "pushad",
        "cld",
        "call {enter}",
        "push esp",             // arg: *mut Regs
        "lea eax, [esp + 40]",
        "push eax",
        "call {keep}",          // the input into INPUT, once the gate is passed
        "add esp, 4",
        "call {dispatch}",
        "add esp, 4",
        "lea eax, [esp + 36]",
        "push eax",
        "call {give}",          // RESULT onto the stack, before the gate is left
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
    INPUT.set(unsafe { at.read_unaligned() });
}

extern "C" fn give(at: *mut [u8; 10]) {
    unsafe { at.write_unaligned(*RESULT.0.get()) };
}

/// The original with `INPUT` in st(0); its st(0) taken off into `RESULT`.
#[unsafe(naked)]
pub(crate) unsafe extern "C" fn original_with_input(_regs: *mut Regs) {
    core::arch::naked_asm!(
        "push dword ptr [esp + 4]",
        "fld tbyte ptr [{i}]",
        "call {co}",
        "add esp, 4",
        "fstp tbyte ptr [{r}]",
        "ret",
        co = sym call_original,
        i = sym INPUT,
        r = sym RESULT,
    )
}

crate::framework::hook_asm!(@call_original call_original, ORIGINAL);

const SCRATCH_RVA: usize = rva(0x62400C); // dword_62400C
const LIMIT_RVA: usize = rva(0x5C5D38); // flt_5C5D38, 1.0

/// The Rust core: the scratch float and the limit at `scratch`, `limit`.
pub unsafe fn clamped_asin_at(x: [u8; 10], scratch: u32, limit: u32) -> [u8; 10] {
    unsafe { clamped_asin_80(x, scratch as *mut u32, limit as *mut u32) }
}

/// The Rust version.
pub unsafe fn clamped_asin(x: [u8; 10]) -> [u8; 10] {
    unsafe {
        clamped_asin_at(
            x,
            game_addr(SCRATCH_RVA) as u32,
            game_addr(LIMIT_RVA) as u32,
        )
    }
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    let x = unsafe { *INPUT.0.get() };
    RESULT.set(unsafe { clamped_asin(x) });
    Ok(Left::nothing())
}

extern "C" fn dispatch(r: &mut Regs) {
    let regions = [
        RESULT.region(),
        (game_addr(SCRATCH_RVA) as u32, 4),
        (game_addr(LIMIT_RVA) as u32, 4),
    ];
    dispatch_state(&HOOK, r, original_with_input, &regions, &rust);
}

/// Runs code at `addr` with the 80-bit `x` in st(0); its st(0) into `out`.
#[unsafe(naked)]
pub unsafe extern "C" fn run_80(_addr: u32, _x: *const [u8; 10], _out: *mut [u8; 10]) {
    core::arch::naked_asm!(
        "mov eax, [esp + 8]",
        "fld tbyte ptr [eax]",
        "call dword ptr [esp + 4]",
        "mov eax, [esp + 12]",
        "fstp tbyte ptr [eax]",
        "ret",
    )
}

/// A copy of the original whose scratch and limit are at `scratch`, `limit`.
pub fn test_copy(scratch: u32, limit: u32) -> Result<crate::redirect::RedirectedCopy, String> {
    crate::audio::test_copy::make(
        &HOOK,
        &crate::audio::test_copy::Copy {
            address: 0x42120B,
            len: 0x77,
            entry: &[0xD9, 0x15, 0x0C, 0x40, 0x62, 0x00],
            calls: &[],
            jumps_out: &[],
        },
        &|_| None,
        &[(0x62400C, scratch), (0x5C5D38, limit)],
        14,
    )
}

/// Test sines: by lot in -1.5 .. 1.5, with odd low bits now and then (not floats), and
/// the edge cases.
pub fn test_sine(seed: u32) -> [u8; 10] {
    match seed % 8 {
        0 => extended(0x3F80_0000 ^ ((seed & 8) << 28)), // +-1
        1 => extended(0x7FC0_0000),                      // NaN
        2 => extended(0x3FC0_0000 ^ ((seed & 8) << 28)), // +-1.5
        _ => {
            let mut f = [0u32; 1];
            test_floats(seed * 5 + 1, &mut f);
            let v = (f32::from_bits(f[0]) % 1.5).to_bits();
            let mut x = extended(v);
            if seed & 1 != 0 {
                x[0] ^= (seed as u8) | 1;
            }
            x
        }
    }
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let mut cells = vec![0u32; 2];
    let scratch = cells.as_mut_ptr() as u32;
    let limit = scratch + 4;
    let copy = match test_copy(scratch, limit) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    let set = |l: u32| unsafe {
        (scratch as *mut [u32; 2]).write_volatile([0xAAAA_AAAA, l]);
    };
    let read = || unsafe { (scratch as *const [u32; 2]).read_volatile() };
    for word in TEST_CONTROL_WORDS {
        with_control_word(word, &mut || {
            for l in [0x3F80_0000u32, 0x3F40_0000, 0xBF80_0000] {
                for seed in 0..24u32 {
                    let x = test_sine(seed);
                    set(l);
                    let mut theirs = [0u8; 10];
                    unsafe { run_80(copy.addr() as u32, &x, &mut theirs) };
                    let their_cells = read();
                    set(l);
                    let mine = unsafe { clamped_asin_at(x, scratch, limit) };
                    let my_cells = read();
                    t.check(theirs == mine && their_cells == my_cells, || {
                        format!(
                            "control word {word:#x}, limit {l:#x}, sine {x:x?}: original {theirs:x?} {their_cells:x?}; rust {mine:x?} {my_cells:x?}"
                        )
                    });
                }
            }
        });
    }
    drop((copy, cells));
    t.finish()
}
