//! The menu's background kept on the GPU. `gfx::copy_picture_to_surface` notes which picture
//! buffer (and which state of it) is the background instead of copying it onto the canvas; the
//! GPU backend draws it under the canvas at the frame's first flush and uploads it only when the
//! picture buffer has changed, so the canvas holds the foreground alone. A background covers the
//! whole screen: whatever was drawn before it in the frame (noted quads, canvas pixels) is
//! dropped, and the background is again the first thing under what follows.

use super::Target;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering::Relaxed};

/// A background waiting for the next flush.
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

static PENDING: Mutex<Option<Pending>> = Mutex::new(None);

/// What a GPU's retained background texture holds, so that the next frame's background that is
/// the same is not uploaded again.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Held {
    /// A picture buffer at a change counter.
    Buffer { handle: u32, generation: u64 },
    /// The loading picture's rescale, by its generation.
    Rescaled { generation: u64 },
}

/// Brings the noted background `pending` up to a retained texture of `bytes` bytes (the
/// screen's rows) that holds `held`: when it does not already hold the same picture, `upload`
/// is given the picture's bytes and `held` becomes it. Whether `upload` ran.
pub(super) fn bring_up(
    pending: Pending,
    held: &mut Option<Held>,
    bytes: usize,
    buffers: &crate::gfx::picture::PictureBuffers,
    mut upload: impl FnMut(&[u8]) -> Result<(), String>,
) -> Result<bool, String> {
    const SMALL: &str = "the background's picture is smaller than the screen";
    match pending {
        Pending::Buffer { handle, generation } => {
            let now = Held::Buffer { handle, generation };
            if *held == Some(now) {
                return Ok(false);
            }
            crate::gfx::picture::with_buffer_unchanged(buffers, handle, |picture| {
                upload(picture.get(..bytes).ok_or(SMALL)?)
            })
            .map_err(|e| e.0.to_string())??;
            *held = Some(now);
        }
        Pending::Copy(picture) => {
            upload(picture.get(..bytes).ok_or(SMALL)?)?;
            *held = None;
        }
        Pending::Rescaled { generation, pixels } => {
            let now = Held::Rescaled { generation };
            if *held == Some(now) {
                return Ok(false);
            }
            upload(pixels.get(..bytes).ok_or(SMALL)?)?;
            *held = Some(now);
        }
    }
    Ok(true)
}

/// Whether a background was noted since the last frame's end check (`recorded_this_frame`).
static RECORDED: AtomicBool = AtomicBool::new(false);

/// Notes picture buffer `handle` as the frame's background, for the GPU to draw under the canvas.
/// False when the caller must copy it onto the canvas as before: no GPU backend, a canvas that is
/// not 32-bit rows of exactly the screen's width, or a buffer too small for it.
pub fn defer(buffers: &crate::gfx::picture::PictureBuffers, target: &Target, handle: u32) -> bool {
    use crate::gfx::picture;
    defer_with(target, || {
        let (Some(generation), Ok(len)) = (
            picture::generation(buffers, handle),
            picture::with_buffer_unchanged(buffers, handle, <[u8]>::len),
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
pub fn defer_pixels(target: &Target, generation: u64, pixels: &std::sync::Arc<Vec<u8>>) -> bool {
    defer_with(target, || {
        (pixels.len() >= target.width * target.height * 4).then(|| Pending::Rescaled {
            generation,
            pixels: std::sync::Arc::clone(pixels),
        })
    })
}

/// Notes the background `source` gives, when the GPU can keep it for `target`.
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

/// The noted background, taken: the flush draws it now.
pub fn take() -> Option<Pending> {
    PENDING.lock().unwrap().take()
}

/// The noted background dropped (the canvas is written over it, or the GPU is given up).
#[cfg_attr(handheld, allow(dead_code))] // the handheld's GPU is never given up
pub fn clear() {
    *PENDING.lock().unwrap() = None;
}

/// Whether a background was noted since the last call: a menu frame with one stays on the GPU
/// even with no 3D in it.
pub(super) fn recorded_this_frame() -> bool {
    RECORDED.swap(false, Relaxed)
}

/// Picture buffer `handle` is about to change (or go): a noted background that names it takes
/// the bytes first (`bytes` as they are now).
pub fn before_change(handle: u32, bytes: &[u8]) {
    let mut pending = PENDING.lock().unwrap();
    if matches!(*pending, Some(Pending::Buffer { handle: noted, .. }) if noted == handle) {
        *pending = Some(Pending::Copy(bytes.to_vec()));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `bring_up` with an upload that keeps what it was given.
    fn bring_up_kept(
        pending: Pending,
        held: &mut Option<Held>,
        bytes: usize,
        buffers: &crate::gfx::picture::PictureBuffers,
        sent: &std::cell::RefCell<Vec<Vec<u8>>>,
    ) -> Result<bool, String> {
        bring_up(pending, held, bytes, buffers, |picture| {
            sent.borrow_mut().push(picture.to_vec());
            Ok(())
        })
    }

    #[test]
    fn a_background_is_uploaded_only_when_the_texture_does_not_hold_it() {
        use crate::gfx::picture;
        let _lock = crate::test_support::test_lock();
        let figures = crate::memory::figures();
        let memory = crate::memory::PlainList::empty();
        let buffers = &picture::PictureBuffers::new();
        let n = picture::take(buffers, picture::Buffer::Key, &memory, 0x100).unwrap();
        picture::write_bytes(buffers, n, 0, &[7]).unwrap();
        let sent = std::cell::RefCell::new(Vec::new());
        let mut held = None;
        let buffer = |generation| Pending::Buffer {
            handle: n,
            generation,
        };
        let first = picture::generation(buffers, n).unwrap();
        assert_eq!(
            bring_up_kept(buffer(first), &mut held, 8, buffers, &sent),
            Ok(true)
        );
        assert_eq!(sent.borrow()[0], [7, 0, 0, 0, 0, 0, 0, 0]);
        assert_eq!(
            bring_up_kept(buffer(first), &mut held, 8, buffers, &sent),
            Ok(false)
        );
        assert_eq!(sent.borrow().len(), 1, "the same picture is not sent again");
        picture::write_bytes(buffers, n, 1, &[8]).unwrap();
        let second = picture::generation(buffers, n).unwrap();
        assert_eq!(
            bring_up_kept(buffer(second), &mut held, 8, buffers, &sent),
            Ok(true)
        );
        assert_eq!(sent.borrow()[1][..2], [7, 8]);
        // A picture smaller than the screen is refused, and the texture still holds the last.
        let third = Pending::Copy(vec![1; 4]);
        assert!(bring_up_kept(third, &mut held, 8, buffers, &sent).is_err());
        assert!(
            held == Some(Held::Buffer {
                handle: n,
                generation: second
            })
        );
        // A copy is always sent, and is no picture buffer's state.
        let copy = Pending::Copy(vec![5; 9]);
        assert_eq!(bring_up_kept(copy, &mut held, 8, buffers, &sent), Ok(true));
        assert_eq!((sent.borrow()[2].len(), held.is_none()), (8, true));
        // A rescale by its generation, the pixels shared.
        let pixels = std::sync::Arc::new(vec![3u8; 8]);
        let rescaled = |generation| Pending::Rescaled {
            generation,
            pixels: std::sync::Arc::clone(&pixels),
        };
        assert_eq!(
            bring_up_kept(rescaled(4), &mut held, 8, buffers, &sent),
            Ok(true)
        );
        assert_eq!(
            bring_up_kept(rescaled(4), &mut held, 8, buffers, &sent),
            Ok(false)
        );
        assert_eq!(
            bring_up_kept(rescaled(5), &mut held, 8, buffers, &sent),
            Ok(true)
        );
        assert_eq!(sent.borrow().len(), 5);
        picture::give_back(buffers, picture::Buffer::Key);
        crate::memory::with_figures(|f| *f = figures);
    }

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
