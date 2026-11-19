//! Camera frames made up for the tests: a QR code as a screen shows it, seen by a camera.
//!
//! The code fills a share of the frame's height, turned a little, mirrored when asked, blurred
//! by the lens and noisy; the room round the screen is dark.

use super::qr::{self, Modules};

/// One made-up view.
#[derive(Clone, Copy)]
pub struct View {
    pub width: usize,
    pub height: usize,
    /// The share of the frame's height the code (with its quiet zone) takes, in percent.
    pub percent: usize,
    pub degrees: f32,
    pub mirrored: bool,
    /// How many passes of a 3 x 3 blur.
    pub blur: usize,
    /// The noise's amplitude, in grey levels.
    pub noise: i32,
    /// Light modules on a dark ground.
    pub inverted: bool,
    /// Another number makes another noise and a slightly different turn, as the next frame of
    /// a camera held in a hand would be.
    pub frame: u32,
}

impl View {
    pub fn plain(width: usize, height: usize, percent: usize) -> Self {
        Self {
            width,
            height,
            percent,
            degrees: 0.0,
            mirrored: false,
            blur: 0,
            noise: 0,
            inverted: false,
            frame: 0,
        }
    }
}

/// The grey frame of `modules` as seen in `view`.
pub fn frame(modules: &Modules, view: View) -> Vec<u8> {
    let (w, h) = (view.width, view.height);
    let units = (modules.side + 2 * qr::QUIET) as f32;
    let size = (h * view.percent / 100) as f32;
    let (sin, cos) = {
        let angle = (view.degrees + 0.6 * view.frame as f32).to_radians();
        (
            crate::math::portable::sinf(angle),
            crate::math::portable::cosf(angle),
        )
    };
    let (cx, cy) = (w as f32 / 2.0, h as f32 / 2.0);
    let sample = |fx: f32, fy: f32| -> f32 {
        // Frame to code: back from the turn, then from pixels to modules.
        let (dx, dy) = (fx - cx, fy - cy);
        let (rx, ry) = (dx * cos + dy * sin, -dx * sin + dy * cos);
        let (u, v) = (
            rx / size * units + units / 2.0,
            ry / size * units + units / 2.0,
        );
        if u < 0.0 || v < 0.0 || u >= units || v >= units {
            return 50.0;
        }
        let (mx, my) = (u as usize, v as usize);
        let in_code = mx >= qr::QUIET
            && my >= qr::QUIET
            && mx < qr::QUIET + modules.side
            && my < qr::QUIET + modules.side;
        if in_code && modules.at(mx - qr::QUIET, my - qr::QUIET) {
            30.0
        } else {
            220.0
        }
    };
    let mut grey: Vec<f32> = Vec::with_capacity(w * h);
    for y in 0..h {
        for x in 0..w {
            let mut sum = 0.0;
            for sy in 0..3 {
                for sx in 0..3 {
                    sum += sample(
                        x as f32 + (sx as f32 + 0.5) / 3.0,
                        y as f32 + (sy as f32 + 0.5) / 3.0,
                    );
                }
            }
            grey.push(sum / 9.0);
        }
    }
    for _ in 0..view.blur {
        let before = grey.clone();
        for y in 1..h - 1 {
            for x in 1..w - 1 {
                let mut sum = 0.0;
                for dy in 0..3 {
                    for dx in 0..3 {
                        sum += before[(y + dy - 1) * w + x + dx - 1];
                    }
                }
                grey[y * w + x] = sum / 9.0;
            }
        }
    }
    let mut seed = 0x2545_F491_u32 ^ view.frame.wrapping_mul(0x9E37_79B9);
    let mut out: Vec<u8> = grey
        .iter()
        .map(|g| {
            seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            let n = if view.noise > 0 {
                (seed >> 16) as i32 % (2 * view.noise + 1) - view.noise
            } else {
                0
            };
            (*g as i32 + n).clamp(0, 255) as u8
        })
        .collect();
    if view.inverted {
        out.iter_mut().for_each(|g| *g = 255 - *g);
    }
    if view.mirrored {
        out = out
            .chunks_exact(w)
            .flat_map(|row| row.iter().rev().copied())
            .collect();
    }
    out
}
