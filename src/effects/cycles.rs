//! The processor's cycle counter as the effects' time shares read it. The shares only feed the
//! debug window's effects page, so no counter is read while that window is closed.

/// The counter's low 32 bits, or `None` while the debug window is closed.
#[inline]
pub fn start() -> Option<u32> {
    crate::debug::imgui_sdl::is_open().then(|| crate::platform::arch::rdtsc() as u32)
}

/// The cycles since `begun` (`start`'s), or `None` when it read nothing.
#[inline]
pub fn since(begun: Option<u32>) -> Option<i32> {
    let begun = begun?;
    Some((crate::platform::arch::rdtsc() as u32).wrapping_sub(begun) as i32)
}
