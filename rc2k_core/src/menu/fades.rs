//! The inspection's fades: the see-through of the fade's quad by the time gone
//! (from `menu::fade_in` 0x4A1CFA and `menu::fade_out` 0x4A1DA5). The length
//! (`MenuState.menus.fade_length`, ms) is cut in a hundred steps; the see-through goes from 0 up to the
//! full one in, or from it down out, a hundredth of it a step.

/// The steps of a fade.
pub const STEPS: u32 = 100;

/// A length of 1 .. 99 ms makes a step of 0 ms, which the original divides by.
pub fn too_short(length: u32) -> bool {
    (1..STEPS).contains(&length)
}

/// The see-through a fade starts at.
pub fn first_see(out: bool, full: u32) -> u32 {
    if out { full } else { 0 }
}

/// Why a step cannot be made.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stop {
    /// The original's checks before its division (a length of 0, or no time gone on the
    /// way out): its fatal error 0x80701010.
    DivideByZero,
    /// A step of 0 ms (a length of 1 .. 99 ms): the original's `div` traps.
    ShortStep,
}

/// The step at `gone` ms: the see-through to write, or none (on the way in with no time
/// gone, or while a hundredth of the full one is still 0). `full` is asked only past the
/// checks.
pub fn step(
    gone: u32,
    length: u32,
    full: impl FnOnce() -> u32,
    out: bool,
) -> Result<Option<u32>, Stop> {
    if gone == 0 && !out {
        return Ok(None);
    }
    if length == 0 || gone == 0 {
        return Err(Stop::DivideByZero);
    }
    let per_step = length / STEPS;
    if per_step == 0 {
        return Err(Stop::ShortStep);
    }
    // How many hundredths of the length have gone.
    let share = gone / per_step;
    let full = full();
    let scaled = share.wrapping_mul(full);
    if out {
        // The full one less the share of it; a share of 0 makes it 1.
        Ok(Some(if scaled == 0 {
            1
        } else {
            full.wrapping_sub(scaled / STEPS)
        }))
    } else {
        Ok((scaled != 0).then_some(scaled / STEPS))
    }
}

/// The fade is over once the time gone reaches the length (signed).
pub fn over(gone: u32, length: u32) -> bool {
    gone as i32 >= length as i32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn steps() {
        assert!(too_short(1) && too_short(99) && !too_short(0) && !too_short(100));
        assert_eq!(first_see(true, 0xFF), 0xFF);
        assert_eq!(first_see(false, 0xFF), 0);
        assert_eq!(step(0, 1000, || 0xFF, false), Ok(None));
        assert_eq!(step(0, 1000, || 0xFF, true), Err(Stop::DivideByZero));
        assert_eq!(step(5, 0, || 0xFF, false), Err(Stop::DivideByZero));
        assert_eq!(step(5, 50, || 0xFF, false), Err(Stop::ShortStep));
        // in: 370 ms of 1000: 37 hundredths of 0xFF.
        assert_eq!(step(370, 1000, || 0xFF, false), Ok(Some(37 * 0xFF / 100)));
        // in, under a step: nothing written.
        assert_eq!(step(5, 1000, || 0xFF, false), Ok(None));
        // out: the full one less the share; a share of 0 makes it 1.
        assert_eq!(
            step(370, 1000, || 0xFF, true),
            Ok(Some(0xFF - 37 * 0xFF / 100))
        );
        assert_eq!(step(5, 1000, || 0xFF, true), Ok(Some(1)));
        assert!(over(1000, 1000) && !over(999, 1000) && !over(0x8000_0000, 1000));
    }
}
