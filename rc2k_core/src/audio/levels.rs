//! The audio thread's levels: which entry of the volume table a volume, a pan or the master volume
//! takes, a buffer's frequency, the kept master volume, the samples' work buffer's size and the EAX
//! rooms' entries.
//!
//! The volume table has 0x41 entries, DirectSound's hundredths of a decibel from silence (index 0)
//! to full (0x40); the table itself is the caller's, asked for only at the index it reads.

use crate::Invalid;

/// The volume table's last index.
pub const FULL: u32 = 0x40;

/// A voice's volume (`SetVolume`): the entry at `volume * volume_factor >> 6` (signed).
/// Refused: an index outside the table.
pub fn buffer_volume(
    volume: u32,
    volume_factor: u32,
    table: impl Fn(u32) -> u32,
) -> Result<u32, Invalid> {
    let level = (volume as i32).wrapping_mul(volume_factor as i32) >> 6;
    if !(0..=FULL as i32).contains(&level) {
        return Err(Invalid("a volume past the volume table"));
    }
    Ok(table(level as u32))
}

/// A voice's pan (`SetPan`): below 0 the entry at `pan + 0x40`, else the entry at
/// `0x40 - pan` negated. Refused: a pan outside -0x40 .. 0x40.
pub fn pan(pan: i32, table: impl Fn(u32) -> u32) -> Result<u32, Invalid> {
    if !(-(FULL as i32)..=FULL as i32).contains(&pan) {
        return Err(Invalid("a pan past the volume table"));
    }
    Ok(if pan < 0 {
        table((pan + FULL as i32) as u32)
    } else {
        table((FULL as i32 - pan) as u32).wrapping_neg()
    })
}

/// A voice's buffer's frequency: its own shifted right by the data's halvings (the
/// format's low four bits).
pub fn buffer_frequency(frequency: u32, format: u32) -> u32 {
    frequency >> (format & 0xF)
}

/// The primary buffer's volume: the entry at the master volume, held to 0x40 (unsigned).
pub fn master_volume(volume: u32, table: impl Fn(u32) -> u32) -> u32 {
    table(volume.min(FULL))
}

/// A stream player's volume: the entry at its volume. Refused: past the table.
pub fn stream_volume(volume: u32, table: impl Fn(u32) -> u32) -> Result<u32, Invalid> {
    if volume > FULL {
        return Err(Invalid("a stream volume past the volume table"));
    }
    Ok(table(volume))
}

/// Command 8: the master volume to keep and set, `None` for a negative one (nothing).
pub fn kept_master_volume(volume: u32) -> Option<u32> {
    ((volume as i32) >= 0).then_some(volume)
}

/// The samples' work buffer in bytes: the setting's KB times 0x400 (wrapping), 1 KB when that
/// comes to 0.
pub fn sample_buffer_size(kb: u32) -> u32 {
    match kb.wrapping_mul(0x400) {
        0 => 0x400,
        size => size,
    }
}

/// The EAX rooms: ten reverbs of 0x10 bytes.
pub(crate) const EAX_ROOMS: u32 = 10;
pub(crate) const EAX_ROOM_LEN: u32 = 0x10;

/// Where room `room`'s reverb is in the rooms' table. Refused: past the ten.
pub fn eax_room_offset(room: u32) -> Result<u32, Invalid> {
    if room >= EAX_ROOMS {
        return Err(Invalid("an EAX room past the table"));
    }
    Ok(room * EAX_ROOM_LEN)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn table(n: u32) -> u32 {
        // -10000 .. 0 in steps, like the game's table
        (n as i32 * 156 - 10000).min(0) as u32
    }

    #[test]
    fn volumes() {
        assert_eq!(buffer_volume(0x40, 0x40, table), Ok(table(0x40)));
        assert_eq!(buffer_volume(0x20, 0x40, table), Ok(table(0x20)));
        assert_eq!(buffer_volume(0x20, 0x20, table), Ok(table(0x10)));
        assert!(buffer_volume(0x41, 0x40, table).is_err());
        assert!(buffer_volume(u32::MAX, 0x40, table).is_err());
        assert_eq!(master_volume(0x30, table), table(0x30));
        assert_eq!(master_volume(0xFFFF_FFFF, table), table(0x40));
        assert_eq!(stream_volume(0x40, table), Ok(table(0x40)));
        assert!(stream_volume(0x41, table).is_err());
    }

    #[test]
    fn pans() {
        assert_eq!(pan(-0x40, table), Ok(table(0)));
        assert_eq!(pan(-1, table), Ok(table(0x3F)));
        assert_eq!(pan(0, table), Ok(table(0x40).wrapping_neg()));
        assert_eq!(pan(0x40, table), Ok(table(0).wrapping_neg()));
        assert!(pan(0x41, table).is_err());
        assert!(pan(-0x41, table).is_err());
    }

    #[test]
    fn small_rules() {
        assert_eq!(buffer_frequency(44100, 0x121), 22050);
        assert_eq!(buffer_frequency(44100, 0x110), 44100);
        assert_eq!(kept_master_volume(0x20), Some(0x20));
        assert_eq!(kept_master_volume(0x8000_0000), None);
        assert_eq!(sample_buffer_size(0), 0x400);
        assert_eq!(sample_buffer_size(0x40_0000), 0x400);
        assert_eq!(sample_buffer_size(64), 0x1_0000);
        assert_eq!(eax_room_offset(9), Ok(0x90));
        assert!(eax_room_offset(10).is_err());
    }
}
