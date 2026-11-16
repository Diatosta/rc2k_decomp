//! The GPU tab: the diagnostic cuts of `render::gpu_cut` as tick boxes, applied to the next
//! frame drawn. Each removes one kind of GPU work so that the frame rate shows what it cost;
//! the picture is wrong under a cut, which is the point.

use crate::race::owner::Race;
use crate::render::gpu_cut::{CUTS, Cuts, live, set_live};
use dear_imgui_rs::Ui;
use std::sync::Mutex;
use std::sync::atomic::Ordering::Relaxed;
use std::time::Instant;

/// The GPU flushes counted at the last reading, and when.
static LAST_FLUSHES: Mutex<Option<(u64, Instant)>> = Mutex::new(None);

/// The flushes the GPU drew a second, over the time since the last call (none before a first
/// reading and where no GPU draws).
fn flushes_a_second() -> Option<f32> {
    let now = Instant::now();
    let flushes = crate::render::GPU_FLUSHES.load(Relaxed);
    let mut last = LAST_FLUSHES
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let rate = last.and_then(|(was, at)| {
        let seconds = now.duration_since(at).as_secs_f32();
        (seconds > 0.0).then(|| (flushes - was) as f32 / seconds)
    });
    *last = Some((flushes, now));
    rate
}

/// The tab.
pub fn tab(race: &Race, ui: &Ui) {
    let fps = race.shared.switches.get().frame_rate.get();
    let flushes = crate::render::GPU_FLUSHES.load(Relaxed);
    let backend = if !crate::render::choice::gpu_wanted() {
        "software rasteriser (--renderer or RC2K_RENDERER)"
    } else if flushes > 0 {
        "SDL GPU"
    } else {
        "SDL GPU wanted, nothing drawn by it yet"
    };
    ui.text(format!("Backend: {backend}"));
    let rate =
        flushes_a_second().map_or_else(String::new, |r| format!(", GPU flushes: {r:.1} a second"));
    ui.text(format!(
        "Frame rate (the game's, over 16 frames): {fps:.1}{rate}"
    ));
    ui.text_disabled("Each cut removes one kind of GPU work; the picture is wrong under it.");
    let mut bits = live().bits();
    let mut changed = false;
    if ui.button("none") {
        bits = 0;
        changed = true;
    }
    ui.same_line();
    ui.text_disabled(format!("On: {}", Cuts::from_bits(bits).names()));
    ui.separator();
    for (n, cut) in CUTS.iter().enumerate() {
        let mut on = bits & 1 << n != 0;
        if ui.checkbox(cut.name, &mut on) {
            bits = if on { bits | 1 << n } else { bits & !(1 << n) };
            changed = true;
        }
        ui.same_line();
        ui.text_disabled(cut.about);
    }
    if changed {
        set_live(Cuts::from_bits(bits));
    }
}
