//! What the processor is and has, and its clock.
//!
//! The family is 3 when the flags' AC bit cannot be changed, 4 when the ID bit cannot (or
//! `cpuid` has no leaf 1), else `cpuid`'s; leaf 1 says which of the FPU, time-stamp counter,
//! CMOV, MMX and SSE are there, and an AMD's leaf 0x80000001 3DNow!. Without `cpuid` the FPU
//! is looked for by its words after `fninit`. Each found feature is set to 1; nothing is
//! cleared. (The original also timed the clock in MHz for 250 ms at every start; nothing needs
//! it: the frame rate is measured with a clock of its own.)

/// The flags' alignment check bit and ID bit.
pub const AC_FLAG: u32 = 0x4_0000;
pub const ID_FLAG: u32 = 0x20_0000;
/// The vendors' texts as `cpuid` 0's ebx, edx, ecx.
pub const AMD: [u32; 3] = [0x6874_7541, 0x6974_6E65, 0x444D_4163];
pub const CYRIX: [u32; 3] = [0x6972_7943, 0x736E_4978, 0x6461_6574];

/// What the processor answers (the platform's, or a test's).
pub trait Processor {
    /// Whether toggling `bit` in the flags changes them.
    fn flag_toggles(&self, bit: u32) -> bool;
    /// The four result words (eax, ebx, ecx, edx) of `cpuid` leaf `leaf`.
    fn cpuid(&self, leaf: u32) -> (u32, u32, u32, u32);
    /// The status word after `fninit` (0x5A5A there before) and the control word.
    fn fpu_words(&self) -> (u16, u16);
}

/// The game's variables the test writes and reads.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Var {
    Family,
    /// Leaf 1's first result word.
    Signature,
    /// The vendor's text, dword 0 .. 2.
    Vendor(usize),
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

/// The test: `put` writes a variable, `get` reads one back.
pub fn test(p: &dyn Processor, put: &mut dyn FnMut(Var, u32), get: &dyn Fn(Var) -> u32) {
    if !test_cpuid(p, put, get) {
        test_fpu(p, put);
    }
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

/// `cpuid` and what follows from it; false where the FPU test is needed instead.
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
    // Each feature, with whether its instruction is tried (the filter's place 0 after it).
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
    use std::collections::HashMap;

    struct Fake {
        toggles: [bool; 2],
        leaves: HashMap<u32, (u32, u32, u32, u32)>,
        fpu: (u16, u16),
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
    }

    fn run(f: &Fake) -> Vec<(Var, u32)> {
        let vars = std::cell::RefCell::new(Vec::new());
        test(f, &mut |v, x| vars.borrow_mut().push((v, x)), &|v| {
            vars.borrow()
                .iter()
                .rev()
                .find(|w| w.0 == v)
                .map_or(0, |w| w.1)
        });
        vars.into_inner()
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
        };
        let vars = run(&f);
        assert!(vars.contains(&(Var::Family, 6)));
        assert!(vars.contains(&(Var::Has3dNow, 1)));
        assert!(vars.contains(&(Var::HasSse, 1)));
    }

    #[test]
    fn no_cpuid() {
        let f = Fake {
            toggles: [false, false],
            leaves: HashMap::new(),
            fpu: (0, 0x37F),
        };
        let vars = run(&f);
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
            run(&f),
            [(Var::Family, 3), (Var::Family, 4), (Var::FpuWord, 0x5A5A)]
        );
    }
}
