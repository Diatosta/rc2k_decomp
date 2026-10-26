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
//! `g_hasFpu`).
//! `start` calls it directly (2026-10-23); the registers above are the original's, for
//! reference.

use crate::arch::{__cpuid, _rdtsc};
use crate::errors::{BadInput, Refused};
use crate::files::scratch::SCRATCH;
use crate::pointers::GamePtr;
use crate::session::MACHINE;
use rc2k_core::system::processor::{Var, test};

// ---- game data ----

// ---- Rust version ----

/// Whether toggling `bit` in the flags changes them (`pushf` .. `popf` .. `pushf`); the
/// flags are put back.
#[cfg(target_arch = "x86_64")]
fn flag_toggles(_bit: u32) -> bool {
    // Every 64-bit processor has CPUID and the alignment flag.
    true
}

#[cfg(target_arch = "x86")]
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
        crate::os::ticks()
    }
    fn cycles(&self) -> u64 {
        unsafe { _rdtsc() }
    }
}

/// Where each variable is.
fn place(v: Var) -> u32 {
    let m = &MACHINE;
    match v {
        Var::Family => m.cpu_family.addr(),
        Var::Signature => m.cpu_signature.addr(),
        Var::Vendor(n) => m.cpu_vendor[n].addr(),
        Var::Clock => m.cpu_mhz.addr(),
        Var::HasFpu => m.has_fpu.addr(),
        Var::HasTsc => m.has_tsc.addr(),
        Var::HasCmov => m.has_cmov.addr(),
        Var::HasMmx => m.found_mmx.addr(),
        Var::HasSse => m.found_sse.addr(),
        Var::Has3dNow => m.found_3dnow.addr(),
        Var::Resume => m.exception_resume.addr(),
        Var::FpuWord => SCRATCH.buffer.addr(),
    }
}

pub fn test_processor(p: &dyn Processor) -> Result<(), Refused> {
    // Converted: `rc2k_core::system::processor::test`.
    // The core's closures cannot refuse: the first refusal is kept and raised after.
    let refused = std::cell::Cell::new(None);
    let keep = |e: BadInput| {
        if refused.get().is_none() {
            refused.set(Some(e));
        }
    };
    let mut put = |v: Var, value: u32| {
        let at = GamePtr::new(place(v));
        let done = if v == Var::FpuWord {
            at.write(value as u16, "a processor test's variable")
        } else {
            at.write(value, "a processor test's variable")
        };
        if let Err(e) = done {
            keep(e);
        }
    };
    let get = |v: Var| {
        GamePtr::new(place(v))
            .read::<u32>("a processor test's variable")
            .unwrap_or_else(|e| {
                keep(e);
                0
            })
    };
    let answer = test(p, &mut put, &get);
    if let Some(e) = refused.take() {
        return Err(e.into());
    }
    answer.map_err(|e| BadInput::from(e).into())
}
