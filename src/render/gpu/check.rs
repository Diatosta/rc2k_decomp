//! `RC2K_BACKGROUND_CHECK`: every frame is drawn a second time, into targets of its own, with the
//! background carried by the canvas as it was before the GPU kept it (the canvas whole, the
//! whole canvas put over the target); at the frame's end the two pictures are compared and
//! the differing pixels counted in the log. A check only: it costs about twice the drawing.

use super::{Gpu, UNWRITTEN};
use crate::platform::ffi::sdl_gpu::{Texture, Transfer};
use crate::program_log::log;
use sdl3_sys::everything::*;

/// Whether the check is asked for.
pub fn wanted() -> bool {
    static WANTED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *WANTED.get_or_init(|| std::env::var_os("RC2K_BACKGROUND_CHECK").is_some())
        || crate::gfx::draw2d::quads::check_wanted()
}

/// Whether `RC2K_CHECK_SAME` is set: the second pass draws as the first does, so the check
/// counts what the GPU alone makes differ.
pub fn same() -> bool {
    static SAME: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *SAME.get_or_init(|| std::env::var_os("RC2K_CHECK_SAME").is_some())
}

/// Logs rows that hold a written pixel but were not noted as written (the GPU path would never
/// draw them).
pub fn note_missed_rows(scanned: &[bool], noted: &[bool]) {
    use std::sync::atomic::{AtomicU32, Ordering::Relaxed};
    static LOGGED: AtomicU32 = AtomicU32::new(0);
    let missed: Vec<usize> = (0..scanned.len().min(noted.len()))
        .filter(|&y| scanned[y] && !noted[y])
        .collect();
    if !missed.is_empty() && LOGGED.fetch_add(1, Relaxed) < 50 {
        log(&format!(
            "render: canvas rows written but not noted: {missed:?}"
        ));
    }
}

/// The second set of targets, at the screen's size.
pub struct Check {
    pub colour: Texture,
    pub depth: Texture,
    pub canvas: Texture,
    pub canvas_transfer: Transfer,
    pub depth_is_new: bool,
    width: u32,
    height: u32,
}

/// The canvas as it was when the background was copied onto it: the background where the
/// canvas is unwritten, the canvas's rows (`runs`) over it. With no background, the canvas.
pub fn canvas_as_before(
    canvas: &[u8],
    runs: &[(usize, usize)],
    stride: usize,
    background: Option<&[u8]>,
) -> Vec<u8> {
    let mut before = background.map_or_else(
        || {
            let mut blank = vec![0u8; canvas.len()];
            blank.as_chunks_mut::<4>().0.fill(UNWRITTEN.to_le_bytes());
            blank
        },
        <[u8]>::to_vec,
    );
    for &(first, last) in runs {
        let rows = first * stride..(last + 1) * stride;
        for (to, from) in before[rows.clone()]
            .as_chunks_mut::<4>()
            .0
            .iter_mut()
            .zip(canvas[rows].as_chunks::<4>().0)
        {
            if *from != UNWRITTEN.to_le_bytes() {
                *to = *from;
            }
        }
    }
    before
}

impl Gpu {
    /// The check's targets, made for the screen's size when it is asked for.
    pub(super) fn ensure_check(&mut self) -> Result<(), String> {
        if !wanted()
            || self
                .check
                .as_ref()
                .is_some_and(|c| (c.width, c.height) == (self.width, self.height))
        {
            return Ok(());
        }
        if let Some(old) = self.check.take() {
            for texture in [old.colour, old.depth, old.canvas] {
                self.device.release_texture(texture);
            }
            self.device.release_transfer(old.canvas_transfer);
        }
        let (width, height) = (self.width, self.height);
        self.check = Some(Check {
            colour: self.make_texture(
                width,
                height,
                SDL_GPU_TEXTUREUSAGE_COLOR_TARGET | SDL_GPU_TEXTUREUSAGE_SAMPLER,
            )?,
            depth: self.make_texture_of(
                super::DEPTH_FORMAT,
                width,
                height,
                SDL_GPU_TEXTUREUSAGE_DEPTH_STENCIL_TARGET,
            )?,
            canvas: self.make_texture(width, height, SDL_GPU_TEXTUREUSAGE_SAMPLER)?,
            canvas_transfer: self
                .make_transfer(SDL_GPU_TRANSFERBUFFERUSAGE_UPLOAD, width * height * 4)?,
            depth_is_new: true,
            width,
            height,
        });
        Ok(())
    }

    /// At a frame's end: the picture drawn the new way against the one drawn the old way.
    pub(super) fn compare_check(&mut self) -> Result<(), String> {
        use std::sync::atomic::{AtomicU64, Ordering::Relaxed};
        static FRAMES: AtomicU64 = AtomicU64::new(0);
        static BAD_FRAMES: AtomicU64 = AtomicU64::new(0);
        let Some(old_colour) = self.check.as_ref().map(|c| c.colour) else {
            return Ok(());
        };
        let new = self.read_colour()?;
        let old = self.read_texture(old_colour)?;
        let width = self.width as usize;
        let mut differing = 0usize;
        let mut largest = 0u8;
        let (mut top, mut left, mut bottom, mut right) = (usize::MAX, usize::MAX, 0, 0);
        for (n, (a, b)) in new
            .as_chunks::<4>()
            .0
            .iter()
            .zip(old.as_chunks::<4>().0)
            .enumerate()
            .filter(|(_, (a, b))| a != b)
        {
            differing += 1;
            largest = largest.max((0..4).map(|k| a[k].abs_diff(b[k])).max().unwrap_or(0));
            let (y, x) = (n / width, n % width);
            (top, bottom) = (top.min(y), bottom.max(y));
            (left, right) = (left.min(x), right.max(x));
        }
        let frames = FRAMES.fetch_add(1, Relaxed) + 1;
        if differing > 0 {
            let bad = BAD_FRAMES.fetch_add(1, Relaxed) + 1;
            if bad <= 3
                && let Some(dir) = std::env::var_os("RC2K_BACKGROUND_CHECK").filter(|d| d.len() > 1)
            {
                let dir = std::path::Path::new(&dir);
                let _ = std::fs::write(
                    dir.join(format!("bad{}_{bad}_new.raw", std::process::id())),
                    &new,
                );
                let _ = std::fs::write(
                    dir.join(format!("bad{}_{bad}_old.raw", std::process::id())),
                    &old,
                );
            }
            if bad <= 50 {
                log(&format!(
                    "render: background check: {differing} pixels differ by up to {largest}, in x {left}..={right}, y {top}..={bottom} (frame {frames} checked, {bad} bad)"
                ));
            }
        } else if frames.is_power_of_two() {
            log(&format!(
                "render: background check: {frames} frames checked, {} bad",
                BAD_FRAMES.load(Relaxed)
            ));
        }
        Ok(())
    }
}
