//! The screen quad's inputs, a view's own working values: the colour, the texture coordinates and
//! the corners (`corners`: [0] left, [1] right, [4] top, [5] bottom), each view's own. One
//! per view slot (`race::view_scratch::ViewScratch::quad`).

use crate::race::plain_slot::{Merge, PlainSlot, merge_run};

/// The quad's values, plain.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct QuadWork {
    /// The quad's colour and see-through (`scene::draw_flat_quad`, `scene::draw_tinted_quad`).
    pub colour: [u32; 2],
    /// The quad's texture coordinates (`race::frame_flush`, `hud::draw_view_strip`).
    pub uv: [u32; 8],
    /// The quad's corners.
    pub corners: [u32; 8],
}

impl QuadWork {
    pub const fn new() -> Self {
        Self {
            colour: [0; 2],
            uv: [0; 8],
            corners: [0; 8],
        }
    }
}

impl Merge for QuadWork {
    fn merge_changes(&mut self, now: &Self, start: &Self) {
        merge_run(&mut self.colour, &now.colour, &start.colour);
        merge_run(&mut self.uv, &now.uv, &start.uv);
        merge_run(&mut self.corners, &now.corners, &start.corners);
    }
}

/// A view's quad values.
pub type QuadSlot = PlainSlot<QuadWork>;

#[cfg(test)]
mod tests {
    use super::*;

    /// A merge puts over only what the copy changed since it was taken.
    #[test]
    fn a_merge_puts_over_what_the_copy_changed() {
        let (view, copy, taken) = (
            PlainSlot::new(QuadWork::new()),
            PlainSlot::new(QuadWork::new()),
            PlainSlot::new(QuadWork::new()),
        );
        view.borrow_mut().corners[0] = 5;
        view.borrow_mut().uv[4] = 7;
        copy.copy_from(&view);
        taken.copy_from(&view);
        view.borrow_mut().uv[4] = 2;
        copy.borrow_mut().corners[0] = 9;
        copy.borrow_mut().colour[1] = 3;
        view.merge_changes(&copy, &taken);
        let now = *view.borrow();
        assert_eq!((now.corners[0], now.uv[4], now.colour[1]), (9, 2, 3));
    }
}
