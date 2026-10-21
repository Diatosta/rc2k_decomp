//! `system::test_processor` (0x410883): what the processor is and has, and its clock.
//!
//! In:  nothing
//! Out: every register preserved (`pusha` .. `popa`)
//! `dword_5183D0` 3; if the flags' AC bit (0x40000) cannot be changed, the FPU test below.
//! Else 4; if the ID bit (0x200000) cannot be changed, or `cpuid` 0's highest leaf is 0,
//! the FPU test. Else the vendor's text (ebx, edx, ecx) to 0x5183D8 .. 0x5183E0; leaf 1's
//! eax to `dword_5183D4` and its family ((eax >> 8) & 0xF) to `dword_5183D0`; from its
//! edx: bit 0 (FPU) `g_hasFpu` (0x5183EC, which `start`
//! requires) 1; bit 4 (TSC) `dword_5183F0` 1; bit 15 (CMOV) `dword_5183F4` 1; bit 23 (MMX)
//! `dword_5183F8` 1; bit 25 (SSE) `dword_5183FC` 1; "AuthenticAMD" with an extended leaf
//! 0x80000001 whose edx has bit 31 (3DNow!) `dword_518400` 1 ("CyrixInstead" is asked the
//! same and nothing is kept). The FPU test (no `cpuid`): `fninit`, the status word (0x5A5A
//! there before) and, if its low byte is 0, the control word through the word at 0x93E8B0
//! (`g_pngInfo`'s, a scratch place here); `& 0x103F` = 0x3F: `g_hasFpu` 1. Then, if
//! `dword_5183F0` is 1: from the next change of `GetTickCount` 250 ms are timed by `rdtsc`,
//! and the cycles / 250000 (the clock in MHz) go to `dword_5183E8`.
//! The original executes `rdtsc`, `emms` and `xorps` once each to see that they do not
//! fault, with `dword_51849C` the place its exception filter goes on at (0 after each); the
//! Rust version does not execute them (a processor that says it has them has them) and
//! leaves `dword_51849C` 0 where the original does. Refuses (with the processor's variables
//! already written) a clock whose quotient does not fit in 32 bits (the original's `div`
//! traps there). One caller, `start` (0x40101A); in the game this never gets a call (the
//! hook DLL is loaded later); the standalone program needs it (it used a stand-in for
//! `g_hasFpu`). Shadow mode runs the original alone (it takes a quarter of a second).
//! Self-test: the original and the Rust version from zeros and from 0xAA, every variable
//! compared, the clock within 5%.

use crate::errors::{BadInput, Refused};
use crate::framework::{
    RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva, shadow,
};
use crate::service::harness::{Region, capture, restore};
use core::arch::x86::{__cpuid, _rdtsc};
use rc2k_core::system::processor::{Var, test};
use std::sync::atomic::AtomicUsize;
use windows_sys::Win32::System::SystemInformation::GetTickCount;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("system::test_processor", rva(0x410883), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

const FAMILY_RVA: usize = rva(0x5183D0); // dword_5183D0
const SIGNATURE_RVA: usize = rva(0x5183D4); // dword_5183D4: leaf 1's eax
const VENDOR_RVA: usize = rva(0x5183D8); // dword_5183D8 .. dword_5183E0
const CLOCK_RVA: usize = rva(0x5183E8); // dword_5183E8: MHz
const HAS_FPU_RVA: usize = rva(0x5183EC); // g_hasFpu
const HAS_TSC_RVA: usize = rva(0x5183F0); // dword_5183F0
const HAS_CMOV_RVA: usize = rva(0x5183F4); // dword_5183F4
const HAS_MMX_RVA: usize = rva(0x5183F8); // dword_5183F8
const HAS_SSE_RVA: usize = rva(0x5183FC); // dword_5183FC
const HAS_3DNOW_RVA: usize = rva(0x518400); // dword_518400
const RESUME_RVA: usize = rva(0x51849C); // dword_51849C: where the filter goes on
const FPU_WORD_RVA: usize = rva(0x93E8B0); // g_pngInfo's first word
/// The variables from `dword_5183D0` to `dword_518400`.
const VARS_LEN: usize = 0x34;

// ---- Rust version ----

/// Whether toggling `bit` in the flags changes them (`pushf` .. `popf` .. `pushf`); the
/// flags are put back.
fn flag_toggles(bit: u32) -> bool {
    let changed: u32;
    unsafe {
        core::arch::asm!(
            "pushfd",
            "pop {was}",
            "mov {now}, {was}",
            "xor {now}, {bit}",
            "push {now}",
            "popfd",
            "pushfd",
            "pop {now}",
            "xor {now}, {was}",
            "push {was}",
            "popfd",
            was = out(reg) _,
            now = out(reg) changed,
            bit = in(reg) bit,
        );
    }
    changed != 0
}

/// The FPU test: (the status word after `fninit`, the control word).
fn fpu_words() -> (u16, u16) {
    let (mut status, mut control) = (0x5A5Au16, 0u16);
    unsafe {
        core::arch::asm!(
            "fninit",
            "fnstsw word ptr [{s}]",
            "fnstcw word ptr [{c}]",
            s = in(reg) &mut status,
            c = in(reg) &mut control,
            // `fninit` empties the x87 stack.
            out("st(0)") _,
            out("st(1)") _,
            out("st(2)") _,
            out("st(3)") _,
            out("st(4)") _,
            out("st(5)") _,
            out("st(6)") _,
            out("st(7)") _,
        );
    }
    (status, control)
}

pub use rc2k_core::system::processor::Processor;

pub struct Real;

impl Processor for Real {
    fn flag_toggles(&self, bit: u32) -> bool {
        flag_toggles(bit)
    }
    fn cpuid(&self, leaf: u32) -> (u32, u32, u32, u32) {
        let r = __cpuid(leaf);
        (r.eax, r.ebx, r.ecx, r.edx)
    }
    fn fpu_words(&self) -> (u16, u16) {
        fpu_words()
    }
    fn ticks(&self) -> u32 {
        unsafe { GetTickCount() }
    }
    fn cycles(&self) -> u64 {
        unsafe { _rdtsc() }
    }
}

/// Where each variable is.
fn place(v: Var) -> usize {
    match v {
        Var::Family => FAMILY_RVA,
        Var::Signature => SIGNATURE_RVA,
        Var::Vendor(n) => VENDOR_RVA + 4 * n,
        Var::Clock => CLOCK_RVA,
        Var::HasFpu => HAS_FPU_RVA,
        Var::HasTsc => HAS_TSC_RVA,
        Var::HasCmov => HAS_CMOV_RVA,
        Var::HasMmx => HAS_MMX_RVA,
        Var::HasSse => HAS_SSE_RVA,
        Var::Has3dNow => HAS_3DNOW_RVA,
        Var::Resume => RESUME_RVA,
        Var::FpuWord => FPU_WORD_RVA,
    }
}

pub unsafe fn test_processor(p: &dyn Processor) -> Result<(), Refused> {
    // Converted: `rc2k_core::system::processor::test`.
    let mut put = |v: Var, value: u32| unsafe {
        if v == Var::FpuWord {
            game_write(place(v), value as u16);
        } else {
            game_write(place(v), value);
        }
    };
    let get = |v: Var| unsafe { game_read::<u32>(place(v)) };
    test(p, &mut put, &get).map_err(|e| BadInput::from(e).into())
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    if let Err(e) = unsafe { test_processor(&Real) } {
        unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) };
    }
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let regions: [Region; 3] = [
        (game_addr(FAMILY_RVA) as u32, VARS_LEN),
        (game_addr(RESUME_RVA) as u32, 4),
        (game_addr(FPU_WORD_RVA) as u32, 2),
    ];
    let saved = capture(&regions);
    let len = VARS_LEN + 6;
    for fill in [0u8, 0xAA] {
        restore(&regions, &vec![fill; len]);
        let before = Regs::SENTINEL;
        let mut ro = before;
        unsafe { call_original(&mut ro) };
        let preserved = ro.preserved_except(&before, 0);
        let theirs = capture(&regions);
        restore(&regions, &vec![fill; len]);
        let done = unsafe { test_processor(&Real) };
        let mine = capture(&regions);
        // The clock (+0x18) within 5% (the ticks are coarse); everything else byte for byte.
        let clock = |b: &[u8]| u32::from_le_bytes(b[0x18..0x1C].try_into().unwrap());
        let (a, b) = (clock(&theirs), clock(&mine));
        let close = a.abs_diff(b) as u64 * 100 <= a.max(b) as u64 * 5;
        let rest = |v: &[u8]| [&v[..0x18], &v[0x1C..]].concat();
        t.check(
            preserved && done.is_ok() && close && rest(&theirs) == rest(&mine),
            || {
                format!(
                    "fill {fill:#x}: original preserved={preserved} {theirs:02x?}; rust {done:?} {mine:02x?}"
                )
            },
        );
    }
    restore(&regions, &saved);
    t.finish()
}
