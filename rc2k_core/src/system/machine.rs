//! What the game asks of the machine.

/// The keyboard type for a keyboard's subtype: 2 (Japanese) for 0xD01 ..= 0xD06.
pub fn keyboard_type(subtype: u32) -> Option<u32> {
    (0xD01..=0xD06).contains(&subtype).then_some(2)
}

/// `system::same_guid`: whether two GUIDs are the same, compared as four dwords (`a(k)`,
/// `b(k)` the k-th of each), the first difference ending it.
pub fn same_guid(a: impl Fn(u32) -> u32, b: impl Fn(u32) -> u32) -> bool {
    (0..4).all(|k| a(k) == b(k))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn machine() {
        assert_eq!((keyboard_type(0xD03), keyboard_type(4)), (Some(2), None));
    }

    #[test]
    fn guids() {
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
    }
}
