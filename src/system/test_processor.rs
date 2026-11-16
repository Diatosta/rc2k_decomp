//! What the processor is and has: the family, vendor, signature, and the feature bits the game
//! uses (FPU, which `start` requires, TSC, CMOV, MMX, SSE, 3DNow!), through `cpuid`, with a
//! plain FPU test on a processor without it. The clock is no longer timed (a quarter of a
//! second at every start, for the log alone): `cpu_mhz` stays 0.

use crate::memory::session::{Machine, PlainDword};
use rc2k_core::system::processor::{Var, test};

/// Whether toggling `bit` in the flags changes them: yes on every processor these builds can
/// run on, which all have SSE2.
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
/// `arch::rcpss`). The vendor and the signature stay the machine's (the log).
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
    let r = crate::platform::arch::__cpuid(leaf);
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
}

/// Each variable but the FPU's word (the scratch buffer's first bytes) and the FPU flag
/// (`HostShared`, read by the audio thread).
fn cell(m: &Machine, v: Var) -> &PlainDword {
    match v {
        Var::Family => &m.cpu_family,
        Var::Signature => &m.cpu_signature,
        Var::Vendor(n) => &m.cpu_vendor[n],
        Var::HasTsc => &m.has_tsc,
        Var::HasCmov => &m.has_cmov,
        Var::HasMmx => &m.found_mmx,
        Var::HasSse => &m.found_sse,
        Var::Has3dNow => &m.found_3dnow,
        Var::Resume => &m.exception_resume,
        Var::HasFpu => unreachable!("the FPU flag is in `HostShared`"),
        Var::FpuWord => unreachable!("the FPU's word is in the scratch buffer"),
    }
}

pub fn test_processor(p: &dyn Processor, system: &crate::system::owner::System) {
    let mut put = |v: Var, value: u32| {
        if v == Var::FpuWord {
            system.work().files.buffer.set(0, value as u8);
            system.work().files.buffer.set(1, (value >> 8) as u8);
        } else {
            if v == Var::HasFpu {
                system.session.host.has_fpu.set(value);
            } else {
                cell(system.machine(), v).set(value);
            }
        }
    };
    let get = |v: Var| {
        if v == Var::FpuWord {
            system.work().files.buffer.get_u32(0)
        } else {
            if v == Var::HasFpu {
                system.session.host.has_fpu.get()
            } else {
                cell(system.machine(), v).get()
            }
        }
    };
    test(p, &mut put, &get);
}
