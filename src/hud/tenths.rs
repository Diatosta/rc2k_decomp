//! Six times of the panel in tenths (signed `idiv` by 10): each view's
//! `split2`, `negated` and `time` (the first view's, then the second's) into the HUD's `times`.

use crate::hud::owner::Hud;
use crate::race::owner_frame::Frame;

/// The panel's six times in tenths.
pub fn tenths(frame: &Frame, hud: &Hud) {
    let (a, b) = (
        &frame.panel_and_overlay.get().panel.view_a,
        &frame.panel_and_overlay.get().panel.view_b,
    );
    let times = [
        &a.split2, &a.negated, &a.time, &b.split2, &b.negated, &b.time,
    ]
    .map(|cell| cell.get() as i32);
    let (tenths, _) = rc2k_core::hud::tenths::tenths(times);
    for (cell, v) in hud.block.times.iter().zip(tenths) {
        cell.set(v as u32);
    }
}
