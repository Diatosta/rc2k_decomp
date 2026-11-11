//! What the processor is and has, and its clock (from
//! `system::test_processor` 0x410883).
//!
//! The family is 3 when the flags' AC bit cannot be changed, 4 when the ID bit cannot (or
//! `cpuid` has no leaf 1), else `cpuid`'s; its leaf 1 says which of the FPU, the time-stamp
//! counter, CMOV, MMX and SSE are there, and an AMD's extended leaf 0x80000001 3DNow!.
//! Without `cpuid` the FPU is looked for by its words after `fninit`. With a time-stamp
//! counter, the cycles of 250 ms of `GetTickCount` (from its next change) give the clock in
//! MHz. Each found feature is set to 1; nothing is cleared.

use crate::Invalid;

/// The flags' alignment check bit and ID bit.
pub const AC_FLAG: u32 = 0x4_0000;
pub const ID_FLAG: u32 = 0x20_0000;
/// The timed span, in milliseconds, and its cycles per MHz.
pub const SPAN: u32 = 250;
pub const PER_MHZ: u64 = 250_000;
/// The vendors' texts as `cpuid` 0's ebx, edx, ecx.
pub const AMD: [u32; 3] = [0x6874_7541, 0x6974_6E65, 0x444D_4163];
pub const CYRIX: [u32; 3] = [0x6972_7943, 0x736E_4978, 0x6461_6574];

/// What the processor answers (the platform's, or a test's).
pub trait Processor {
    /// Whether toggling `bit` in the flags changes them.
    fn flag_toggles(&self, bit: u32) -> bool;
    /// `cpuid` leaf ->.
    fn cpuid(&self, leaf: u32) -> (u32, u32, u32, u32);
    /// The status word after `fninit` (0x5A5A there before) and the control word.
    fn fpu_words(&self) -> (u16, u16);
    fn ticks(&self) -> u32;
    fn cycles(&self) -> u64;
}

/// The game's variables the test writes and reads.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Var {
    Family,
    /// Leaf 1's first result word.
    Signature,
    /// The vendor's text, dword 0 .. 2.
    Vendor(usize),
    /// MHz.
    Clock,
    HasFpu,
    HasTsc,
    HasCmov,
    HasMmx,
    HasSse,
    Has3dNow,
    /// Where the exception filter goes on (0 after each instruction tried).
    Resume,
    /// A scratch word the FPU's words go through (a word, not a dword).
    FpuWord,
}

/// The test, in the original's order: `put` writes a variable, `get` reads one back
/// (the vendor and the counter's flag are read where the original reads them). Refuses
/// (with the processor's variables already written) a clock whose quotient does not fit in
/// 32 bits (the original's `div` traps).
pub fn test(
    p: &dyn Processor,
    put: &mut dyn FnMut(Var, u32),
    get: &dyn Fn(Var) -> u32,
) -> Result<(), Invalid> {
    if !test_cpuid(p, put, get) {
        test_fpu(p, put);
    }
    if get(Var::HasTsc) != 1 {
        return Ok(());
    }
    let first = p.ticks();
    let mut now = p.ticks();
    while now == first {
        now = p.ticks();
    }
    let end = now.wrapping_add(SPAN);
    let start = p.cycles();
    while p.ticks() < end {}
    let cycles = p.cycles().wrapping_sub(start);
    let mhz = u32::try_from(cycles / PER_MHZ)
        .map_err(|_| Invalid("a clock past 32 bits (the original's div traps)"))?;
    put(Var::Clock, mhz);
    Ok(())
}

/// The FPU test (no `cpuid`).
fn test_fpu(p: &dyn Processor, put: &mut dyn FnMut(Var, u32)) {
    let (status, control) = p.fpu_words();
    put(Var::FpuWord, status as u32);
    if status as u8 != 0 {
        return;
    }
    put(Var::FpuWord, control as u32);
    if control & 0x103F == 0x3F {
        put(Var::HasFpu, 1);
    }
}

/// `cpuid` and what follows from it; false where the original goes to the FPU test.
fn test_cpuid(p: &dyn Processor, put: &mut dyn FnMut(Var, u32), get: &dyn Fn(Var) -> u32) -> bool {
    put(Var::Family, 3);
    if !p.flag_toggles(AC_FLAG) {
        return false;
    }
    put(Var::Family, 4);
    if !p.flag_toggles(ID_FLAG) {
        return false;
    }
    let (highest, b, c, d) = p.cpuid(0);
    if highest < 1 {
        return false;
    }
    put(Var::Vendor(0), b);
    put(Var::Vendor(1), d);
    put(Var::Vendor(2), c);
    let (signature, _, _, features) = p.cpuid(1);
    put(Var::Signature, signature);
    put(Var::Family, (signature >> 8) & 0xF);
    // Each with the instruction the original tries first (the filter's place 0 after it).
    for (bit, var, tried) in [
        (1, Var::HasFpu, false),
        (0x10, Var::HasTsc, true),
        (0x8000, Var::HasCmov, false),
        (0x80_0000, Var::HasMmx, true),
        (0x200_0000, Var::HasSse, true),
    ] {
        if features & bit != 0 {
            if tried {
                put(Var::Resume, 0);
            }
            put(var, 1);
        }
    }
    let vendor = [0, 1, 2].map(|n| get(Var::Vendor(n)));
    if vendor == AMD && p.cpuid(0x8000_0000).0 != 0 && p.cpuid(0x8000_0001).3 & 0x8000_0000 != 0 {
        put(Var::Has3dNow, 1);
    }
    // "CyrixInstead": asked, nothing kept.
    if vendor == CYRIX && p.cpuid(0x8000_0000).0 > 0x8000_0000 {
        let _ = p.cpuid(0x8000_0001);
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;
    use std::collections::HashMap;

    struct Fake {
        toggles: [bool; 2],
        leaves: HashMap<u32, (u32, u32, u32, u32)>,
        fpu: (u16, u16),
        tick: Cell<u32>,
        cycle: Cell<u64>,
    }

    impl Processor for Fake {
        fn flag_toggles(&self, bit: u32) -> bool {
            self.toggles[(bit == ID_FLAG) as usize]
        }
        fn cpuid(&self, leaf: u32) -> (u32, u32, u32, u32) {
            self.leaves.get(&leaf).copied().unwrap_or_default()
        }
        fn fpu_words(&self) -> (u16, u16) {
            self.fpu
        }
        fn ticks(&self) -> u32 {
            self.tick.set(self.tick.get() + 1);
            self.tick.get() / 4
        }
        fn cycles(&self) -> u64 {
            self.cycle.set(self.cycle.get() + 500_000_000);
            self.cycle.get()
        }
    }

    fn run(f: &Fake) -> (Result<(), Invalid>, Vec<(Var, u32)>) {
        let vars = std::cell::RefCell::new(Vec::new());
        let got = test(f, &mut |v, x| vars.borrow_mut().push((v, x)), &|v| {
            vars.borrow()
                .iter()
                .rev()
                .find(|w| w.0 == v)
                .map_or(0, |w| w.1)
        });
        (got, vars.into_inner())
    }

    #[test]
    fn amd_with_everything() {
        let mut leaves = HashMap::new();
        leaves.insert(0, (1, AMD[0], AMD[2], AMD[1]));
        leaves.insert(1, (0x0000_0612, 0, 0, 0x0280_8011));
        leaves.insert(0x8000_0000, (0x8000_0001, 0, 0, 0));
        leaves.insert(0x8000_0001, (0, 0, 0, 0x8000_0000));
        let f = Fake {
            toggles: [true, true],
            leaves,
            fpu: (0, 0),
            tick: Cell::new(0),
            cycle: Cell::new(0),
        };
        let (got, vars) = run(&f);
        assert_eq!(got, Ok(()));
        assert!(vars.contains(&(Var::Family, 6)));
        assert!(vars.contains(&(Var::Has3dNow, 1)));
        assert!(vars.contains(&(Var::HasSse, 1)));
        assert_eq!(vars.last(), Some(&(Var::Clock, 2000)));
    }

    #[test]
    fn no_cpuid() {
        let f = Fake {
            toggles: [false, false],
            leaves: HashMap::new(),
            fpu: (0, 0x37F),
            tick: Cell::new(0),
            cycle: Cell::new(0),
        };
        let (_, vars) = run(&f);
        assert_eq!(
            vars,
            [
                (Var::Family, 3),
                (Var::FpuWord, 0),
                (Var::FpuWord, 0x37F),
                (Var::HasFpu, 1)
            ]
        );
        let f = Fake {
            fpu: (0x5A5A, 0),
            toggles: [true, false],
            ..f
        };
        assert_eq!(
            run(&f).1,
            [(Var::Family, 3), (Var::Family, 4), (Var::FpuWord, 0x5A5A)]
        );
    }
}
