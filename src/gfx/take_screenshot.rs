//! `gfx::take_screenshot`: with screenshots on, the screen saved as `000.pcx` and on: the surface
//! locked unless it is (`gfx::lock_surface`; a failed lock ends it; `locked_here` 1 while it is
//! this function's lock), written (`gfx::write_screen_pcx`), unlocked again if it was locked here
//! (`gfx::unlock_surface`), `locked_here` 0. The lock and unlock (the platform's) and the writer
//! are called directly.

use crate::errors::Refused;

use crate::gfx::owner::Gfx;

/// The parts: the game's memory, the platform's lock and unlock, the writer.
struct Shot<'a> {
    gfx: &'a Gfx,
    system: &'a crate::system::owner::System,
}

impl rc2k_core::gfx::screenshot::Shooting for Shot<'_> {
    type Error = Refused;
    fn screenshots_on(&mut self) -> u32 {
        self.gfx.shots.on.get()
    }
    fn surface_locked(&mut self) -> u32 {
        self.gfx.locked.surface.get()
    }
    fn locked_here(&mut self) -> u32 {
        self.gfx.shots.locked_here.get()
    }
    fn set_locked_here(&mut self, value: u32) {
        self.gfx.shots.locked_here.set(value);
    }
    fn lock(&mut self) -> Result<bool, Refused> {
        // Not locked.
        Ok(crate::platform::lock_surface(self.gfx, self.system) < 1)
    }
    fn write(&mut self) -> Result<(), Refused> {
        if let Some(screen) = crate::platform_3d::screen(self.system.session.screen.get()) {
            crate::render::bring_back(&screen);
        }
        crate::gfx::write_screen_pcx::write_screen_pcx(
            &self.gfx.shots,
            None,
            self.gfx,
            self.system,
        )?;
        Ok(())
    }
    fn unlock(&mut self) -> Result<(), Refused> {
        crate::platform::unlock_surface(self.gfx);
        Ok(())
    }
}
pub fn take_screenshot(
    gfx: &crate::gfx::owner::Gfx,
    system: &crate::system::owner::System,
) -> Result<(), Refused> {
    rc2k_core::gfx::screenshot::take_screenshot(&mut Shot { gfx, system })
}
