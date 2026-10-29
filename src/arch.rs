//! What differs between the 32-bit build and a 64-bit one (phase 9, step 4): the
//! processor's intrinsics and a thread context's instruction and stack pointers.

#[cfg(target_arch = "x86")]
pub use core::arch::x86::{__cpuid, _mm_cvtss_f32, _mm_mul_ss, _mm_rcp_ss, _mm_set_ss, _rdtsc};
#[cfg(target_arch = "x86_64")]
pub use core::arch::x86_64::{__cpuid, _mm_cvtss_f32, _mm_mul_ss, _mm_rcp_ss, _mm_set_ss, _rdtsc};

/// The processor's time-stamp counter.
pub fn rdtsc() -> u64 {
    // SAFETY: `rdtsc` only reads the counter; every x86 the game builds for has it.
    unsafe { _rdtsc() }
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
