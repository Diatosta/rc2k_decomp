//! The unit labels' string ids, out of RAL.EXE's image (phase 5; typed in phase 7,
//! `notes/conversion.md`): IDA 0x51EB84 .. 0x51EBB0, tables of two words, one for each
//! units setting (0 metric, 1 imperial): `g_temperatureLabelIds`, `g_speedLabelIds`,
//! `word_51EB8C`, `word_51EB90`, `word_51EB94` and six more tables (the figures' labels).
//! The converted `strings` functions have the ids as constants (`rc2k_core::strings`);
//! nothing reads these.

use crate::session::Word;

/// IDA's address of the tables.
pub const START: usize = 0x51EB84;
/// Their length.
pub const LEN: usize = 0x2C;

const fn words<const N: usize>(values: [u16; N]) -> [Word; N] {
    let mut out = [const { Word::new(0) }; N];
    let mut i = 0;
    while i < N {
        out[i] = Word::new(values[i]);
        i += 1;
    }
    out
}

/// The label ids by table and units setting.
pub static LABEL_IDS: [Word; 22] = words([
    1398, 1397, 1395, 1396, 1367, 1368, 1372, 1373, 1375, 1374, 581, 582, 583, 584, 621, 620, 613,
    610, 611, 617, 616, 615,
]);

/// Where the tables are (`moved::MOVED`'s home for IDA 0x51EB84).
pub fn home() -> usize {
    crate::pointers::host_number(LABEL_IDS.as_ptr())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_values_are_the_games() {
        crate::gfx::first_values_are_the_games(&LABEL_IDS, START, START + LEN);
        assert_eq!(crate::moved::place(START), home());
    }
}
