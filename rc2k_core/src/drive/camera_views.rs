//! A car block's camera view: the view wanted taken, and the next one the change-view key asks for:
//! views 0 .. 6 cycle, 2 and 3 only for cars with their own views (`kind_has_views`) and with one
//! player; 7 and above are the fixed replay cameras.

/// A car block's view fields (block offsets +0x684 .. +0x69C).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CameraView {
    /// The view shown.
    pub view: u32,
    /// The view wanted.
    pub wanted: u32,
    /// The step count when it was taken.
    pub taken_at: u32,
    /// Held (no change).
    pub held: u32,
    /// Not known; cleared when a view is taken.
    pub w698: u32,
    /// The share of the switch to the view made (a float).
    pub switch_share: f32,
}

/// The views (0 .. 6); 2 and 3 only for cars that have them.
pub const VIEWS: u32 = 7;
/// The cars with an entry in the table of who has views 2 and 3.
pub(crate) const CARS_WITH_ENTRY: u32 = 0x22;

impl CameraView {
    /// Takes the wanted view at step count `step` and gives `step` back.
    pub fn take_wanted(&mut self, step: u32) -> u32 {
        self.view = self.wanted;
        self.held = 0;
        self.switch_share = 0.0;
        self.w698 = 0;
        self.taken_at = step;
        step
    }
}

/// The view after `view` for the change-view key. 2 and 3 become 4 when the car (`car()`, the
/// block's +0x24) has none (past the table or `has_views(car)` false) or with two players
/// (`players()`, asked only when it has them); 7 and above wrap to 0.
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
        if !has || players() >= 2 {
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
