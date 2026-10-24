//! `marks::flare_frame` (0x4C4EA2): `scene::set_texture` with `dword_5D1E10`,
//! `scene::depth_test_off`, and when the settings' +0x2754 (the flare's strength) is set
//! `marks::draw_lens_flare`.
//!
//! In:  nothing (`g_marksSettings`)
//! Out: nothing (no caller reads a register it leaves)
//!
//! Caller: `marks::run_frame` (0x4D1645). Its calls through `part` (1 the texture, 2 the
//! depth test, 3 the flare); in the game their Rust versions. Refuses null settings,
//! before anything. Shadow mode runs the original alone (it tells the device). Self-test:
//! a copy whose calls go to stand-ins, against the Rust version, the strength 0 and not.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, Refused};
use crate::framework::{game_read, rva};
use crate::marks::setup::SETTINGS_RVA;

const TEXTURE_RVA: usize = rva(0x5D1E10); // dword_5D1E10

/// The Rust version (see the header): `scene::set_texture`, `scene::depth_test_off`, and
/// with the settings' +0x2754 `draw_lens_flare`.
pub unsafe fn flare_frame() -> Result<(), Refused> {
    unsafe {
        if game_read::<u32>(SETTINGS_RVA) == 0 {
            return Err(BadInput("no marks settings").into());
        }
        crate::scene::set_texture::set_texture(game_read(TEXTURE_RVA));
        crate::scene::depth_test_off::depth_test_off();
        let settings: u32 = game_read(SETTINGS_RVA);
        if ((settings + 0x2754) as *const u32).read_unaligned() != 0 {
            crate::marks::draw_lens_flare::draw_lens_flare()?;
        }
    }
    Ok(())
}
