//! The quad check (`RC2K_QUAD_CHECK`): the GPU backend draws each frame twice, once as quads and
//! once the canvas way, and compares the pictures. This is its recorder side: what the funnels
//! wrote on the screen since the GPU last took it. (`quad_check_none.rs` is the handheld's, which
//! has no such check.)

use super::{quads, recorder};
use std::sync::OnceLock;

/// What the environment asks of the check.
struct Asked {
    /// `RC2K_QUAD_CHECK` is set.
    check: bool,
    /// `RC2K_QUAD_CHECK=2d`: both drawings leave out the 3D, so that only the 2D is compared
    /// (the GPU's drawing of the 3D differs from itself by a few pixels a frame).
    only_2d: bool,
}

fn asked() -> &'static Asked {
    static ASKED: OnceLock<Asked> = OnceLock::new();
    ASKED.get_or_init(|| Asked {
        check: std::env::var_os("RC2K_QUAD_CHECK").is_some(),
        only_2d: std::env::var("RC2K_QUAD_CHECK").is_ok_and(|v| v == "2d"),
    })
}

/// Whether `RC2K_QUAD_CHECK` is set (and quads are drawn): the GPU backend draws each frame
/// twice and compares.
pub fn wanted() -> bool {
    quads::drawn() && asked().check
}

/// Whether the check leaves the 3D out of both drawings (`RC2K_QUAD_CHECK=2d`).
pub fn only_2d() -> bool {
    wanted() && asked().only_2d
}

/// What the funnels wrote on the screen since the last `take`, as the pixels the canvas would
/// hold, the overlay's "unwritten" pixel where nothing.
pub(super) struct Interval(Vec<u8>);

impl Interval {
    pub const fn new() -> Interval {
        Interval(Vec::new())
    }

    /// Notes `bytes` written at `at` from the screen's start, the screen being `screen_len`
    /// bytes long; nothing unless the check is wanted.
    pub fn note(&mut self, screen_len: usize, at: usize, bytes: &[u8]) {
        if !wanted() {
            return;
        }
        if self.0.is_empty() {
            let mut blank = vec![0u8; screen_len];
            blank
                .as_chunks_mut::<4>()
                .0
                .fill(rc2k_core::gfx::atlas::UNWRITTEN_PIXEL.to_le_bytes());
            self.0 = blank;
        }
        if let Some(place) = self.0.get_mut(at..at.wrapping_add(bytes.len())) {
            place.copy_from_slice(bytes);
        }
    }
}

/// What the funnels wrote since the last call, as the canvas would hold it; `None` when nothing
/// was.
pub fn take_interval() -> Option<Vec<u8>> {
    let interval = std::mem::take(&mut recorder().interval.0);
    (!interval.is_empty()).then_some(interval)
}
