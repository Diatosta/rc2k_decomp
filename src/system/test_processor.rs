//! `system::test_processor` (0x410883): what the processor is and has, and its clock.
//!
//! `cpu_family` 3; if the flags' AC bit (0x40000) cannot be changed, the FPU test below.
//! Else 4; if the ID bit (0x200000) cannot be changed, or `cpuid` 0's highest leaf is 0,
//! the FPU test. Else the vendor's text goes to `cpu_vendor`; leaf 1's signature to
//! `cpu_signature` and its family ((signature >> 8) & 0xF) to `cpu_family`; from leaf 1's
//! feature bits: bit 0 (FPU) `has_fpu` (which `start` requires) 1; bit 4 (TSC) `has_tsc`
//! 1; bit 15 (CMOV) `has_cmov` 1; bit 23 (MMX) `found_mmx` 1; bit 25 (SSE) `found_sse` 1;
//! "AuthenticAMD" with an extended leaf 0x80000001 whose bit 31 is set (3DNow!)
//! `found_3dnow` 1 ("CyrixInstead" is asked the same and nothing is kept). The FPU test (no
//! `cpuid`): `fninit`, the status word (0x5A5A there before) and, if its low byte is 0, the
//! control word through a scratch place; `& 0x103F` = 0x3F: `has_fpu` 1. Then, if `has_tsc`
//! is 1: from the next change of `GetTickCount` 250 ms are timed by `rdtsc`, and the
//! cycles / 250000 (the clock in MHz) go to `cpu_mhz`. The original executes `rdtsc`,
//! `emms` and `xorps` once each to see that they do not fault, with `exception_resume` the
//! place its exception filter goes on at (0 after each); this port does not execute them
//! (a processor that says it has them has them) and leaves `exception_resume` 0 where the
//! original does. Refuses (with the processor's variables already written) a clock whose
//! quotient does not fit in 32 bits (the original's `div` traps there). One caller,
//! `start`, directly.

use crate::errors::{BadInput, Refused};
use crate::session::{Dword, Machine};
use rc2k_core::system::processor::{Var, test};

// ---- game data ----

// ---- Rust version ----

/// Whether toggling `bit` in the flags changes them (the original's `pushf` .. `popf` test
/// for the alignment flag and CPUID): yes on every processor these builds can run on, which
/// all have SSE2.
fn flag_toggles(_bit: u32) -> bool {
    true
}

/// The FPU test, as an x87 answers it after `fninit`: (the status word, the control word).
/// There is always one: `math::x87_float` is the game's x87.
fn fpu_words() -> (u16, u16) {
    (0, 0x037F)
}

/// The features every machine reports (leaf 1's feature bits): an FPU, a counter, `cmov`, MMX and
/// SSE, as every x86 these builds run on has, and no 3DNow!. The game picks its MMX and
/// SSE paths by them (lighting, polygon depths, the ground's face shares), which compute
/// differently: one answer everywhere, so the handheld, the Mac and every PC run the same
/// arithmetic and replays and network games agree (the estimates themselves are tables,
/// `arch::rcpss`). The vendor, the signature and the clock stay the machine's (the log and
/// the frame rate shown).
const FEATURES: u32 = 1 | 1 << 4 | 1 << 15 | 1 << 23 | 1 << 25;

fn cpuid(leaf: u32) -> (u32, u32, u32, u32) {
    let (a, b, c, d) = machine_cpuid(leaf);
    match leaf {
        1 => (a, b, c, FEATURES),
        0x8000_0001 => (a, b, c, d & !0x8000_0000),
        _ => (a, b, c, d),
    }
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
fn machine_cpuid(leaf: u32) -> (u32, u32, u32, u32) {
    let r = crate::arch::__cpuid(leaf);
    (r.eax, r.ebx, r.ecx, r.edx)
}

/// The handheld's (and an ARM Mac's) answer: an Intel of family 6 (the features are
/// `FEATURES`, as everywhere).
#[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
fn machine_cpuid(leaf: u32) -> (u32, u32, u32, u32) {
    match leaf {
        0 => (
            1,
            u32::from_le_bytes(*b"Genu"),
            u32::from_le_bytes(*b"ntel"),
            u32::from_le_bytes(*b"ineI"),
        ),
        1 => (0x680, 0, 0, 0),
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
fn cell(m: &Machine, v: Var) -> &Dword {
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

pub fn test_processor(
    p: &dyn Processor,
    system: &crate::system::owner::System,
) -> Result<(), Refused> {
    // Converted: `rc2k_core::system::processor::test`.
    let mut put = |v: Var, value: u32| {
        if v == Var::FpuWord {
            system.work.files.buffer.set(0, value as u8);
            system.work.files.buffer.set(1, (value >> 8) as u8);
        } else {
            cell(&system.session.machine, v).set(value);
        }
    };
    let get = |v: Var| {
        if v == Var::FpuWord {
            system.work.files.buffer.get_u32(0)
        } else {
            cell(&system.session.machine, v).get()
        }
    };
    let answer = test(p, &mut put, &get);
    answer.map_err(|e| BadInput::from(e).into())
}
