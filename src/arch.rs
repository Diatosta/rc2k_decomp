//! What differs between the 32-bit build and a 64-bit one: the
//! processor's intrinsics and a thread context's instruction and stack pointers.

#![allow(unsafe_code)]

#[cfg(target_arch = "x86")]
pub use core::arch::x86::{__cpuid, _rdtsc};
#[cfg(target_arch = "x86_64")]
pub use core::arch::x86_64::{__cpuid, _rdtsc};

/// The processor's time-stamp counter.
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub fn rdtsc() -> u64 {
    // SAFETY: `rdtsc` only reads the counter; every x86 the game builds for has it.
    unsafe { _rdtsc() }
}

/// The handheld has no such counter: the clock at 1.5 GHz, what its processor runs at.
#[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
pub fn rdtsc() -> u64 {
    static START: std::sync::OnceLock<std::time::Instant> = std::sync::OnceLock::new();
    START
        .get_or_init(std::time::Instant::now)
        .elapsed()
        .as_nanos() as u64
        * 3
        / 2
}

/// `rcpss`: the 5800X's estimate on every machine (the processors' own differ).
pub fn rcpss(a: f32) -> f32 {
    rc2k_core::math::estimates::reciprocal_estimate(a)
}

/// `rsqrtss`: the 5800X's estimate on every machine.
pub fn rsqrtss(a: f32) -> f32 {
    rc2k_core::math::estimates::reciprocal_root_estimate(a)
}

/// Whether this processor's own `rcpss` / `rsqrtss` give the tables' results (on the
/// octaves the tables were made from): the tests that run the original's instructions
/// compare with the tables only where they do.
#[cfg(all(test, any(target_arch = "x86", target_arch = "x86_64")))]
pub fn estimates_are_the_processors() -> bool {
    #[cfg(target_arch = "x86")]
    use core::arch::x86::{_mm_cvtss_f32, _mm_rcp_ss, _mm_rsqrt_ss, _mm_set_ss};
    #[cfg(target_arch = "x86_64")]
    use core::arch::x86_64::{_mm_cvtss_f32, _mm_rcp_ss, _mm_rsqrt_ss, _mm_set_ss};
    (0..4096u32).all(|i| {
        [127u32, 128].iter().all(|&e| {
            let a = f32::from_bits((e << 23) | (i << 11));
            // SAFETY: SSE is part of every x86 target's baseline.
            let (r, s) = unsafe {
                (
                    _mm_cvtss_f32(_mm_rcp_ss(_mm_set_ss(a))),
                    _mm_cvtss_f32(_mm_rsqrt_ss(_mm_set_ss(a))),
                )
            };
            r.to_bits() == rcpss(a).to_bits() && s.to_bits() == rsqrtss(a).to_bits()
        })
    })
}

#[cfg(windows)]
use windows_sys::Win32::System::Diagnostics::Debug::CONTEXT;

/// The context flag for the control registers (`GetThreadContext`).
#[cfg(all(windows, target_arch = "x86"))]
pub const CONTEXT_CONTROL: u32 =
    windows_sys::Win32::System::Diagnostics::Debug::CONTEXT_CONTROL_X86;
#[cfg(all(windows, target_arch = "x86_64"))]
pub const CONTEXT_CONTROL: u32 =
    windows_sys::Win32::System::Diagnostics::Debug::CONTEXT_CONTROL_AMD64;

/// A thread context's instruction pointer.
#[cfg(windows)]
pub fn ip(c: &CONTEXT) -> usize {
    #[cfg(target_arch = "x86")]
    return c.Eip as usize;
    #[cfg(target_arch = "x86_64")]
    return c.Rip as usize;
}

/// A thread context's stack pointer.
#[cfg(windows)]
pub fn sp(c: &CONTEXT) -> usize {
    #[cfg(target_arch = "x86")]
    return c.Esp as usize;
    #[cfg(target_arch = "x86_64")]
    return c.Rsp as usize;
}
