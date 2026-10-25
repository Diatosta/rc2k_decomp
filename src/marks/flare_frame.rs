//! `marks::flare_frame` (0x4C4EA2): `scene::set_texture` with `dword_5D1E10`,
//! `scene::depth_test_off`, and when the settings' +0x2754 (the flare's strength) is set
//! `marks::draw_lens_flare`.
//!
//! In:  nothing (`g_marksSettings`)
//! Out: nothing (no caller reads a register it leaves)
//!
//! Caller: `marks::run_frame` (0x4D1645). Refuses null settings, before anything.

use crate::errors::{BadInput, Refused};
use crate::framework::rva;
use crate::marks::store::game_read;

const TEXTURE_RVA: usize = rva(0x5D1E10); // dword_5D1E10

/// The Rust version (see the header): `scene::set_texture`, `scene::depth_test_off`, and
/// with the settings' +0x2754 `draw_lens_flare`.
pub unsafe fn flare_frame() -> Result<(), Refused> {
    unsafe {
        if crate::marks::setup::SETTINGS.pointer.get() == 0 {
            return Err(BadInput("no marks settings").into());
        }
        crate::scene::set_texture::set_texture(game_read(TEXTURE_RVA));
        crate::scene::depth_test_off::depth_test_off();
        let settings: u32 = crate::marks::setup::SETTINGS.pointer.get();
        if ((settings + 0x2754) as *const u32).read_unaligned() != 0 {
            crate::marks::draw_lens_flare::draw_lens_flare()?;
        }
    }
    Ok(())
}
