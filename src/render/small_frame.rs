//! A race's 3D drawn by the software renderer at a share of the screen's size
//! (`rc2k_core::gfx::render_scale`): the screen's picture so far made smaller, the fans drawn
//! over it, the result enlarged over the screen (each pixel the nearest).

use rc2k_core::gfx::render_scale::{scaled, source};
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
    pub fn from_screen(
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
    pub fn frame_parts(&mut self) -> (&mut [u32], usize, usize, &mut [f32]) {
        let b = &mut *self.buffers;
        let height = b.pixels.len() / b.width;
        (&mut b.pixels, b.width, height, &mut b.depth)
    }

    /// Back on the screen, each small pixel made as many pixels wide and high as it covers.
    pub fn enlarge_to(self, screen: &mut [u8], pitch: usize) {
        let b = &*self.buffers;
        let (width, height) = self.full;
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

    /// The noted things' screen places made fit the small frame: each fan's vertices and
    /// bounds, each depth clear's rectangle.
    pub fn fit_noted(&self, noted: &mut super::CommandList) {
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
