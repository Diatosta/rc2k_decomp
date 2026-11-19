//! `Sleep(milliseconds)`.

/// (`std::thread::sleep`, which is `Sleep` on Windows, 0 a yield as there.)
pub fn sleep(milliseconds: u32) {
    std::thread::sleep(duration(milliseconds));
}

/// How long `milliseconds` of the game's time last (a remote build's speed scales them).
pub fn duration(milliseconds: u32) -> std::time::Duration {
    std::time::Duration::from_micros(crate::remote::wall_micros(milliseconds))
}
