//! `gfx::take_screenshot` (0x407897): with screenshots on (`g_screenshotsOn`), the screen
//! saved as `000.pcx` and on: the surface locked unless it is (`gfx::lock_surface`; a
//! failed lock ends it; `dword_4F52B8` 1 while it is this function's lock), written
//! (`gfx::write_screen_pcx`), unlocked again if it was locked here
//! (`gfx::unlock_surface`), `dword_4F52B8` 0.
//!
//! In:  nothing
//! Out: every register kept (`pusha` / `popa`); no flags read
//!
//! Callers: `race::replay_viewer` (key 0x52, Numpad 0), `gfx::screenshot_if_key`,
//! `menu::handle_keys`, `sub_4146E4`, `sub_4168CF`, `sub_4174C5`, `sub_417726`,
//! `sub_417A0D`, 0x471CA6. The lock and unlock (the platform's) and the writer are
//! called directly (2026-10-23).

use crate::errors::Refused;

const LOCKED: LockSlot = LockSlot::at(0x4F3870); // g_surfaceLocked
use crate::gfx::locked::Slot as LockSlot;
use crate::gfx::shots::{LOCKED_HERE, on};

/// The parts: the game's memory, the platform's lock and unlock, the writer.
struct Shot;

impl rc2k_core::gfx::screenshot::Shooting for Shot {
    type Error = Refused;
    fn screenshots_on(&mut self) -> u32 {
        on()
    }
    fn surface_locked(&mut self) -> u32 {
        unsafe { crate::gfx::block::game_read(LOCKED) }
    }
    fn locked_here(&mut self) -> u32 {
        unsafe { crate::gfx::block::game_read(LOCKED_HERE) }
    }
    fn set_locked_here(&mut self, value: u32) {
        unsafe { crate::gfx::block::game_write(LOCKED_HERE, value) };
    }
    fn lock(&mut self) -> Result<bool, Refused> {
        // `cmp g_surfaceLocked, 1`'s carry: not locked.
        Ok(crate::platform::lock_surface() < 1)
    }
    fn write(&mut self) -> Result<(), Refused> {
        unsafe { crate::gfx::write_screen_pcx::write_screen_pcx() }?;
        Ok(())
    }
    fn unlock(&mut self) -> Result<(), Refused> {
        crate::platform::unlock_surface();
        Ok(())
    }
}

/// The Rust version (see the header).
pub fn take_screenshot() -> Result<(), Refused> {
    // Converted: `rc2k_core::gfx::screenshot::take_screenshot`.
    rc2k_core::gfx::screenshot::take_screenshot(&mut Shot)
}
