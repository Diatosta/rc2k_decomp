//! The box the HUD keeps to (`rc2k_core::hud::anchor::BoxRatio`), chosen on the graphics page and
//! kept in `rc2k.toml` (`hud_box`).

use rc2k_core::hud::anchor::BoxRatio;
use std::sync::atomic::{AtomicU8, Ordering::Relaxed};

static BOX: AtomicU8 = AtomicU8::new(0);

pub fn get() -> BoxRatio {
    BoxRatio::ALL[usize::from(BOX.load(Relaxed)).min(BoxRatio::ALL.len() - 1)]
}

pub fn set(ratio: BoxRatio) {
    let at = BoxRatio::ALL.iter().position(|&r| r == ratio).unwrap_or(0);
    BOX.store(at as u8, Relaxed);
}

/// One choice on (`up`) or back.
pub fn step(up: bool) {
    set(get().step(up));
}
