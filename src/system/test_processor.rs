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

#![allow(unsafe_code)]

use crate::errors::{BadInput, Refused};
use crate::files::scratch::SCRATCH;
use crate::session::MACHINE;
use rc2k_core::system::processor::{Var, test};

// ---- game data ----

// ---- Rust version ----

/// Whether toggling `bit` in the flags changes them (`pushf` .. `popf` .. `pushf`); the
/// flags are put back.
#[cfg(not(target_arch = "x86"))]
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

/// The FPU test: what an x87 answers (the handheld has none; `math::x87_float` is the model).
#[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
fn fpu_words() -> (u16, u16) {
    (0, 0x037F)
}

/// The FPU test: (the status word after `fninit`, the control word).
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
fn fpu_words() -> (u16, u16) {
    let (mut status, mut control) = (0x5A5Au16, 0u16);
    // SAFETY: the instructions write only the two locals; the x87 state is not otherwise used.
    unsafe {
        core::arch::asm!(
            "fninit",
            "fnstsw word ptr [{s}]",
            "fnstcw word ptr [{c}]",
            s = in(reg) &raw mut status,
            c = in(reg) &raw mut control,
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

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
fn cpuid(leaf: u32) -> (u32, u32, u32, u32) {
    let r = crate::arch::__cpuid(leaf);
    (r.eax, r.ebx, r.ecx, r.edx)
}

/// The handheld's answer: a processor with an FPU, a counter and `cmov`, no SIMD of any kind,
/// so the game takes its plain paths (no x86 there).
#[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
fn cpuid(leaf: u32) -> (u32, u32, u32, u32) {
    match leaf {
        0 => (
            1,
            u32::from_le_bytes(*b"Genu"),
            u32::from_le_bytes(*b"ntel"),
            u32::from_le_bytes(*b"ineI"),
        ),
        1 => (0x680, 0, 0, 1 | 1 << 4 | 1 << 15),
        _ => (0, 0, 0, 0),
    }
}

pub use rc2k_core::system::processor::Processor;

pub struct Real;

impl Processor for Real {
    fn flag_toggles(&self, bit: u32) -> bool {
        flag_toggles(bit)
    }
    fn cpuid(&self, leaf: u32) -> (u32, u32, u32, u32) {
        cpuid(leaf)
    }
    fn fpu_words(&self) -> (u16, u16) {
        fpu_words()
    }
    fn ticks(&self) -> u32 {
        crate::os::ticks()
    }
    fn cycles(&self) -> u64 {
        crate::arch::rdtsc()
    }
}

/// Each variable but the FPU's word (the scratch buffer's first bytes).
fn cell(v: Var) -> &'static crate::session::Dword {
    let m = &MACHINE;
    match v {
        Var::Family => &m.cpu_family,
        Var::Signature => &m.cpu_signature,
        Var::Vendor(n) => &m.cpu_vendor[n],
        Var::Clock => &m.cpu_mhz,
        Var::HasFpu => &m.has_fpu,
        Var::HasTsc => &m.has_tsc,
        Var::HasCmov => &m.has_cmov,
        Var::HasMmx => &m.found_mmx,
        Var::HasSse => &m.found_sse,
        Var::Has3dNow => &m.found_3dnow,
        Var::Resume => &m.exception_resume,
        Var::FpuWord => unreachable!("the FPU's word is in the scratch buffer"),
    }
}

pub fn test_processor(p: &dyn Processor) -> Result<(), Refused> {
    // Converted: `rc2k_core::system::processor::test`.
    let mut put = |v: Var, value: u32| {
        if v == Var::FpuWord {
            SCRATCH.buffer.set(0, value as u8);
            SCRATCH.buffer.set(1, (value >> 8) as u8);
        } else {
            cell(v).set(value);
        }
    };
    let get = |v: Var| {
        if v == Var::FpuWord {
            SCRATCH.buffer.get_u32(0)
        } else {
            cell(v).get()
        }
    };
    let answer = test(p, &mut put, &get);
    answer.map_err(|e| BadInput::from(e).into())
}
