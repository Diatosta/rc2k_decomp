//! `Sleep(milliseconds)`.

/// (`std::thread::sleep`, which is `Sleep` on Windows, 0 a yield as there.)
pub fn sleep(milliseconds: u32) {
    #[cfg(feature = "remote")]
    let wait = std::time::Duration::from_micros(crate::remote::wall_micros(milliseconds));
    #[cfg(not(feature = "remote"))]
    let wait = std::time::Duration::from_millis(milliseconds.into());
    std::thread::sleep(wait);
}
