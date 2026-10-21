//! The walk over a section's records from `index` (converted 2026-10-21 from
//! `menu::walk_section_records` 0x471F0D): each record's tag (its first dword) says what
//! is done for it, until a tag 3 or 4.

use crate::Invalid;

/// The records the walk may look at: `index` 0 .. 63.
pub const MOST_RECORDS: u32 = 64;

/// What a record's tag asks for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Step {
    /// Tag 0: the computer drivers' times (`menu::calc_cpu_times`), `dword_755944` 0.
    CpuTimes,
    /// Tag 1: the drivers' condition reset, `dword_755944` 1.
    Condition,
    /// Any other tag but 3 and 4: nothing of its own.
    Other,
    /// Tag 3: the walk ends with `dword_755928` 1.
    Ended,
    /// Tag 4: the walk ends.
    Stop,
}

impl Step {
    pub fn of(tag: u32) -> Step {
        match tag {
            0 => Step::CpuTimes,
            1 => Step::Condition,
            3 => Step::Ended,
            4 => Step::Stop,
            _ => Step::Other,
        }
    }

    /// The walk goes on past this record (`index`, `dword_755950` up by one, the
    /// players' totals reset).
    pub fn goes_on(self) -> bool {
        !matches!(self, Step::Ended | Step::Stop)
    }

    /// `dword_755944` after this record's step, if it is written.
    pub fn first(self) -> Option<u32> {
        match self {
            Step::CpuTimes => Some(0),
            Step::Condition => Some(1),
            _ => None,
        }
    }
}

/// The records the walk visits from `index` (`tag` reads a record's tag), with their
/// steps, the last a `Stop` or `Ended`; `check` is asked about each `CpuTimes` record when
/// it is met. Refused: a walk past `MOST_RECORDS`.
pub fn walk<E: From<Invalid>>(
    mut index: u32,
    mut tag: impl FnMut(u32) -> u32,
    mut check: impl FnMut(u32) -> Result<(), E>,
) -> Result<Vec<(u32, Step)>, E> {
    let mut out = Vec::new();
    loop {
        if index >= MOST_RECORDS {
            return Err(Invalid("a walk past 64 records").into());
        }
        let step = Step::of(tag(index));
        out.push((index, step));
        if !step.goes_on() {
            return Ok(out);
        }
        if step == Step::CpuTimes {
            check(index)?;
        }
        index += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn walks_to_the_end() {
        let tags = [7u32, 0, 1, 2, 3, 0];
        let mut checked = Vec::new();
        let w = walk::<Invalid>(
            1,
            |i| tags[i as usize],
            |i| {
                checked.push(i);
                Ok(())
            },
        )
        .unwrap();
        assert_eq!(
            w,
            vec![
                (1, Step::CpuTimes),
                (2, Step::Condition),
                (3, Step::Other),
                (4, Step::Ended)
            ]
        );
        assert_eq!(checked, vec![1]);
        assert_eq!(Step::CpuTimes.first(), Some(0));
        assert_eq!(Step::Other.first(), None);
        assert!(walk::<Invalid>(60, |_| 2, |_| Ok(())).is_err());
        assert_eq!(
            walk::<Invalid>(63, |_| 4, |_| Ok(())),
            Ok(vec![(63, Step::Stop)])
        );
        assert!(walk::<Invalid>(0, |_| 0, |_| Err(Invalid("no"))).is_err());
    }
}
