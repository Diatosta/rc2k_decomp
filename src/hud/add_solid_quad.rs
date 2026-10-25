//! `hud::add_solid_quad` (0x4B7216): the quad record's texture places all on one texel
//! (0xF8 << 16, 8 << 16: a solid colour), then `hud::add_quad`.
//!
//! In:  nothing (the record's corners and colour set by the caller)
//! Out: as `add_quad` leaves them
//!
//! Callers: the HUD's drawers (code at 0x4B27E1, 0x4B3BF8, 0x4B5AD0, ...).

use crate::errors::Refused;
use crate::hud::store::HUD;

/// The Rust version (see the header).
// Converted: `rc2k_core::hud::quads::SOLID_PLACES`
pub unsafe fn add_solid_quad() -> Result<(), Refused> {
    for (n, v) in rc2k_core::hud::quads::SOLID_PLACES {
        HUD.quad[n].set(v);
    }
    unsafe { crate::hud::add_quad::add_quad() }
}
