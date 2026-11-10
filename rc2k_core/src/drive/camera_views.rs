//! A car block's camera view (converted 2026-10-21 from `drive::set_camera_view`
//! 0x43A9BA and `next_camera_view` 0x43AA38): the view wanted taken, and the next one
//! the change-view key asks for: views 0 .. 6 cycle, 2 and 3 only for cars with their
//! own views (`kind_has_views`) and with one player; 7 and above are the fixed replay cameras.

/// The block's view fields (+0x684 .. +0x69C).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CameraView {
    /// +0x684: the view shown.
    pub view: u32,
    /// +0x688: the view wanted.
    pub wanted: u32,
    /// +0x690: the step count when it was taken.
    pub taken_at: u32,
    /// +0x694: held (no change).
    pub held: u32,
    /// +0x698.
    pub w698: u32,
    /// +0x69C.
    pub w69c: u32,
}

/// The views (0 .. 6); 2 and 3 only for cars that have them.
pub const VIEWS: u32 = 7;
/// The cars with an entry in the table of who has views 2 and 3.
pub const CARS_WITH_ENTRY: u32 = 0x22;

impl CameraView {
    /// `drive::set_camera_view`: the view wanted taken at step count `step`; the step
    /// count (the original's eax).
    pub fn take_wanted(&mut self, step: u32) -> u32 {
        self.view = self.wanted;
        self.held = 0;
        self.w69c = 0;
        self.w698 = 0;
        self.taken_at = step;
        step
    }
}

/// `drive::next_camera_view`'s choice: the view after `view`, but 2 and 3 become 4 when
/// the car (`car()`, the block's +0x24) has none (past the table, or `has_views(car)`
/// false) or with two players (`players()` 2, asked only when it has them); 7 and above
/// wrap to 0.
pub fn next_view(
    view: u32,
    car: impl FnOnce() -> u32,
    has_views: impl FnOnce(u32) -> bool,
    players: impl FnOnce() -> u32,
) -> u32 {
    let mut next = view.wrapping_add(1);
    if (2..=3).contains(&next) {
        let car = car();
        let has = car < CARS_WITH_ENTRY && has_views(car);
        if !has || players() == 2 {
            next = 4;
        }
    }
    if next >= VIEWS {
        next = 0;
    }
    next
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn views() {
        assert_eq!(
            next_view(0, || unreachable!(), |_| unreachable!(), || unreachable!()),
            1
        );
        assert_eq!(next_view(1, || 3, |_| true, || 1), 2);
        assert_eq!(next_view(1, || 3, |_| true, || 2), 4);
        assert_eq!(next_view(2, || 3, |_| false, || unreachable!()), 4);
        assert_eq!(
            next_view(2, || 0x22, |_| unreachable!(), || unreachable!()),
            4
        );
        assert_eq!(next_view(6, || 0, |_| true, || 1), 0);
        assert_eq!(next_view(u32::MAX, || 0, |_| true, || 1), 0);
        let mut v = CameraView {
            view: 1,
            wanted: 5,
            held: 1,
            w698: 40,
            ..Default::default()
        };
        assert_eq!(v.take_wanted(9), 9);
        assert_eq!(
            v,
            CameraView {
                view: 5,
                wanted: 5,
                taken_at: 9,
                ..Default::default()
            }
        );
    }
}
