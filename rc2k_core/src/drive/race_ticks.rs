//! The race's ticks and its stops: which ticks run the race's step, the ticks of a frame, the
//! "quit?" question's answers and the best frame kept, the keys that end a pause, and when the
//! tyre-change screen shows.

/// The fatal error a tick raises with the first car block's +0x260 at 0.
pub const NO_CAR_FATAL: u32 = 0x8060_0655;

/// Whether tick `tick` (already counted) runs the race's step, by `RaceView.after_credits`
/// (`every`): 0 every tick, 1 every second (odd ticks), more every fourth (bits 0 and 1 set).
pub fn tick_runs(tick: u32, every: u32) -> bool {
    every == 0 || (tick & 1 != 0 && (every == 1 || tick & 2 != 0))
}

/// A frame's ticks: from the tenths gone (`PhysicsTables.race_tenths`, signed), `tick(counter)`
/// runs while the counter is above 0 and the players are not all out; it gives (all out, the
/// counter as it leaves it), and the counter goes one down after each. Gives back the counter.
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

/// The "quit?" question's best whole frame: the first block's clock (+8) / 100 less 7, and the
/// remainder.
pub fn best_frame(clock: u32) -> (u32, u32) {
    ((clock / 100).wrapping_sub(7), clock % 100)
}

/// Whether the best frame is kept over `kept`: not below it, by the sign of the difference.
pub fn keeps_frame(frame: u32, kept: u32) -> bool {
    (frame.wrapping_sub(kept) as i32) >= 0
}

/// The question's notice: 0xC2 with the third answer offered, else 0xC1.
pub fn quit_notice(third: bool) -> u32 {
    if third { 0xC2 } else { 0xC1 }
}

/// Where the first two key letters are in the question's text: the last "(a/b" group's `a` and
/// `b` (their byte places); none when the text has no such group.
pub fn answer_letters(text: &[u8]) -> Option<(usize, usize)> {
    let open = text.iter().rposition(|&b| b == b'(')?;
    match text.get(open + 1..open + 4)? {
        [a, b'/', b] if !matches!(a, b'/' | b')') && !matches!(b, b'/' | b')') => {
            Some((open + 1, open + 3))
        }
        _ => None,
    }
}

/// The answer to a key from `input::yes_no_key`: 1 (yes) gives -1, -1 (no) gives 0, 2 gives -2
/// outside race mode 2 with the third answer offered; any other key, no answer yet.
pub fn quit_answer(key: u32, mode: u32, third: bool) -> Option<u32> {
    match key {
        1 => Some(u32::MAX),
        u32::MAX => Some(0),
        2 if mode != 2 && third => Some(0xFFFF_FFFE),
        _ => None,
    }
}

/// Whether a key down (scan code 1 .. 0x7F) ends the pause: any but Ctrl (0x1D), Alt (0x38),
/// Tab (0x0F), F1 .. F10 (0x3B ..= 0x44) and F11 (0x57).
pub fn ends_pause(key: usize) -> bool {
    !matches!(key, 0x1D | 0x38 | 0x0F | 0x3B..=0x44 | 0x57)
}

/// The pause's wait while the window is not active: held at 10 when it is 10 or less (unsigned),
/// else kept.
pub fn held_wait(wait: u32) -> u32 {
    if wait <= 10 { 10 } else { wait }
}

/// Whether the tyre-change screen shows: a change under way (the first block's +0x26C not 0 or
/// -1).
pub fn tyre_change_shown(change: u32) -> bool {
    change != 0 && change != u32::MAX
}

#[cfg(test)]
mod tests {
    #[test]
    fn the_key_letters_are_in_the_last_group() {
        use super::answer_letters;
        assert_eq!(answer_letters(b"RETIRAR? (s/n)"), Some((10, 12)));
        assert_eq!(answer_letters(b"QUIT? (y/n/r)"), Some((7, 9)));
        assert_eq!(answer_letters(b"Quit"), None);
        assert_eq!(answer_letters(b"Quit (yes)"), None);
        assert_eq!(answer_letters(b"A (b/"), None);
    }

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
