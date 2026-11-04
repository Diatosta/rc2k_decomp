//! An event's set-ups of the drivers and players and its pages' numbers (converted
//! 2026-10-21 from `menu::reset_drivers` 0x470399, `clear_result_slots` 0x470DDA,
//! `ready_players` 0x471F7E, `reset_player_totals` 0x471FB8, `begin_event` 0x472BAA,
//! `reset_driver_condition` 0x47E7BC, `reset_player_times` 0x4813D6,
//! `reset_single_event` 0x4AAA2F, `add_player_times` 0x4A25AD, `player_entered` 0x4A95EB,
//! `mark_stage_choice` 0x486C21, `count_stage_pages` 0x480032, `first_page_stage`
//! 0x4A2440, `stage_range_caption` 0x4A1FBB; the computer drivers' times of
//! `calc_cpu_times` 0x470950). The drivers (`g_drivers`, 0x94 each) and the
//! players' records are written by offset, as far as their fields are worked out: each
//! step's stores, (offset, value), in the original's order.

use super::standings::Store;
use crate::strings::format::{Pad, decimal};

/// No time yet: an hour in milliseconds.
pub const HOUR: u32 = 3_600_000;

/// `menu::begin_event` and `reset_single_event`: each player's +0x8C and +0x84 0, +0xD0 1.
pub const PLAYER_BEGIN: [Store; 3] = [(0x8C, 0), (0x84, 0), (0xD0, 1)];

/// `menu::reset_player_times`: each player's time (+0x88) none yet, +0xD0 6.
pub const PLAYER_TIMES: [Store; 2] = [(0x88, HOUR), (0xD0, 6)];

/// `menu::reset_player_totals`: each counted player's total (+0x8C) and time none yet.
pub const PLAYER_TOTALS: [Store; 2] = [(0x8C, HOUR), (0x88, HOUR)];

/// `menu::ready_players`: each counted player taking part again (+0xD0 1), time and
/// figure (+0x88, +0x94) 0.
pub const PLAYER_READY: [Store; 3] = [(0xD0, 1), (0x88, 0), (0x94, 0)];

/// `menu::clear_result_slots`: fourteen result slots from `at` (a player's +0x98, a
/// driver's +0x5C) emptied: -1, but 0 in the seventh and the last (two groups of six
/// sections' points and their total: a player's stage points then overall points, a
/// driver's overall then stage points).
pub fn result_slots(at: usize) -> [Store; 14] {
    std::array::from_fn(|k| (at + 4 * k, empty_result_slot(k)))
}

/// Result slot `k` emptied (`result_slots`).
pub fn empty_result_slot(k: usize) -> u32 {
    if k == 6 || k == 13 { 0 } else { u32::MAX }
}

/// The event mode that keeps the players' setups (`menu::begin_event`).
pub const KEEPS_SETUP: u32 = 3;

/// `menu::begin_event`: a player's setup's defaults: gear ratio 2, brake balance 0x32,
/// tyre 3, ride height 0x46, suspension 0x1E, steering sensitivity the controller's
/// (+0x280, `sensitivity`).
pub fn setup_defaults(sensitivity: u32) -> [Store; 6] {
    [
        (0x120, 2),
        (0x110, 0x32),
        (0x10C, 3),
        (0x114, 0x46),
        (0x118, 0x1E),
        (0x11C, sensitivity),
    ]
}

/// `menu::add_player_times`: a player that took part (+0xD0 1) has its stage time added
/// to its total (+0x8C), another both none yet.
pub fn added_time(entered: u32, time: u32, total: u32) -> Vec<Store> {
    if entered == 1 {
        vec![(0x8C, total.wrapping_add(time))]
    } else {
        vec![(0x88, HOUR), (0x8C, HOUR)]
    }
}

/// Which stage the stage screens' map shows (`dword_755700`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MapStage {
    /// The stage choice's (`menu::mark_stage_choice`).
    Choice = 1,
    /// A championship's.
    Championship = 2,
    /// A record's.
    Record = 3,
}

/// The most pages `count_stage_pages` takes.
pub const MOST_PAGES: u32 = 256;

/// `menu::count_stage_pages`: how many records after the stage's (`kind(n)` the first
/// dword of the n-th after it) have a first dword 0, in a row. None past `MOST_PAGES`.
pub fn stage_pages(mut kind: impl FnMut(u32) -> u32) -> Option<u32> {
    let mut n = 0u32;
    while kind(n + 1) == 0 {
        n += 1;
        if n > MOST_PAGES {
            return None;
        }
    }
    Some(n)
}

/// The stages' last page, ten a page.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LastPage {
    /// Its first stage.
    pub first: u32,
    /// The stages past the last whole ten.
    pub rest: u32,
}

/// `menu::first_page_stage`: the last page of `count` stages: None up to ten (signed: the
/// first page, stage 1); else from (count / 10) x 10 + 1, less ten for a multiple of ten.
pub fn last_page(count: u32) -> Option<LastPage> {
    if count as i32 <= 10 {
        return None;
    }
    let rest = count % 10;
    let first = (count / 10) * 10 + 1;
    Some(LastPage {
        first: if rest == 0 { first - 10 } else { first },
        rest,
    })
}

/// `menu::stage_range_caption`: "first-last" for a page from `first` (it plus 9), no NUL.
pub fn stage_range(first: u32) -> Vec<u8> {
    let mut text = decimal(first, Pad::None);
    text.push(b'-');
    text.extend(decimal(first.wrapping_add(9), Pad::None));
    text
}

/// The computer drivers' spread (`menu::calc_cpu_times`): whether the next is the first
/// (`dword_755944` 1) and the step (`dword_755948`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Spread {
    pub first: u32,
    pub step: u32,
}

impl Spread {
    /// The next driver's spread: a random number under 3000, plus (unless the first) one
    /// under 1000 times the step, which then goes up by one and the first is past; for the
    /// first the step 0. `random(limit)` draws the numbers, in the original's order.
    pub fn next<E>(&mut self, mut random: impl FnMut(u32) -> Result<u32, E>) -> Result<u32, E> {
        let spread = random(3000)?;
        if self.first == 1 {
            self.step = 0;
            return Ok(spread);
        }
        let more = random(1000)?;
        self.first = 0;
        let spread = spread.wrapping_add(more.wrapping_mul(self.step));
        self.step = self.step.wrapping_add(1);
        Ok(spread)
    }
}

/// The gap between a class's computer drivers: half the class's time over its drivers
/// (`count` not 0: the original divides by it).
pub fn cpu_gap(class_time: u32, count: u32) -> u32 {
    (class_time >> 1) / count
}

/// A computer driver's time: its spread, the gaps of the class's drivers before it and the
/// class's time.
pub fn cpu_time(spread: u32, gaps: u32, class_time: u32) -> u32 {
    spread.wrapping_add(gaps).wrapping_add(class_time)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stores() {
        let s = result_slots(0x98);
        assert_eq!(
            (s[0], s[6], s[13]),
            ((0x98, u32::MAX), (0xB0, 0), (0xCC, 0))
        );
        assert_eq!(setup_defaults(9)[5], (0x11C, 9));
        assert_eq!(added_time(1, 5, u32::MAX), [(0x8C, 4)]);
        assert_eq!(added_time(2, 5, 7), [(0x88, HOUR), (0x8C, HOUR)]);
        assert_eq!(MapStage::Choice as u32, 1);
    }

    #[test]
    fn computer_times() {
        let mut draws = [7u32, 3, 5, 2].into_iter();
        let mut random = |_| Ok::<u32, ()>(draws.next().unwrap());
        let mut s = Spread { first: 1, step: 9 };
        assert_eq!(s.next(&mut random), Ok(7));
        assert_eq!(s, Spread { first: 1, step: 0 });
        s.first = 0;
        s.step = 4;
        assert_eq!(s.next(&mut random), Ok(3 + 5 * 4));
        assert_eq!(s, Spread { first: 0, step: 5 });
        assert_eq!((cpu_gap(101, 5), cpu_time(1, 2, 3)), (10, 6));
    }

    #[test]
    fn pages() {
        let kinds = [9, 0, 0, 0, 5];
        assert_eq!(stage_pages(|n| kinds[n as usize]), Some(3));
        assert_eq!(stage_pages(|_| 0), None);
        assert_eq!(last_page(10), None);
        assert_eq!(last_page(0xFFFF_FFF0), None);
        assert_eq!(last_page(11), Some(LastPage { first: 11, rest: 1 }));
        assert_eq!(last_page(20), Some(LastPage { first: 11, rest: 0 }));
        assert_eq!(last_page(37), Some(LastPage { first: 31, rest: 7 }));
        assert_eq!(stage_range(11), b"11-20");
    }
}
