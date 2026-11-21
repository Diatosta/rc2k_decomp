//! A saved game's state laid out so that deflate packs it better, and back.
//!
//! The 180 driver records at the state's end are nine tenths of it. Written record after
//! record, a field's bytes are 0x94 apart, which deflate cannot see; written column after
//! column (every record's byte 0, then every record's byte 1, ...) the fields that hardly
//! change from driver to driver become runs. The first dword of a record counts up one
//! record to the next, so those four columns are written as the difference to the row
//! above (a count of ones). Nothing is dropped: the same bytes come back.

use crate::saves::savegame::{LEN, RECORD_LEN, RECORDS, RECORDS_AT};

/// How many of a record's first bytes are written as differences.
const DELTA_BYTES: usize = 4;

/// The state with its records laid out in columns. `state` is `LEN` bytes.
pub fn pack(state: &[u8]) -> Vec<u8> {
    assert_eq!(state.len(), LEN, "a saved game state");
    let mut out = state[..RECORDS_AT].to_vec();
    for column in 0..RECORD_LEN {
        let mut above = 0u8;
        for row in 0..RECORDS {
            let byte = state[RECORDS_AT + row * RECORD_LEN + column];
            if column < DELTA_BYTES {
                out.push(byte.wrapping_sub(above));
                above = byte;
            } else {
                out.push(byte);
            }
        }
    }
    out
}

/// The state `pack` was made from. `packed` is `LEN` bytes.
pub fn unpack(packed: &[u8]) -> Vec<u8> {
    assert_eq!(packed.len(), LEN, "a packed saved game state");
    let mut out = packed.to_vec();
    for column in 0..RECORD_LEN {
        let mut above = 0u8;
        for row in 0..RECORDS {
            let stored = packed[RECORDS_AT + column * RECORDS + row];
            let byte = if column < DELTA_BYTES {
                stored.wrapping_add(above)
            } else {
                stored
            };
            above = byte;
            out[RECORDS_AT + row * RECORD_LEN + column] = byte;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A state with counting first dwords and every other byte different.
    fn state() -> Vec<u8> {
        let mut s: Vec<u8> = (0..LEN).map(|n| (n * 7 + n / 13) as u8).collect();
        for row in 0..RECORDS {
            let at = RECORDS_AT + row * RECORD_LEN;
            s[at..at + 4].copy_from_slice(&(row as u32 * 300).to_le_bytes());
        }
        s
    }

    #[test]
    fn the_same_bytes_come_back() {
        let s = state();
        let packed = pack(&s);
        assert_eq!(packed.len(), LEN);
        assert_ne!(packed, s);
        assert_eq!(unpack(&packed), s);
        let zeros = vec![0; LEN];
        assert_eq!(unpack(&pack(&zeros)), zeros);
        let ones = vec![0xFF; LEN];
        assert_eq!(unpack(&pack(&ones)), ones);
    }

    #[test]
    fn the_head_and_players_stay_where_they_are() {
        let s = state();
        assert_eq!(pack(&s)[..RECORDS_AT], s[..RECORDS_AT]);
    }
}
