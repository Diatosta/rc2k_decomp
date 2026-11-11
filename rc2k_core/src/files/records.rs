//! The game's record tables: rows of 0xA4 bytes (stages, sections, best times), each
//! starting with its tag, ended by a tag of -1 (from
//! `records::find_record` 0x47255D, `find_stage_record` 0x468289,
//! `default_stage_records` 0x47C7E0, `seal_time` 0x40264D, `trakpak_entry` 0x473027).
//! Places are byte offsets from the table's start.

use crate::Invalid;
use crate::strings::bytes::Bytes;

/// A record's size.
pub const RECORD_LEN: usize = 0xA4;
/// The tag of a stage's record.
pub const STAGE: u32 = 0;
/// The tag of a section's first record (a rally, a class).
pub const SECTION: u32 = 4;
/// The tag after the last record.
pub const END: u32 = u32::MAX;

/// How far `find_stage_record` goes before it is refused (the original has no end test).
const MOST_STEPS: usize = 0x1000;

/// Record `n`'s tag.
pub fn tag(table: &(impl Bytes + ?Sized), n: usize) -> Result<u32, Invalid> {
    table
        .u32_le(n * RECORD_LEN)
        .ok_or(Invalid("a record past the record table"))
}

/// The record after the `section`-th section record (from the start for 0), as a record
/// number; `None` if there is no such section within `limit` records.
fn section_start(
    table: &(impl Bytes + ?Sized),
    section: u32,
    limit: usize,
) -> Result<usize, Invalid> {
    if section == 0 {
        return Ok(0);
    }
    let mut seen = 0u32;
    for n in 0..limit {
        if tag(table, n)? == SECTION {
            seen += 1;
            if seen == section {
                return Ok(n + 1);
            }
        }
    }
    Err(Invalid("no such section in the record table"))
}

/// The place of record `index` of section `section` (`find_record`): the index is not
/// checked, the place may be anywhere. The section is looked for as far as the table goes
/// (the original has no end test; its callers check `has_section` first).
pub fn find_record(
    table: &(impl Bytes + ?Sized),
    index: i32,
    section: u32,
) -> Result<isize, Invalid> {
    let start = section_start(table, section, usize::MAX)?;
    Ok((start as isize + index as isize) * RECORD_LEN as isize)
}

/// Whether section `section` begins within the first `limit` records.
pub fn has_section(table: &(impl Bytes + ?Sized), section: u32, limit: usize) -> bool {
    section_start(table, section, limit).is_ok()
}

/// The place of the `number`-th stage (from 1) of section `section` (`find_stage_record`).
/// The original counts the section record itself when looking for the stage (its tag is
/// not a stage's, so that is the same).
pub fn find_stage_record(
    table: &(impl Bytes + ?Sized),
    section: u32,
    number: u32,
) -> Result<usize, Invalid> {
    if number == 0 {
        return Err(Invalid("stage number 0: the original runs on"));
    }
    // The original stops at the section record itself, not after it.
    let from =
        section_start(table, section, MOST_STEPS + 1)?.saturating_sub((section != 0) as usize);
    let mut seen = 0u32;
    for n in from..=MOST_STEPS {
        if tag(table, n)? == STAGE {
            seen += 1;
            if seen == number {
                return Ok(n * RECORD_LEN);
            }
        }
    }
    Err(Invalid("no such stage record within 0x1000 records"))
}

/// The number of records before the end tag; the first record is not looked at for it
/// (`default_stage_records`).
pub fn record_count(table: &(impl Bytes + ?Sized), most: usize) -> Result<usize, Invalid> {
    for n in 1..=most {
        if tag(table, n)? == END {
            return Ok(n);
        }
    }
    Err(Invalid("a record table without its end"))
}

/// The longest time a record seals (hundredths of a second).
pub const LONGEST_TIME: u32 = 0x22550FF;

/// A time as sealed for the records (`seal_time`): at most `LONGEST_TIME`; negated when a
/// cheat of 0x16000 is on, or negated with bit 30 cleared with cheat 0x40000 or the patch
/// switch.
pub fn sealed_value(time: u32, cheats: u32, patch_switch: bool) -> u32 {
    let time = time.min(LONGEST_TIME);
    if cheats & 0x16000 != 0 {
        time.wrapping_neg()
    } else if cheats & 0x40000 != 0 || patch_switch {
        time.wrapping_neg() & 0xBFFF_FFFF
    } else {
        time
    }
}

/// The eight bytes sealed: the value, three bytes (the car, the stage, ...), then 0x5A
/// plus the seven before each inverted.
pub fn seal_block(value: u32, bytes: [u8; 3]) -> [u8; 8] {
    let mut b = [0u8; 8];
    b[..4].copy_from_slice(&value.to_le_bytes());
    b[4..7].copy_from_slice(&bytes);
    b[7] = b[..7].iter().fold(0x5Au8, |s, &x| s.wrapping_add(!x));
    b
}

/// Where a track pack's header has its entries, and their size.
pub const PACK_ENTRIES: usize = 0x34;
pub const PACK_ENTRY_LEN: usize = 0x40;

/// The place in the loaded pack's header of the entry a record names (`trakpak_entry`);
/// `None` when the record belongs to another pack (the original's "Invalid TrakPak
/// Header", or a loop that never ends when the header's first dword is not 0: fixed).
pub fn pack_entry(index: u32, pack: u32, loaded_pack: u32) -> Option<usize> {
    (pack == loaded_pack).then(|| PACK_ENTRIES + index as usize * PACK_ENTRY_LEN)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn table(tags: &[u32]) -> Vec<u8> {
        let mut t = vec![0u8; RECORD_LEN * tags.len()];
        for (n, &tag) in tags.iter().enumerate() {
            t[n * RECORD_LEN..n * RECORD_LEN + 4].copy_from_slice(&tag.to_le_bytes());
        }
        t
    }

    #[test]
    fn finds() {
        let t = table(&[1, 0, 0, 3, 4, 1, 0, 0, 0, 4, 0, 4, 0, 0, END]);
        assert_eq!(find_record(&t, 2, 0).unwrap(), 2 * RECORD_LEN as isize);
        assert_eq!(find_record(&t, 0, 1).unwrap(), 5 * RECORD_LEN as isize);
        assert_eq!(find_record(&t, -1, 2).unwrap(), 9 * RECORD_LEN as isize);
        assert!(find_record(&t, 0, 4).is_err());
        assert!(has_section(&t, 3, 15));
        assert!(!has_section(&t, 3, 11));
        assert_eq!(find_stage_record(&t, 0, 1).unwrap(), RECORD_LEN);
        assert_eq!(find_stage_record(&t, 1, 2).unwrap(), 7 * RECORD_LEN);
        assert_eq!(find_stage_record(&t, 3, 2).unwrap(), 13 * RECORD_LEN);
        assert!(find_stage_record(&t, 3, 3).is_err());
        assert!(find_stage_record(&t, 0, 0).is_err());
        assert_eq!(record_count(&t, 100).unwrap(), 14);
        assert!(record_count(&table(&[END, 0]), 100).is_err());
    }

    #[test]
    fn seals_and_packs() {
        assert_eq!(sealed_value(500, 0, false), 500);
        assert_eq!(sealed_value(u32::MAX, 0, false), LONGEST_TIME);
        assert_eq!(sealed_value(500, 0x2000, false), 500u32.wrapping_neg());
        assert_eq!(
            sealed_value(500, 0, true),
            500u32.wrapping_neg() & 0xBFFF_FFFF
        );
        let b = seal_block(0x0403_0201, [5, 6, 7]);
        assert_eq!(&b[..7], &[1, 2, 3, 4, 5, 6, 7]);
        assert_eq!(
            b[7],
            (0x5Au8).wrapping_add((0..7).map(|i| !b[i]).fold(0u8, u8::wrapping_add))
        );
        assert_eq!(pack_entry(2, 9, 9), Some(0x34 + 0x80));
        assert_eq!(pack_entry(2, 9, 8), None);
    }
}
