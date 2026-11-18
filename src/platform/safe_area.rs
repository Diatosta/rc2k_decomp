//! The part of the window clear of a phone's notch, rounded corners and home indicator, kept for
//! what places the frame (`frame_fit`) and the HUD (`hud_inset`). Nothing is set on a screen
//! without such edges: the whole window is usable.

use rc2k_core::gfx::screen_fit::{self, Fit};
use std::sync::atomic::{AtomicU32, Ordering::Relaxed};

/// The usable part (x, y, width, height) and the window (width, height), in window pixels; a
/// window width of 0 is nothing noted.
static AREA: [AtomicU32; 6] = [const { AtomicU32::new(0) }; 6];

/// Notes the usable part of a window.
#[cfg(target_os = "ios")]
pub fn set(safe: [u32; 4], window: (u32, u32)) {
    let all = [safe[0], safe[1], safe[2], safe[3], window.0, window.1];
    for (cell, value) in AREA.iter().zip(all) {
        cell.store(value, Relaxed);
    }
}

/// The usable part and the window as last noted, if any.
fn noted() -> Option<([u32; 4], (u32, u32))> {
    let [x, y, w, h, ww, wh]: [u32; 6] = std::array::from_fn(|n| AREA[n].load(Relaxed));
    (ww != 0 && wh != 0).then_some(([x, y, w, h], (ww, wh)))
}

/// Where a frame is shown in a window (`rc2k_core::gfx::screen_fit::place`): the whole window
/// usable where no edges are noted for a window of this size.
pub fn frame_fit(frame: (u32, u32), window: (u32, u32), menu: bool) -> Fit {
    let safe = noted()
        .filter(|&(_, noted_window)| noted_window == window)
        .map_or([0, 0, window.0, window.1], |(safe, _)| safe);
    screen_fit::place(frame, window, safe, menu)
}

/// The part of a race frame of `frame` pixels the HUD keeps to (x, y, width, height): all of it
/// where no edges are noted.
pub fn hud_inset(frame: (u32, u32)) -> [u32; 4] {
    match noted() {
        Some((safe, window)) => screen_fit::safe_in_frame(safe, window, frame),
        None => [0, 0, frame.0, frame.1],
    }
}

/// The frame pixel under a window pixel (a touch or the pointer), for a frame of `frame` pixels
/// shown in a window of `window` pixels as `frame_fit` places it; None outside the frame. The
/// one mapping every pointing device uses.
#[allow(dead_code)] // taken up by the touch input; the unit test below keeps it honest
pub fn window_to_frame(
    point: (f64, f64),
    frame: (u32, u32),
    window: (u32, u32),
    menu: bool,
) -> Option<(f64, f64)> {
    screen_fit::window_to_frame(&frame_fit(frame, window, menu), frame, point)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_touch_in_the_middle_of_the_window_is_the_middle_of_the_frame() {
        let middle = window_to_frame((960.0, 540.0), (800, 600), (1920, 1080), true).unwrap();
        assert!((middle.0 - 400.0).abs() < 1.0 && (middle.1 - 300.0).abs() < 1.0);
        assert_eq!(
            window_to_frame((5.0, 5.0), (800, 600), (1920, 1080), true),
            None
        );
    }
}
