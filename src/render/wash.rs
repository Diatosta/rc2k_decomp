//! The menu's background kept for the wash beside the menu's frame (`rc2k_core::gfx::
//! screen_wash`). The picture a menu frame is built over is noted when it is drawn (the
//! background alone: no 3D, no texts, no buttons); the wash made of it is made again only when
//! the picture or the window's size changes, never per frame.

use std::sync::atomic::{AtomicU64, Ordering::Relaxed};
use std::sync::{Arc, Mutex};

use rc2k_core::gfx::screen_wash::{Wash, make};

/// What names the noted picture's state, so that the same picture is not taken again.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Key {
    /// Picture buffer `handle` at change counter `generation`.
    Buffer { handle: u32, generation: u64 },
    /// The loading picture's rescale, by its generation.
    Rescaled { generation: u64 },
}

/// A wash made for a window's size, with a number no other wash of this run has.
pub struct Image {
    pub version: u64,
    pub wash: Wash,
}

struct State {
    key: Option<Key>,
    /// The noted picture: 32-bit pixels, rows with no gaps.
    picture: Option<Arc<Vec<u8>>>,
    size: (u32, u32),
    /// The wash of the picture for a window of the size.
    made: Option<((u32, u32), Arc<Image>)>,
}

static STATE: Mutex<State> = Mutex::new(State {
    key: None,
    picture: None,
    size: (0, 0),
    made: None,
});

static VERSIONS: AtomicU64 = AtomicU64::new(1);

/// Picture buffer `handle` (`rows` rows of `row_bytes` bytes, with no gaps) is the menu's
/// background now.
pub fn note_buffer(
    buffers: &crate::gfx::picture::PictureBuffers,
    handle: u32,
    rows: u32,
    row_bytes: usize,
) {
    use crate::gfx::picture;
    let Some(generation) = picture::generation(buffers, handle) else {
        return;
    };
    let key = Key::Buffer { handle, generation };
    let mut state = STATE.lock().unwrap();
    if state.key == Some(key) {
        return;
    }
    let wanted = rows as usize * row_bytes;
    let copied = picture::with_buffer_unchanged(buffers, handle, |bytes| {
        bytes.get(..wanted).map(<[u8]>::to_vec)
    });
    if let Ok(Some(picture)) = copied {
        *state = State {
            key: Some(key),
            picture: Some(Arc::new(picture)),
            size: ((row_bytes / 4) as u32, rows),
            made: None,
        };
    }
}

/// The loading picture rescaled to the screen (`width` x `height`) is the menu's background now;
/// `generation` names the rescale.
pub fn note_pixels(generation: u64, pixels: &Arc<Vec<u8>>, width: u32, height: u32) {
    let key = Key::Rescaled { generation };
    let mut state = STATE.lock().unwrap();
    if state.key != Some(key) {
        *state = State {
            key: Some(key),
            picture: Some(Arc::clone(pixels)),
            size: (width, height),
            made: None,
        };
    }
}

/// The wash of the noted background for a window of `window` pixels; `None` before any
/// background is noted. Made when the background or the window's size has changed.
pub fn image_for(window: (u32, u32)) -> Option<Arc<Image>> {
    let mut state = STATE.lock().unwrap();
    if let Some((size, image)) = &state.made
        && *size == window
    {
        return Some(Arc::clone(image));
    }
    let wash = make(state.picture.as_ref()?, state.size, window)?;
    let image = Arc::new(Image {
        version: VERSIONS.fetch_add(1, Relaxed),
        wash,
    });
    state.made = Some((window, Arc::clone(&image)));
    Some(image)
}
