//! `system::find_running_game` (0x4103BE): `hInstance` (0x5184A0) = `GetModuleHandleA(0)`,
//! then whether a window of the game's class ("MF-SDG R5", 0x4E0080) is open already.
//!
//! In:  nothing
//! Out: every register preserved (`pusha` .. `popa`); carry set (and zero clear) if such a
//!      window was found, else carry and zero as `test eax, eax` leaves them (clear, set)
//! One caller, `start` (0x40107E), which leaves at once on the carry. In the game this
//! never gets a call (the hook DLL is loaded later). The Win32 calls are made from Rust (the
//! same in both programs; in the standalone program its own module's handle, and a running
//! game is found as in the game: its window is not of that class).
//! `start` calls it directly (2026-10-23); the registers above are the original's, for
//! reference.
//! Since phase 11 (no Win32): no window of that class is ever made (SDL makes the game's,
//! of its own class), so the original's search found none in the standalone and this says
//! so outright; `hInstance` was read by nothing and is no longer written. A second copy
//! runs, as it did.

/// True if the game runs already: never (see the header).
pub fn find_running_game() -> bool {
    false
}
