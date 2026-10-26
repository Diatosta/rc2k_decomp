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

use crate::session::MACHINE;
use crate::system::start_texts::START_TEXTS;
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleA;
use windows_sys::Win32::UI::WindowsAndMessaging::FindWindowA;

// ---- game data ----

// ---- Rust version ----

/// True if the game runs already.
pub unsafe fn find_running_game() -> bool {
    unsafe {
        MACHINE
            .instance
            .set(crate::pointers::game_address(GetModuleHandleA(
                std::ptr::null(),
            )));
        !FindWindowA(
            crate::pointers::host(START_TEXTS.class_name.addr()) as *const u8,
            std::ptr::null(),
        )
        .is_null()
    }
}
