//! The render states a noted fan carries, shared between the fans that have the same ones.
//!
//! A scene changes a state again and again (blending on for one polygon, off for the next) and
//! goes back to values it had a moment ago, so the sets that come up are few. A set already
//! handed out recently is handed out again instead of making another one.

use crate::render::STATES;
use std::sync::Arc;

/// Render states as a fan carries them: the same allocation for every fan that shares them.
pub type States = Arc<[u32; STATES]>;

/// How many sets are kept in mind.
const RECENT: usize = 8;

/// The sets handed out last, the oldest replaced first.
pub struct SharedStates {
    recent: [Option<States>; RECENT],
    /// The slot the next new set goes into.
    next: usize,
}

impl SharedStates {
    pub const fn new() -> Self {
        Self {
            recent: [const { None }; RECENT],
            next: 0,
        }
    }

    /// The shared set of `kept`: one handed out recently when it has the same values, else a new
    /// one (which is remembered).
    pub fn of(&mut self, kept: [u32; STATES]) -> States {
        if let Some(same) = self.recent.iter().flatten().find(|set| ***set == kept) {
            return Arc::clone(same);
        }
        let set = Arc::new(kept);
        self.recent[self.next] = Some(Arc::clone(&set));
        self.next = (self.next + 1) % RECENT;
        set
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn with(state: usize, value: u32) -> [u32; STATES] {
        let mut kept = [0; STATES];
        kept[state] = value;
        kept
    }

    #[test]
    fn equal_sets_share_and_different_ones_do_not() {
        let mut shared = SharedStates::new();
        let a = shared.of(with(3, 1));
        let b = shared.of(with(3, 2));
        assert!(!Arc::ptr_eq(&a, &b));
        assert_eq!((a[3], b[3]), (1, 2));
        // Going back to the first values hands out the first set again.
        assert!(Arc::ptr_eq(&a, &shared.of(with(3, 1))));
        assert!(Arc::ptr_eq(&b, &shared.of(with(3, 2))));
    }

    #[test]
    fn the_oldest_set_is_forgotten_first() {
        let mut shared = SharedStates::new();
        let first = shared.of(with(0, 100));
        for n in 0..RECENT as u32 {
            shared.of(with(0, n));
        }
        // Eight others came after: the first is made again, with the same values.
        let again = shared.of(with(0, 100));
        assert!(!Arc::ptr_eq(&first, &again));
        assert_eq!(*first, *again);
    }
}
