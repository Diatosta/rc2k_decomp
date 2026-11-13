//! With a front surface, locks the screen (`gfx::lock_surface`; failing, nothing), zeroes its rows
//! (the screen's height of its row dwords from `present_pixels`, `present_gap` bytes between them)
//! and unlocks it (`gfx::unlock_surface`). A height of 0 is refused, before anything.

use crate::errors::{BadInput, Refused};
pub fn clear_back_surface(
    gfx: &crate::gfx::owner::Gfx,
    system: &crate::system::owner::System,
) -> Result<(), Refused> {
    let drawing = &gfx.drawing;
    if drawing.surfaces.front.get() == 0 {
        return Ok(());
    }
    if system.session.screen.get().height.get() == 0 {
        return Err(BadInput("a screen of no rows: the original goes round 2^32 times").into());
    }
    // Not locked.
    if crate::platform::lock_surface(gfx, system) < 1 {
        return Ok(());
    }
    // Each row's dwords, then the gap to the next.
    let dwords = system.session.screen.get().row_dwords.get();
    let rows = crate::gfx::rows::Rows::new(
        drawing.draw_surface.present_pixels.get(),
        (4 * dwords).wrapping_add(drawing.draw_surface.present_gap.get()),
        system.session.screen.get().height.get(),
    );
    for y in 0..rows.count() {
        rows.fill(y, 0, 0, 4 * dwords as usize, "the back surface")?;
    }
    crate::platform::unlock_surface(gfx);
    Ok(())
}
