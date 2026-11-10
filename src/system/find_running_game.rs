//! `system::find_running_game` (0x4103BE): `hInstance` (0x5184A0) = `GetModuleHandleA(0)`,
//! then whether a window of the game's class ("MF-SDG R5", 0x4E0080) is open already.
//!
//! One caller, `start` (0x40107E), which leaves at once on the carry. `start` calls it directly. Since phase
//! 11 (no Win32): no window of that class is ever made (SDL makes the game's, of its own class), so
//! the original's search found none in the standalone and this says so outright; `hInstance` was
//! read by nothing and is no longer written. A second copy runs, as it did.

/// True if the game runs already: never (see the header).
pub fn find_running_game() -> bool {
    false
}
