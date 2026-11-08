//! What differs between the 32-bit build and a 64-bit one (phase 9, step 4): the
//! processor's intrinsics and a thread context's instruction and stack pointers.

#![allow(unsafe_code)]

#[cfg(target_arch = "x86")]
pub use core::arch::x86::{__cpuid, _mm_cvtss_f32, _mm_rcp_ss, _mm_rsqrt_ss, _mm_set_ss, _rdtsc};
#[cfg(target_arch = "x86_64")]
pub use core::arch::x86_64::{
    __cpuid, _mm_cvtss_f32, _mm_rcp_ss, _mm_rsqrt_ss, _mm_set_ss, _rdtsc,
};

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

/// `rcpss`: an approximation on x86, whose result differs between processors; exact elsewhere.
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub fn rcpss(a: f32) -> f32 {
    // SAFETY: SSE is part of every x86 target's baseline.
    unsafe { _mm_cvtss_f32(_mm_rcp_ss(_mm_set_ss(a))) }
}
#[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
pub fn rcpss(a: f32) -> f32 {
    1.0 / a
}

/// `rsqrtss`: an approximation on x86, whose result differs between processors; exact
/// elsewhere.
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub fn rsqrtss(a: f32) -> f32 {
    // SAFETY: SSE is part of every x86 target's baseline.
    unsafe { _mm_cvtss_f32(_mm_rsqrt_ss(_mm_set_ss(a))) }
}
#[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
pub fn rsqrtss(a: f32) -> f32 {
    1.0 / a.sqrt()
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
