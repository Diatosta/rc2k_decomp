//! The race's ticks and its stops (from `drive::race_tick`
//! 0x4455A9, `drive::race_step` 0x4454D2, `drive::quit_question` 0x445069,
//! `drive::pause_until_key` 0x445376 and `drive::tyre_change_screen` 0x445047): which
//! ticks run the race's step, the ticks of a frame, the "quit?" question's answers and
//! the best frame kept, the keys that end a pause, and when the tyre-change screen shows.

/// The fatal error a tick raises with the first car block's +0x260 at 0.
pub const NO_CAR_FATAL: u32 = 0x8060_0655;

/// `drive::race_tick`: whether tick `tick` (already counted) runs the step, by
/// `RaceView.after_credits`: 0 every tick, 1 every second (odd ticks), more every fourth (ticks
/// with bits 0 and 1 set).
pub fn tick_runs(tick: u32, every: u32) -> bool {
    every == 0 || (tick & 1 != 0 && (every == 1 || tick & 2 != 0))
}

/// `drive::race_step`'s ticks: from the tenths gone (`PhysicsTables.race_tenths`, signed) while above
/// 0, `tick(counter)` (the tick, then whether the players are all out, and the counter
/// as that leaves it) until the players are all out; the counter less one after each.
/// The counter left.
pub fn run_ticks<E>(
    tenths: u32,
    mut tick: impl FnMut(u32) -> Result<(bool, u32), E>,
) -> Result<u32, E> {
    let mut counter = tenths;
    while (counter as i32) > 0 {
        let (all_out, left) = tick(counter)?;
        counter = left;
        if all_out {
            break;
        }
        counter = counter.wrapping_sub(1);
    }
    Ok(counter)
}

/// `drive::quit_question`'s best whole frame: the first block's clock (+8) / 100 less 7,
/// and the remainder.
pub fn best_frame(clock: u32) -> (u32, u32) {
    ((clock / 100).wrapping_sub(7), clock % 100)
}

/// Whether the best frame is kept over `kept`: not below it, by the
/// sign of the difference.
pub fn keeps_frame(frame: u32, kept: u32) -> bool {
    (frame.wrapping_sub(kept) as i32) >= 0
}

/// The question's notice: 0xC2 with the third answer offered, else 0xC1.
pub fn quit_notice(third: bool) -> u32 {
    if third { 0xC2 } else { 0xC1 }
}

/// The answer to a key from `input::yes_no_key`: 1 (yes) -1, -1 (no) 0, 2 outside race
/// mode 2 with the third answer offered -2; any other none yet.
pub fn quit_answer(key: u32, mode: u32, third: bool) -> Option<u32> {
    match key {
        1 => Some(u32::MAX),
        u32::MAX => Some(0),
        2 if mode != 2 && third => Some(0xFFFF_FFFE),
        _ => None,
    }
}

/// `drive::pause_until_key`: whether a key (scan code 1 .. 0x7F) down ends the pause:
/// any but Ctrl (0x1D), Alt (0x38), Tab (0x0F), F1 .. F10 (0x3B ..= 0x44) and F11 (0x57).
pub fn ends_pause(key: usize) -> bool {
    !matches!(key, 0x1D | 0x38 | 0x0F | 0x3B..=0x44 | 0x57)
}

/// The pause's wait while the window is not active (from 0x4453EE): held at 10 when it
/// is 10 or less (unsigned), else kept.
pub fn held_wait(wait: u32) -> u32 {
    if wait <= 10 { 10 } else { wait }
}

/// `drive::tyre_change_screen`: whether a tyre change is under way (the first block's
/// +0x26C not 0 or -1).
pub fn tyre_change_shown(change: u32) -> bool {
    change != 0 && change != u32::MAX
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ticks() {
        assert!((1..9).all(|t| tick_runs(t, 0)));
        assert_eq!(
            (1..9).filter(|&t| tick_runs(t, 1)).collect::<Vec<_>>(),
            [1, 3, 5, 7]
        );
        assert_eq!(
            (1..9).filter(|&t| tick_runs(t, 2)).collect::<Vec<_>>(),
            [3, 7]
        );
        let mut n = 0;
        let left = run_ticks::<()>(3, |c| {
            n += 1;
            Ok((false, c))
        });
        assert_eq!((left, n), (Ok(0), 3));
        let left = run_ticks::<()>(3, |c| Ok((c == 2, c)));
        assert_eq!(left, Ok(2));
        assert_eq!(run_ticks::<()>(0x8000_0000, |_| panic!()), Ok(0x8000_0000));
    }

    #[test]
    fn stops() {
        assert_eq!(best_frame(1234), (5, 34));
        assert_eq!(best_frame(0), (u32::MAX - 6, 0));
        assert!(keeps_frame(5, 5) && !keeps_frame(4, 5));
        assert_eq!(quit_notice(true), 0xC2);
        assert_eq!(quit_answer(1, 0, false), Some(u32::MAX));
        assert_eq!(quit_answer(u32::MAX, 0, false), Some(0));
        assert_eq!(quit_answer(2, 1, true), Some(0xFFFF_FFFE));
        assert_eq!(quit_answer(2, 2, true), None);
        assert_eq!(quit_answer(0, 0, true), None);
        assert!(ends_pause(1) && !ends_pause(0x3B) && !ends_pause(0x57) && ends_pause(0x45));
        assert_eq!(held_wait(30), 30);
        assert_eq!(held_wait(3), 10);
        assert!(tyre_change_shown(1) && !tyre_change_shown(0) && !tyre_change_shown(u32::MAX));
    }
}
