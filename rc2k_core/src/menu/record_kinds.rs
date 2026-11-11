//! The kind of the record after the current one (from
//! `menu::next_record_kind` 0x470D08): the event drivers look ahead with it.

use crate::Invalid;
use crate::files::records;
use crate::strings::bytes::Bytes;

/// How many records `records::has_section` looks through for the section.
pub const SECTION_LIMIT: usize = 600;

/// The kind (first dword) of record `index` + 1 of `section` in the record table; refused
/// for a section or a next index past 63, or a section the table does not hold.
pub fn next_record_kind(
    table: &(impl Bytes + ?Sized),
    section: u32,
    index: u32,
) -> Result<u32, Invalid> {
    let next = index.wrapping_add(1);
    if section >= 64 || next >= 64 || !records::has_section(table, section, SECTION_LIMIT) {
        return Err(Invalid("a record past the table"));
    }
    let place = records::find_record(table, next as i32, section)?;
    table
        .u32_le(place as usize)
        .ok_or(Invalid("a record past the table"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kinds() {
        let mut table = vec![0u8; records::RECORD_LEN * 8];
        for (n, tag) in [0u32, 1, 4, 5, 0, 3, 0xFFFF_FFFF].iter().enumerate() {
            table[n * records::RECORD_LEN..][..4].copy_from_slice(&tag.to_le_bytes());
        }
        assert_eq!(next_record_kind(&table[..], 0, 0), Ok(1));
        assert_eq!(next_record_kind(&table[..], 1, 0), Ok(0));
        assert_eq!(next_record_kind(&table[..], 1, 1), Ok(3));
        assert_eq!(next_record_kind(&table[..], 0, u32::MAX), Ok(0));
        assert!(next_record_kind(&table[..], 0, 63).is_err());
        assert!(next_record_kind(&table[..], 64, 0).is_err());
    }
}
