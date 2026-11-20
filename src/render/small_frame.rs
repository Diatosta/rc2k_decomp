//! A race's 3D drawn by the software renderer at a share of the screen's size
//! (`rc2k_core::gfx::render_scale`): the screen's picture so far made smaller, the fans drawn
//! over it, the result enlarged over the screen (each pixel the nearest). Above 100 percent the
//! frame is larger than the screen and is scaled down with an area filter (every screen pixel
//! the mean of the pixels it covers).

use rc2k_core::gfx::render_scale::{covered, scaled, source};
use std::sync::{Mutex, MutexGuard};

struct SmallBuffers {
    pixels: Vec<u32>,
    depth: Vec<f32>,
    width: usize,
}

static SMALL: Mutex<SmallBuffers> = Mutex::new(SmallBuffers {
    pixels: Vec::new(),
    depth: Vec::new(),
    width: 0,
});

/// A small frame while it is drawn: its buffers held until it is enlarged.
pub struct Small {
    buffers: MutexGuard<'static, SmallBuffers>,
    /// The screen's size.
    full: (usize, usize),
}

impl Small {
    /// The small frame at `percent` of the screen's size, holding the screen's picture made
    /// smaller (what was drawn before this flush, which the 3D may blend over).
    pub(super) fn from_screen(
        screen: &[u8],
        pitch: usize,
        width: usize,
        height: usize,
        percent: u32,
    ) -> Self {
        let (w, h) = (scaled(width, percent), scaled(height, percent));
        let mut b = SMALL
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if b.pixels.len() != w * h {
            b.pixels = vec![0; w * h];
            b.depth = vec![1.0; w * h];
        }
        b.width = w;
        // The screen pixel at the middle of each small pixel.
        let from_x: Vec<usize> = (0..w).map(|x| (2 * x + 1) * width / (2 * w)).collect();
        for (y, to) in b.pixels.chunks_exact_mut(w).enumerate() {
            let row = ((2 * y + 1) * height / (2 * h)).min(height - 1);
            let from = screen[row * pitch..].as_chunks::<4>().0;
            for (pixel, &x) in to.iter_mut().zip(&from_x) {
                *pixel = u32::from_ne_bytes(from[x]);
            }
        }
        Self {
            buffers: b,
            full: (width, height),
        }
    }

    /// The small frame's size.
    pub fn size(&self) -> (usize, usize) {
        let b = &*self.buffers;
        (b.width, b.pixels.len() / b.width)
    }

    /// For the rasteriser's `Frame`, which holds them no longer than this `Small` lives:
    /// (pixels, width, height, depths).
    pub(super) fn frame_parts(&mut self) -> (&mut [u32], usize, usize, &mut [f32]) {
        let b = &mut *self.buffers;
        let height = b.pixels.len() / b.width;
        (&mut b.pixels, b.width, height, &mut b.depth)
    }

    /// Back on the screen: a smaller frame's pixels each made as many pixels wide and high as
    /// it covers, a larger frame's pixels averaged by the area each screen pixel covers.
    pub(super) fn enlarge_to(self, screen: &mut [u8], pitch: usize) {
        let b = &*self.buffers;
        let (width, height) = self.full;
        if b.width > width || b.pixels.len() / b.width > height {
            return self.shrink_to(screen, pitch);
        }
        let small_height = b.pixels.len() / b.width;
        let from_x: Vec<usize> = (0..width).map(|x| source(x, width, b.width)).collect();
        for (y, row) in screen.chunks_exact_mut(pitch).take(height).enumerate() {
            let from = &b.pixels[source(y, height, small_height) * b.width..][..b.width];
            for (to, &x) in row[..width * 4]
                .as_chunks_mut::<4>()
                .0
                .iter_mut()
                .zip(&from_x)
            {
                *to = from[x].to_ne_bytes();
            }
        }
    }

    /// The larger frame averaged onto the screen: across, then down, each screen pixel the mean
    /// of the frame's pixels it covers, weighted by how much of each (channels apart).
    fn shrink_to(&self, screen: &mut [u8], pitch: usize) {
        let b = &*self.buffers;
        let (width, height) = self.full;
        let small_height = b.pixels.len() / b.width;
        let channels = |p: u32| p.to_ne_bytes().map(u64::from);
        // Across: every frame row made `width` pixels wide, as sums of weight x channel.
        let across: Vec<Vec<(usize, usize)>> = (0..width)
            .map(|x| covered(x, width, b.width).collect())
            .collect();
        let mut rows: Vec<[u32; 4]> = Vec::with_capacity(width * small_height);
        for row in b.pixels.chunks_exact(b.width) {
            for parts in &across {
                let mut sum = [0u64; 4];
                for &(i, weight) in parts {
                    for (s, c) in sum.iter_mut().zip(channels(row[i])) {
                        *s += c * weight as u64;
                    }
                }
                rows.push(sum.map(|s| ((s + b.width as u64 / 2) / b.width as u64) as u32));
            }
        }
        // Down: the means of those rows, the same way.
        for (y, out) in screen.chunks_exact_mut(pitch).take(height).enumerate() {
            let parts: Vec<(usize, usize)> = covered(y, height, small_height).collect();
            for (x, to) in out[..width * 4]
                .as_chunks_mut::<4>()
                .0
                .iter_mut()
                .enumerate()
            {
                let mut sum = [0u64; 4];
                for &(j, weight) in &parts {
                    for (s, c) in sum.iter_mut().zip(rows[j * width + x]) {
                        *s += u64::from(c) * weight as u64;
                    }
                }
                let half = small_height as u64 / 2;
                *to = sum.map(|s| ((s + half) / small_height as u64) as u8);
            }
        }
    }

    /// The noted things' screen places made fit the small frame: each fan's vertices and
    /// bounds, each depth clear's rectangle.
    pub(super) fn fit_noted(&self, noted: &mut super::CommandList) {
        let (w, h) = self.size();
        let (bx, by) = (w as f32 / self.full.0 as f32, h as f32 / self.full.1 as f32);
        // Every vertex is a fan's (the list holds no others).
        for v in noted.vertices.iter_mut() {
            v[0] = (f32::from_bits(v[0]) * bx).to_bits();
            v[1] = (f32::from_bits(v[1]) * by).to_bits();
        }
        let across = |n: usize| n * w / self.full.0;
        let across_up = |n: usize| (n * w).div_ceil(self.full.0);
        let down = |n: usize| n * h / self.full.1;
        let down_up = |n: usize| (n * h).div_ceil(self.full.1);
        for what in noted.commands.iter_mut() {
            match what {
                super::Command::Fan { bounds, .. } => {
                    *bounds = (bounds.0 * bx, bounds.1 * by, bounds.2 * bx, bounds.3 * by);
                }
                super::Command::Clear([left, top, right, bottom]) => {
                    *left = across(*left);
                    *top = down(*top);
                    *right = across_up(*right);
                    *bottom = down_up(*bottom);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_larger_frame_comes_back_as_the_screen_it_was_made_from() {
        // The screen's picture made larger and brought back is the same picture.
        for percent in [125, 150, 200] {
            let screen: Vec<u8> = (0..6 * 5 * 4).map(|n| (n * 37 % 251) as u8).collect();
            let small = Small::from_screen(&screen, 6 * 4, 6, 5, percent);
            let mut back = vec![0u8; screen.len()];
            small.enlarge_to(&mut back, 6 * 4);
            if percent == 200 {
                assert_eq!(back, screen);
            }
        }
    }

    #[test]
    fn a_checker_drawn_at_double_size_averages_to_grey() {
        let screen = vec![0u8; 4 * 4 * 4];
        let mut small = Small::from_screen(&screen, 16, 4, 4, 200);
        let (pixels, width, height, _) = small.frame_parts();
        for (n, p) in pixels.iter_mut().enumerate() {
            *p = if (n % width + n / width) % 2 == 0 {
                0xFF_FF_FF_FF
            } else {
                0
            };
        }
        assert_eq!((width, height), (8, 8));
        let mut back = vec![0u8; screen.len()];
        small.enlarge_to(&mut back, 16);
        assert!(back.iter().all(|&b| b == 128 || b == 127), "{back:?}");
    }
}
