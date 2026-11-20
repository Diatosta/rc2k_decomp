//! The race's steps over the cars: every car stepped in turn, the skid marks begun again, the
//! replay's frame read at each whole frame, the replay file's packed reads, and the race's start
//! (its best line, its fifty settling steps).

use crate::Invalid;

/// The most cars a race has (the car blocks' table).
pub(crate) const CAR_ROOM: u32 = 8;

/// Refused: a cars' count outside 1 ..= 8 (0 would loop through every value of the current car;
/// past 8 is past its blocks' table).
pub fn check_cars(count: u32) -> Result<(), Invalid> {
    if !(1..=CAR_ROOM).contains(&count) {
        return Err(Invalid("a cars' count of 0 or past 8"));
    }
    Ok(())
}

/// A car's slot in the car blocks' table.
pub fn block_slot(car: u32) -> usize {
    (car & 7) as usize
}

/// Steps every car: current car 0 (`set_current`), then `step()` (it reads the current car
/// itself) and the current (`current()`, read back) one up, until it is the count (`count()`,
/// read after each). Gives back the last current car.
pub fn each_car<E>(
    current: impl Fn() -> u32,
    count: impl Fn() -> u32,
    mut set_current: impl FnMut(u32),
    mut step: impl FnMut() -> Result<(), E>,
) -> Result<u32, E> {
    set_current(0);
    loop {
        step()?;
        let n = current().wrapping_add(1);
        set_current(n);
        if n == count() {
            return Ok(n);
        }
    }
}

/// Whether the skid marks are begun again: unless the first block's clock (+8) is one past the
/// kept one (+0xF0C).
pub fn skids_restart(clock: u32, kept: u32) -> bool {
    clock.wrapping_sub(kept) != 1
}

/// None while not playing a replay (`Numbers.replay_mode` not 2); else whether the tenth starts
/// a whole frame (0: the frame is read and loaded).
pub fn play_frame(mode: u32, tenth: impl FnOnce() -> u32) -> Option<bool> {
    (mode == crate::drive::replay_frames::PLAYING).then(|| tenth() == 0)
}

/// A replay read's count: 0 or more is a plain read of that many bytes (None); below 0, -count
/// packed bytes to read.
pub fn packed_bytes(count: u32) -> Option<u32> {
    ((count as i32) < 0).then(|| count.wrapping_neg())
}

/// The race start's fatal error with fewer than two checkpoints.
pub const NO_BEST_LINE: u32 = 0x8060_0D1E;

/// Whether the best line failed: fewer than two checkpoints (signed).
pub fn no_best_line(checkpoints: u32) -> bool {
    (checkpoints.wrapping_sub(2) as i32) < 0
}

/// The race's settling steps before the start.
pub const SETTLING_STEPS: u32 = 0x32;

/// The log's indent round each part.
pub const LOG_INDENT: u32 = 3;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cars() {
        assert!(check_cars(0).is_err() && check_cars(9).is_err() && check_cars(8).is_ok());
        assert_eq!(block_slot(9), 1);
        let current = std::cell::Cell::new(7);
        let mut steps = 0;
        let n = each_car::<()>(
            || current.get(),
            || 3,
            |n| current.set(n),
            || {
                steps += 1;
                assert!(steps <= 3);
                Ok(())
            },
        );
        assert_eq!((n, steps), (Ok(3), 3));
    }

    #[test]
    fn rules() {
        assert!(!skids_restart(11, 10));
        assert!(skids_restart(10, 10));
        assert_eq!(play_frame(1, || panic!()), None);
        assert_eq!(play_frame(2, || 0), Some(true));
        assert_eq!(play_frame(2, || 5), Some(false));
        assert_eq!(packed_bytes(5), None);
        assert_eq!(packed_bytes((-5i32) as u32), Some(5));
        assert!(no_best_line(1) && no_best_line(0) && !no_best_line(2));
        assert!(!no_best_line(0x8000_0001));
    }
}
