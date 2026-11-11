//! `gfx::take_screenshot` (0x407897): with screenshots on (`g_screenshotsOn`), the screen
//! saved as `000.pcx` and on: the surface locked unless it is (`gfx::lock_surface`; a
//! failed lock ends it; `locked_here` 1 while it is this function's lock), written
//! (`gfx::write_screen_pcx`), unlocked again if it was locked here
//! (`gfx::unlock_surface`), `locked_here` 0.
//!
//! Callers: `race::replay_viewer` (key 0x52, Numpad 0), `gfx::screenshot_if_key`,
//! `menu::handle_keys`, `install::page_key`, `net::settings_loop`, `net::choose_connection`,
//! `net::choose_session`,
//! `net::lobby_loop`, 0x471CA6. The lock and unlock (the platform's) and the writer are
//! called directly.

use crate::errors::Refused;

use crate::gfx::locked::Locked;
use crate::gfx::shots::{Shots, on};

/// The parts: the game's memory, the platform's lock and unlock, the writer.
struct Shot<'a> {
    shots: &'a Shots,
    locked: &'a Locked,
}

impl rc2k_core::gfx::screenshot::Shooting for Shot<'_> {
    type Error = Refused;
    fn screenshots_on(&mut self) -> u32 {
        on()
    }
    fn surface_locked(&mut self) -> u32 {
        self.locked.surface.get()
    }
    fn locked_here(&mut self) -> u32 {
        self.shots.locked_here.get()
    }
    fn set_locked_here(&mut self, value: u32) {
        self.shots.locked_here.set(value);
    }
    fn lock(&mut self) -> Result<bool, Refused> {
        // `cmp g_surfaceLocked, 1`'s carry: not locked.
        Ok(crate::platform::lock_surface() < 1)
    }
    fn write(&mut self) -> Result<(), Refused> {
        let gfx = crate::gfx::owner::gfx();
        if let Some(screen) = crate::platform_3d::screen() {
            crate::render::bring_back(&screen);
        }
        crate::gfx::write_screen_pcx::write_screen_pcx(self.shots, None, gfx)?;
        Ok(())
    }
    fn unlock(&mut self) -> Result<(), Refused> {
        crate::platform::unlock_surface();
        Ok(())
    }
}
pub fn take_screenshot() -> Result<(), Refused> {
    let gfx = crate::gfx::owner::gfx();
    // Converted: `rc2k_core::gfx::screenshot::take_screenshot`.
    rc2k_core::gfx::screenshot::take_screenshot(&mut Shot {
        shots: &gfx.shots,
        locked: &gfx.locked,
    })
}
