//! What the game asks of the machine (from
//! `system::check_developer_machine` 0x416292 and `system::read_keyboard_type` 0x410AA3;
//! `system::same_guid` 0x4016F9 and `system::read_volume_serial` 0x403131's results).

/// Whether the computer is called "andrew" (its first six bytes, as three words each with 0x2020
/// or'ed in: any case, and some other bytes, pass), and the last word compared. A developer's
/// machine is kept off the best tyre-change time.
pub fn developer(name: [u8; 6]) -> (bool, u16) {
    let mut last = 0;
    for (n, pair) in [*b"an", *b"dr", *b"ew"].into_iter().enumerate() {
        last = u16::from_le_bytes([name[2 * n], name[2 * n + 1]]) | 0x2020;
        if last != u16::from_le_bytes(pair) {
            return (false, last);
        }
    }
    (true, last)
}

/// `g_keyboardType` for a keyboard's subtype: 2 (Japanese) for 0xD01 ..= 0xD06.
pub fn keyboard_type(subtype: u32) -> Option<u32> {
    (0xD01..=0xD06).contains(&subtype).then_some(2)
}

/// `system::same_guid`: whether two GUIDs are the same, compared as four dwords (`a(k)`,
/// `b(k)` the k-th of each), the first difference ending it.
pub fn same_guid(a: impl Fn(u32) -> u32, b: impl Fn(u32) -> u32) -> bool {
    (0..4).all(|k| a(k) == b(k))
}

/// What `system::read_volume_serial` leaves from the 16 bytes `GetVolumeInformationA`
/// filled (+0 the serial, +4 the name in 12 bytes): the name's last byte (+0xF) made 0,
/// the dwords at +4, +8, +0xC and whether the serial is 0 (the zero flag).
pub fn volume_registers(info: &mut [u8; 16]) -> ([u32; 3], bool) {
    info[0xF] = 0;
    let d = |o: usize| u32::from_le_bytes([info[o], info[o + 1], info[o + 2], info[o + 3]]);
    ([d(4), d(8), d(0xC)], d(0) == 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn machine() {
        assert!(developer(*b"ANDREW").0 && developer(*b"andrew").0);
        assert_eq!(developer(*b"andy\0\0"), (false, u16::from_le_bytes(*b"dy")));
        assert_eq!((keyboard_type(0xD03), keyboard_type(4)), (Some(2), None));
    }

    #[test]
    fn guids_and_volume() {
        let a = [1, 2, 3, 4];
        assert!(same_guid(|k| a[k as usize], |k| a[k as usize]));
        let asked = std::cell::Cell::new(0);
        let b = [1, 9, 3, 4];
        assert!(!same_guid(
            |k| {
                asked.set(k);
                a[k as usize]
            },
            |k| b[k as usize]
        ));
        assert_eq!(asked.get(), 1);
        let mut info = [0u8; 16];
        info[4..16].copy_from_slice(b"RALLY_CDABCD");
        let (regs, zero) = volume_registers(&mut info);
        assert_eq!(
            regs,
            [
                u32::from_le_bytes(*b"RALL"),
                u32::from_le_bytes(*b"Y_CD"),
                0x0043_4241
            ]
        );
        assert!(zero);
    }
}
