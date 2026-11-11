//! What blinks in the menus, and the idle switch (from
//! `menu::update_blink` 0x47E826): worked out from the menus' clock each frame.

/// Five minutes without a key, in ms.
pub const IDLE_AFTER_MS: i32 = 300_000;

/// (the highlight's blink, the text entry's cursor, idle) for a clock value: the highlight
/// is on while bit 8 of the time since the last key (`last_key`) is clear, 256 ms on and
/// 256 off, and stays off while `blink_off` is 1; the cursor is on while bit 9 of the time
/// since `cursor_time` is clear; idle once the time since the last key has reached five
/// minutes (signed).
pub fn blink_states(
    clock: u32,
    last_key: u32,
    cursor_time: u32,
    blink_off: bool,
) -> (bool, bool, bool) {
    let since_key = clock.wrapping_sub(last_key);
    let blink = !blink_off && since_key & 0x100 == 0;
    let cursor_on = clock.wrapping_sub(cursor_time) & 0x200 == 0;
    let idle = since_key as i32 >= IDLE_AFTER_MS;
    (blink, cursor_on, idle)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blink() {
        assert_eq!(
            blink_states(0x1000, 0x1000, 0x1000, false),
            (true, true, false)
        );
        assert_eq!(
            blink_states(0x1100, 0x1000, 0x1000, false),
            (false, true, false)
        );
        assert_eq!(
            blink_states(0x1000, 0x1000, 0x1000, true),
            (false, true, false)
        );
        assert_eq!(
            blink_states(0x1200, 0x1200, 0x1000, false),
            (true, false, false)
        );
        assert!(blink_states(300_000, 0, 0, false).2);
        assert!(!blink_states(299_999, 0, 0, false).2);
        assert!(!blink_states(0, 1, 0, false).2);
    }
}
