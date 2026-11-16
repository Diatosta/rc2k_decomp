//! The free-roaming car: no physics, the car moved along its heading by its pedals and turned by
//! its steering. The inputs, the move's turn, the ground and the matrix stay with the caller.

/// The move along the heading: the throttle less the brake (+0x638 - +0x63C) << 5, << 7
/// with the fast switch.
pub fn roam_distance(throttle: u32, brake: u32, fast: bool) -> u32 {
    let d = throttle.wrapping_sub(brake) << 5;
    if fast { d << 2 } else { d }
}

/// The heading's turn: the steering (+0x644) >> 4, by its sign.
pub fn roam_turn(steering: u32) -> u32 {
    ((steering as i32) >> 4) as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roaming() {
        assert_eq!(roam_distance(0x100, 0, false), 0x2000);
        assert_eq!(roam_distance(0x100, 0, true), 0x8000);
        assert_eq!(roam_distance(0, 1, false), (-32i32) as u32);
        assert_eq!(roam_turn((-32i32) as u32), (-2i32) as u32);
        assert_eq!(roam_turn(0x40), 4);
    }
}
