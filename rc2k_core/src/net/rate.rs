//! Keeping a peer from filling the log: of each kind of line, one in each period, with the
//! number of those left out since the last one.

use std::collections::HashMap;
use std::time::Duration;

/// Most kinds kept at a time (a peer cannot grow the map by naming new kinds).
const KINDS_MOST: usize = 256;

#[derive(Default)]
pub struct LineLimit {
    kinds: HashMap<String, (Duration, u32)>,
}

impl LineLimit {
    /// Whether a line of `kind` is to be written at `now` (the time since any fixed start): the first, and then one after each
    /// `period`. Written, it gives the number of lines of the kind left out since the last.
    pub fn due(&mut self, kind: &str, now: Duration, period: Duration) -> Option<u32> {
        if let Some((last, left_out)) = self.kinds.get_mut(kind) {
            if now.saturating_sub(*last) < period {
                *left_out = left_out.saturating_add(1);
                return None;
            }
            *last = now;
            return Some(std::mem::take(left_out));
        }
        if self.kinds.len() >= KINDS_MOST {
            // Kinds not heard from for a period are forgotten; the rest are not written about.
            self.kinds
                .retain(|_, (last, _)| now.saturating_sub(*last) < period);
            if self.kinds.len() >= KINDS_MOST {
                return None;
            }
        }
        self.kinds.insert(kind.to_string(), (now, 0));
        Some(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_kind_is_written_once_a_period_with_the_count_left_out() {
        let period = Duration::from_secs(10);
        let start = Duration::from_secs(100);
        let mut limit = LineLimit::default();
        assert_eq!(limit.due("a", start, period), Some(0));
        for second in 1..=5 {
            assert_eq!(
                limit.due("a", start + Duration::from_secs(second), period),
                None
            );
        }
        // Another kind has its own period.
        assert_eq!(
            limit.due("b", start + Duration::from_secs(5), period),
            Some(0)
        );
        assert_eq!(limit.due("a", start + period, period), Some(5));
        assert_eq!(limit.due("a", start + period, period), None);
        assert_eq!(limit.due("a", start + period * 2, period), Some(1));
    }

    #[test]
    fn new_kinds_cannot_grow_the_map_without_bound() {
        let period = Duration::from_secs(10);
        let start = Duration::from_secs(100);
        let mut limit = LineLimit::default();
        for n in 0..KINDS_MOST * 3 {
            let _ = limit.due(&n.to_string(), start, period);
        }
        assert!(limit.kinds.len() <= KINDS_MOST);
        // A period later the old kinds are forgotten and new ones are written again.
        assert_eq!(limit.due("fresh", start + period, period), Some(0));
    }
}
