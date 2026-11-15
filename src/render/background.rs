//! The menu's background kept on the GPU. `gfx::copy_picture_to_surface` notes which picture
//! buffer (and which state of it) is the background instead of copying it onto the canvas; the
//! GPU backend draws it under the canvas at the frame's first flush and uploads it only when the
//! picture buffer has changed, so the canvas holds the foreground alone. A background covers the
//! whole screen: whatever was drawn before it in the frame (noted quads, canvas pixels) is
//! dropped, and the background is again the first thing under what follows.

use super::Target;
#[cfg(not(all(target_os = "linux", target_arch = "aarch64")))]
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering::Relaxed};

/// A background waiting for the next flush.
#[cfg(not(all(target_os = "linux", target_arch = "aarch64")))]
pub enum Pending {
    /// Picture buffer `handle` as it is at change counter `generation` (`gfx::picture::
    /// generation`).
    Buffer { handle: u32, generation: u64 },
    /// Pixels kept as they were when the background was noted: a picture buffer that changed
    /// before the flush.
    Copy(Vec<u8>),
    /// The loading picture rescaled to the screen (`gfx::rescale_to_surface`): the pixels are
    /// shared with the rescale's result, and `generation` names that result, so the GPU uploads
    /// them only when a new rescale has made other pixels.
    Rescaled {
        generation: u64,
        pixels: std::sync::Arc<Vec<u8>>,
    },
}

#[cfg(not(all(target_os = "linux", target_arch = "aarch64")))]
static PENDING: Mutex<Option<Pending>> = Mutex::new(None);

/// Whether a background was noted since the last frame's end check (`recorded_this_frame`).
static RECORDED: AtomicBool = AtomicBool::new(false);

/// Notes picture buffer `handle` as the frame's background, for the GPU to draw under the canvas.
/// False when the caller must copy it onto the canvas as before: no GPU backend, a canvas that is
/// not 32-bit rows of exactly the screen's width, or a buffer too small for it.
#[cfg(not(all(target_os = "linux", target_arch = "aarch64")))]
pub fn defer(target: &Target, handle: u32) -> bool {
    use crate::gfx::picture;
    defer_with(target, || {
        let (Some(generation), Ok(len)) = (
            picture::generation(handle),
            picture::with_buffer_unchanged(handle, <[u8]>::len),
        ) else {
            return None;
        };
        (len >= target.width * target.height * 4).then_some(Pending::Buffer { handle, generation })
    })
}

/// Notes `pixels` (the screen's rows, `height` of them, 32-bit and without gaps) as the frame's
/// background, as `defer` does for a picture buffer: the loading picture, drawn once rescaled.
/// `generation` is the rescale's number (`gfx::rescale_to_surface::Rescaled::generation`): the
/// same number means the same pixels.
#[cfg(not(all(target_os = "linux", target_arch = "aarch64")))]
pub fn defer_pixels(target: &Target, generation: u64, pixels: &std::sync::Arc<Vec<u8>>) -> bool {
    defer_with(target, || {
        (pixels.len() >= target.width * target.height * 4).then(|| Pending::Rescaled {
            generation,
            pixels: std::sync::Arc::clone(pixels),
        })
    })
}

/// Notes the background `source` gives, when the GPU can keep it for `target`.
#[cfg(not(all(target_os = "linux", target_arch = "aarch64")))]
fn defer_with(target: &Target, source: impl FnOnce() -> Option<Pending>) -> bool {
    if !super::gpu_running() || target.bits != 32 || target.pitch != target.width * 4 {
        return false;
    }
    let Some(held) = crate::gfx::locked_pixels::bytes_of(target.pixels) else {
        return false;
    };
    let Some(written) = held.any_row_written(target.pitch) else {
        return false;
    };
    let Some(background) = source() else {
        return false;
    };
    // What the frame drew before is under the picture: quads noted so far are dropped, and the
    // canvas, which holds either the last frame's picture (which the GPU does not need) or
    // foreground drawn this frame, is blanked: the frame is the GPU's from here, with a canvas
    // that has written nothing yet.
    let holds = super::GPU_HOLDS_FRAME.load(Relaxed);
    if holds {
        crate::gfx::draw2d::quads::discard_pending();
    }
    if written || !holds {
        target.with_canvas(super::gpu::mark_unwritten);
        held.take_written_rows(target.pitch);
    }
    super::GPU_HOLDS_FRAME.store(true, Relaxed);
    *PENDING.lock().unwrap() = Some(background);
    RECORDED.store(true, Relaxed);
    true
}

/// The handheld's GPU path draws the background from the canvas, as before.
#[cfg(all(target_os = "linux", target_arch = "aarch64"))]
pub fn defer(_target: &Target, _handle: u32) -> bool {
    false
}

/// The handheld's GPU path draws the loading picture from the canvas, as before.
#[cfg(all(target_os = "linux", target_arch = "aarch64"))]
pub fn defer_pixels(_target: &Target, _generation: u64, _pixels: &std::sync::Arc<Vec<u8>>) -> bool {
    false
}

/// The noted background, taken: the flush draws it now.
#[cfg(not(all(target_os = "linux", target_arch = "aarch64")))]
pub fn take() -> Option<Pending> {
    PENDING.lock().unwrap().take()
}

/// The noted background dropped (the canvas is written over it, or the GPU is given up).
#[cfg(not(all(target_os = "linux", target_arch = "aarch64")))]
pub fn clear() {
    *PENDING.lock().unwrap() = None;
}

/// Whether a background was noted since the last call: a menu frame with one stays on the GPU
/// even with no 3D in it.
pub fn recorded_this_frame() -> bool {
    RECORDED.swap(false, Relaxed)
}

/// Picture buffer `handle` is about to change (or go): a noted background that names it takes
/// the bytes first (`bytes` as they are now).
#[cfg(not(all(target_os = "linux", target_arch = "aarch64")))]
pub fn before_change(handle: u32, bytes: &[u8]) {
    let mut pending = PENDING.lock().unwrap();
    if matches!(*pending, Some(Pending::Buffer { handle: noted, .. }) if noted == handle) {
        *pending = Some(Pending::Copy(bytes.to_vec()));
    }
}

/// The handheld keeps no background on the GPU: nothing to take.
#[cfg(all(target_os = "linux", target_arch = "aarch64"))]
pub fn before_change(_handle: u32, _bytes: &[u8]) {}

#[cfg(all(test, not(all(target_os = "linux", target_arch = "aarch64"))))]
mod tests {
    use super::*;

    #[test]
    fn a_background_takes_a_copy_before_its_buffer_changes() {
        clear();
        *PENDING.lock().unwrap() = Some(Pending::Buffer {
            handle: 2,
            generation: 7,
        });
        before_change(3, &[9]);
        assert!(matches!(
            take(),
            Some(Pending::Buffer {
                handle: 2,
                generation: 7
            })
        ));
        *PENDING.lock().unwrap() = Some(Pending::Buffer {
            handle: 2,
            generation: 7,
        });
        before_change(2, &[1, 2]);
        assert!(matches!(take(), Some(Pending::Copy(bytes)) if bytes == [1, 2]));
        assert!(take().is_none());
    }

    #[test]
    fn a_rescaled_background_shares_its_pixels_and_keeps_its_generation() {
        clear();
        let pixels = std::sync::Arc::new(vec![3u8; 4]);
        *PENDING.lock().unwrap() = Some(Pending::Rescaled {
            generation: 5,
            pixels: std::sync::Arc::clone(&pixels),
        });
        // A buffer's change does not touch it.
        before_change(2, &[1, 2]);
        assert!(matches!(
            take(),
            Some(Pending::Rescaled { generation: 5, pixels: shared })
                if std::sync::Arc::ptr_eq(&shared, &pixels)
        ));
    }
}
