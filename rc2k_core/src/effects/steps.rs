//! The squares' and windscreen drops' clock and the shapes they take: the steps since the shapes
//! were last looked at counted on a clock of 256 (`WindscreenDropState.clock` the clock,
//! `MaskPicks.index` the latest step), and each step's shapes picked from a table of 256 entries.

use crate::Invalid;

/// The entries of the shapes' table.
pub(crate) const SHAPE_ENTRIES: u32 = 0x100;

/// The steps from the clock to the latest step, on a clock of 256.
pub fn steps(latest: u32, clock: u32) -> u32 {
    latest.wrapping_sub(clock) & 0xFF
}

/// The latest step moved on one (`windscreen_flakes_step`'s loop), on a clock of 256.
pub fn next_step(latest: u32) -> u32 {
    latest.wrapping_add(1) & 0xFF
}

/// A step's shapes (`pick_shapes`): the entry made positive (wrapping, so 0x80000000 stays as
/// it is) and its flag: 2, or 0 for a negative entry.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Shapes {
    pub shapes: u32,
    pub flag: u32,
}

/// The table's entry for a step; refuses a step past the table (its writer masks it).
pub fn shape_entry(step: u32) -> Result<usize, Invalid> {
    if step >= SHAPE_ENTRIES {
        return Err(Invalid("a shape index past the 256 entries"));
    }
    Ok(step as usize)
}

pub fn pick_shapes(entry: i32) -> Shapes {
    if entry < 0 {
        Shapes {
            shapes: entry.wrapping_neg() as u32,
            flag: 0,
        }
    } else {
        Shapes {
            shapes: entry as u32,
            flag: 2,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clock() {
        assert_eq!(steps(5, 3), 2);
        assert_eq!(steps(3, 5), 0xFE);
        assert_eq!(next_step(0xFF), 0);
        assert!(shape_entry(0x100).is_err());
        assert_eq!(shape_entry(0xFF), Ok(0xFF));
        assert_eq!(pick_shapes(-5), Shapes { shapes: 5, flag: 0 });
        assert_eq!(pick_shapes(7), Shapes { shapes: 7, flag: 2 });
        assert_eq!(
            pick_shapes(i32::MIN),
            Shapes {
                shapes: 0x8000_0000,
                flag: 0
            }
        );
    }
}
