//! `sound::open_cd_audio` (0x40F9DE): the CD audio's opening, cut out of this build.
//!
//! In:  eax = anything
//! Out: every register preserved (`pusha` .. `popa`); carry clear, zero set if eax is 0
//!      (`test eax, eax` on the entry's eax)
//! The function begins `pusha; jmp` to its own end: the opening behind the jump ("CDA: Open
//! cd audio", `mciSendCommandA` open (0x803) and set (0x80D) on a CD audio device into
//! `mciId`, `dword_518108` 1, `sub_40FDE4` the volume, the time in `dword_518114`; failing:
//! "CDA: Cannot open CD audio .." and `sub_40FA8B`) is never reached, so `dword_518108`
//! stays 0 and the CD audio functions that test it do nothing. One caller, `start`
//! (0x4010E4), which reads no flag. Manual patch with a private copy (0xAD bytes, the seven
//! calls relocated): the `jmp` at +1 is in the first bytes. Shadow mode: both, compared.
//! Self-test: the original and the Rust version on eax 0 and not 0, the flags and the CD
//! audio's variables (nothing written).

use crate::framework::{RawHook, Regs, SelfTest, ZERO_FLAG, game_addr, hook_asm, rva, shadow};
use crate::service::harness::{Region, capture};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new("sound::open_cd_audio", rva(0x40F9DE), thunk, &ORIGINAL)
    .private_copy(0xAD)
    .relocate_calls(&[0x0D, 0x40, 0x7C, 0x85, 0x8A, 0x9F, 0xA5])
    .manual_patch()
    .self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

/// `dword_518104` .. `dword_518134`: the CD audio's variables the opening would write.
const CD_VARS_RVA: usize = rva(0x518104);
const CD_VARS_LEN: usize = 0x34;

// ---- Rust version ----

/// (carry, zero) for the entry's eax.
pub fn open_cd_audio(eax: u32) -> (bool, bool) {
    (false, eax == 0)
}

// ---- hook ----

fn flags(r: &Regs) -> (bool, bool) {
    (r.carry(), r.eflags & ZERO_FLAG != 0)
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    let mine = open_cd_audio(r.eax);
    if !shadow() {
        r.set_carry(mine.0);
        r.set_zero(mine.1);
        return;
    }
    let mut o = *r;
    unsafe { call_original(&mut o) };
    if flags(&o) != mine || !o.preserved_except(r, 0) {
        HOOK.mismatch(|| format!("eax={:#x}: original {:?}, rust {mine:?}", r.eax, flags(&o)));
    }
    r.eflags = o.eflags;
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let regions: [Region; 1] = [(game_addr(CD_VARS_RVA) as u32, CD_VARS_LEN)];
    for eax in [0u32, 1, 0x8000_0000] {
        let before_vars = capture(&regions);
        let before = Regs {
            eax,
            eflags: 0x41,
            ..Regs::SENTINEL
        };
        let mut ro = before;
        unsafe { call_original(&mut ro) };
        let after_vars = capture(&regions);
        let mine = open_cd_audio(eax);
        let preserved = ro.preserved_except(&before, 0);
        t.check(
            preserved && flags(&ro) == mine && before_vars == after_vars,
            || {
                format!(
                    "eax={eax:#x}: original {:?} preserved={preserved} wrote={}; rust {mine:?}",
                    flags(&ro),
                    before_vars != after_vars
                )
            },
        );
    }
    t.finish()
}
