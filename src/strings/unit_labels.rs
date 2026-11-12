//! The unit labels' string ids, kept with the image's layout
//! (`notes/conversion.md`): IDA 0x51EB84 .. 0x51EBB0, tables of two words, one for each
//! units setting (0 metric, 1 imperial): `g_temperatureLabelIds`, `g_speedLabelIds`,
//! the tables of the first labels and six more (the figures' labels).
//! The converted `strings` functions have the ids as constants (`rc2k_core::strings`);
//! nothing reads these.

/// The label ids by table and units setting (read by nothing: kept for the test against the
/// image).
#[cfg(test)]
pub static LABEL_IDS: [u16; 22] = [
    1398, 1397, 1395, 1396, 1367, 1368, 1372, 1373, 1375, 1374, 581, 582, 583, 584, 621, 620, 613,
    610, 611, 617, 616, 615,
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn values_are_the_games() {
        let ours: Vec<u8> = LABEL_IDS.iter().flat_map(|id| id.to_le_bytes()).collect();
        crate::test_support::first_values::assert_matches(
            "strings.unit_labels.LABEL_IDS/44",
            &ours,
        );
    }
}
