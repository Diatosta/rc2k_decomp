//! The race's steps over the cars (from `drive::race_cars_step`
//! 0x4608BB, `drive::replay_step` 0x460B2F, `drive::play_replay_frame` 0x460049,
//! `drive::read_packed` 0x41EEB5 and `drive::race_init` 0x4462B9): every car stepped in
//! turn (`g_currentCar` from 0 to `g_raceCars`), the skid marks begun again, the
//! replay's frame read at each whole frame, the replay file's packed reads, and the
//! race's start (its best line, its fifty settling steps).

use crate::Invalid;

/// The cars' room (`g_carBlocks`, eight).
pub const CAR_ROOM: u32 = 8;

/// The race's cars' count (`g_raceCars`): 1 ..= 8 (0 runs the original's loop round the
/// whole of `g_currentCar`; past 8 its blocks' table).
pub fn check_cars(count: u32) -> Result<(), Invalid> {
    if !(1..=CAR_ROOM).contains(&count) {
        return Err(Invalid("a cars' count of 0 or past 8"));
    }
    Ok(())
}

/// A car's block slot (`g_carBlocks[n & 7]`).
pub fn block_slot(car: u32) -> usize {
    (car & 7) as usize
}

/// The cars' loop: `g_currentCar` 0 (`set_current`), then `step()` (a car's step: it
/// reads the current car itself), the current (`current()`, read back) one up, until it
/// is the count (`count()`, read after each). The last current car.
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

/// `drive::replay_step`: whether the skid marks are begun again: unless the first
/// block's clock (+8) is one past the kept one (+0xF0C), by the difference.
pub fn skids_restart(clock: u32, kept: u32) -> bool {
    clock.wrapping_sub(kept) != 1
}

/// `drive::play_replay_frame`: None while not playing (`Numbers.replay_mode` not 2); else
/// whether the tenth is a whole frame's (0: the frame read and loaded).
pub fn play_frame(mode: u32, tenth: impl FnOnce() -> u32) -> Option<bool> {
    (mode == crate::drive::replay_frames::PLAYING).then(|| tenth() == 0)
}

/// `drive::read_packed`: the count's meaning: 0 or more a plain read of that many
/// bytes (None); below 0 the packed bytes to read, -count.
pub fn packed_bytes(count: u32) -> Option<u32> {
    ((count as i32) < 0).then(|| count.wrapping_neg())
}

/// `drive::race_init`'s fatal error with fewer than two checkpoints.
pub const NO_BEST_LINE: u32 = 0x8060_0D1E;

/// Whether the best line failed: fewer than two checkpoints (`g_checkpointCount`), by the
/// sign of the `cmp`.
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
