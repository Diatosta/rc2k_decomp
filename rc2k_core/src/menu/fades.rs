//! The inspection's fades: the see-through of the fade's quad by the time gone (converted
//! 2026-10-21 from `menu::fade_in` 0x4A1CFA and `menu::fade_out` 0x4A1DA5). The length
//! (`dword_75AB44`, ms) is cut in a hundred steps; the see-through goes from 0 up to the
//! full one (`dword_75AA94`) in, or from it down out, a hundredth of it a step.

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

/// A step: the see-through written (none on the way in while a hundredth is 0), and what
/// the division leaves for the frame the original hands it to (eax, ebx, edx, ebp
/// there).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Step {
    pub see: Option<u32>,
    pub eax: u32,
    pub ebx: u32,
    pub edx: u32,
    pub ebp: u32,
}

/// The step at `gone` ms: none on the way in with no time gone (`full` asked only past
/// the checks).
pub fn step(
    gone: u32,
    length: u32,
    full: impl FnOnce() -> u32,
    out: bool,
) -> Result<Option<Step>, Stop> {
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
    let share = gone / per_step;
    let full = full();
    let mut s = Step {
        see: None,
        eax: share,
        ebx: per_step,
        edx: gone % per_step,
        ebp: per_step,
    };
    if out {
        s.eax = share.wrapping_mul(full);
        s.ebx = full;
        if s.eax == 0 {
            s.ebx = 1;
        } else {
            s.edx = s.eax % STEPS;
            s.ebp = STEPS;
            s.eax /= STEPS;
            s.ebx = full.wrapping_sub(s.eax);
        }
        s.see = Some(s.ebx);
    } else {
        s.ebx = full.wrapping_mul(share);
        if s.ebx != 0 {
            s.eax = s.ebx;
            s.edx = s.eax % STEPS;
            s.ebp = STEPS;
            s.eax /= STEPS;
            s.see = Some(s.eax);
        }
    }
    Ok(Some(s))
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
        let s = step(370, 1000, || 0xFF, false).unwrap().unwrap();
        assert_eq!(s.see, Some(37 * 0xFF / 100));
        // in, under a step: nothing written.
        assert_eq!(step(5, 1000, || 0xFF, false).unwrap().unwrap().see, None);
        // out: the full one less the share; a share of 0 makes it 1.
        assert_eq!(
            step(370, 1000, || 0xFF, true).unwrap().unwrap().see,
            Some(0xFF - 37 * 0xFF / 100)
        );
        assert_eq!(step(5, 1000, || 0xFF, true).unwrap().unwrap().see, Some(1));
        assert!(over(1000, 1000) && !over(999, 1000) && !over(0x8000_0000, 1000));
    }
}
