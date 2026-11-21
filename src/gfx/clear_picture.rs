//! Fills the picture buffer with zeros: 800 x 600 dwords at the buffer, whatever size the picture
//! in it has. Refuses a null buffer pointer.

use crate::errors::BadInput;
use crate::gfx::owner::Pictures;
use crate::gfx::picture::{PICTURE_DWORDS, has_picture, with_picture};

pub(super) fn clear_picture(pictures: &Pictures) -> Result<(), BadInput> {
    if !has_picture(pictures) {
        return Err(BadInput("no picture buffer"));
    }
    with_picture(pictures, |b| {
        b.get_mut(..PICTURE_DWORDS * 4).map(|p| p.fill(0))
    })?
    .ok_or(BadInput("a picture buffer shorter than a screen"))
}

// Written out: the buffer is nearly two megabytes, and what the function leaves is known without
// running it. It writes zeros.
