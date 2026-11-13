//! A saved game's state and where the event goes on from.
//!
//! The state, in order: seven dwords (two of the event's, its stage index and section, three
//! more), the four players' cars (0x28C bytes each, the name at +0x124; car 0's +0x14 and +0x274
//! hold two of the game's dwords) and 180 stage records
//! of 0x94 bytes.

/// The seven dwords at the start.
pub const HEAD_DWORDS: usize = 7;
pub const CARS: usize = 4;
pub const CAR_LEN: usize = 0x28C;
pub const RECORDS: usize = 0xB4;
pub const RECORD_LEN: usize = 0x94;
pub const RECORDS_LEN: usize = RECORDS * RECORD_LEN;

/// Where each part starts in the state.
pub const fn car_at(n: usize) -> usize {
    HEAD_DWORDS * 4 + n * CAR_LEN
}
pub const RECORDS_AT: usize = car_at(CARS);
/// The state's length.
pub const LEN: usize = RECORDS_AT + RECORDS_LEN;

/// The slot of item `item` (from 1) on page `page` (wrapping).
pub fn slot(page: u32, item: u32) -> u32 {
    page.wrapping_mul(10).wrapping_add(item).wrapping_sub(1)
}

/// Where a loaded event goes on, by what the event's state says.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Resume {
    /// 4 or 5: the next section (its first stage).
    NextSection,
    /// 0 or 1: the next stage.
    NextStage,
    /// 3: nothing more happens.
    Over,
    /// Anything else: fatal error 0x80702821 "Corrupt SaveGame".
    Corrupt,
}

/// The fatal error of a state that says nothing known.
pub const CORRUPT: u32 = 0x8070_2821;

pub fn resume(state: u32) -> Resume {
    match state {
        4 | 5 => Resume::NextSection,
        0 | 1 => Resume::NextStage,
        3 => Resume::Over,
        _ => Resume::Corrupt,
    }
}

/// The event's mode whose loop is the A8 championship's.
pub const A8_MODE: u32 = 0x13;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layout_and_resume() {
        assert_eq!(car_at(0), 0x1C);
        assert_eq!(RECORDS_AT, 0x1C + 4 * 0x28C);
        assert_eq!(LEN, 0x1C + 4 * 0x28C + 0xB4 * 0x94);
        assert_eq!(slot(2, 3), 22);
        assert_eq!(slot(0, 0), u32::MAX);
        assert_eq!(resume(5), Resume::NextSection);
        assert_eq!(resume(1), Resume::NextStage);
        assert_eq!(resume(3), Resume::Over);
        assert_eq!(resume(2), Resume::Corrupt);
    }
}
