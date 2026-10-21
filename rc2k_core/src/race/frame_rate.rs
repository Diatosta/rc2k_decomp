//! The frame rate measured every 16th frame from the time-stamp counter (converted
//! 2026-10-21 from `race::measure_frame_rate` 0x42009E). Floats as the float pass keeps
//! them: the x87's stack in `f64`, the stored rate rounded to `f32` (exact at the game's
//! double precision). Its text converted 2026-10-21 from `race::draw_frame_rate` 0x4562B2.

/// The frames between two measures.
pub const MEASURE_EVERY: u32 = 0x10;

/// The frame counter (`dword_613838`) one on: the count kept and whether to measure now
/// (at 0x10, the count back to 0).
pub fn count_frame(count: u32) -> (u32, bool) {
    let n = count.wrapping_add(1);
    if n == MEASURE_EVERY {
        (0, true)
    } else {
        (n, false)
    }
}

/// The cycles since the last measure, when above 0 (signed): the rate is worked out
/// only then.
pub fn cycles_since(now: u32, last: u32) -> Option<u32> {
    let c = now.wrapping_sub(last);
    ((c as i32) > 0).then_some(c)
}

/// The frame rate (`flt_6214F0`): the CPU's clock (`dword_5183E8`) x `k` x `k` x `k2`
/// (`flt_5C5D18`, `flt_5C5D08`) / the cycles, as `fild`, `fmul`s, `fild`, `fdivp`, `fstp`.
pub fn frame_rate(clock: i32, k: f32, k2: f32, cycles: i32) -> f32 {
    let product = clock as f64 * k as f64 * k as f64 * k2 as f64;
    (product / cycles as f64) as f32
}

/// The rate as shown (`race::draw_frame_rate` 0x4562B2): the float rounded (`fld`,
/// `fistp`; nothing is shown at 0).
pub fn shown_rate(rate: f32) -> i32 {
    crate::math::geometry::fistp(rate)
}

/// The frame rate's text as written into the text buffer: "fps:    " with the rate's
/// size (no padding) from the sixth byte and a NUL after it.
pub fn fps_text(rate: i32) -> Vec<u8> {
    let mut text = b"fps:    ".to_vec();
    let digits = crate::strings::format::decimal(
        rate.wrapping_abs() as u32,
        crate::strings::format::Pad::None,
    );
    text.truncate(5);
    text.extend_from_slice(&digits);
    text.push(0);
    if text.len() < 8 {
        text.extend_from_slice(&b"fps:    "[text.len()..]);
    }
    text
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counting() {
        assert_eq!(count_frame(0xE), (0xF, false));
        assert_eq!(count_frame(0xF), (0, true));
        assert_eq!(count_frame(u32::MAX), (0, false));
        assert_eq!(count_frame(0x10), (0x11, false));
        assert_eq!(cycles_since(10, 4), Some(6));
        assert_eq!(cycles_since(4, 10), None);
        assert_eq!(cycles_since(4, 4), None);
    }

    #[test]
    fn shown() {
        assert_eq!(shown_rate(29.5), 30);
        assert_eq!(shown_rate(30.5), 30);
        assert_eq!(shown_rate(-60.0), -60);
        assert_eq!(fps_text(7), b"fps: 7\0 ");
        assert_eq!(fps_text(-60), b"fps: 60\0");
        assert_eq!(fps_text(1000), b"fps: 1000\0");
    }

    #[test]
    fn rate() {
        assert_eq!(frame_rate(450, 1000.0, 1.0, 1_000_000), 450.0);
        assert_eq!(frame_rate(3, 1.0, 1.0, 2), 1.5);
        // Rounded once from the double quotient.
        assert_eq!(frame_rate(1, 1.0, 1.0, 3), (1.0f64 / 3.0) as f32);
    }
}
