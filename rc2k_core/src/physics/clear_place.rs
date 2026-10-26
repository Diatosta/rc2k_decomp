//! The work car moved to a place where nothing touches it after a reset (converted
//! 2026-10-21 from `physics::find_clear_place` 0x43BDD1; its tries' contacts are the
//! hooks' own): tries n from 0x70 + (the car's clock & 0xF) up to 0xFF, each moving the
//! place by 2^((n >> 4) + 1) times a direction of sixteen (`dword_614014[n & 0xF]`,
//! `dword_614054[n & 0xF]`: x, z), until a try restores nothing; none: the car reset.

/// The tries' last + 1 (none clear: `physics::reset_car`).
pub const TRIES_END: u32 = 0x100;

/// The car's speed scale before the tries' figure (`dword_614C28`).
pub const TRY_MARK: u32 = 0x5A;

/// The first try: 0x70 + the car's clock (`g_car` +8) & 0xF.
pub fn first_try(clock: u32) -> u32 {
    (clock & 0xF) + 0x70
}

/// A try's move: the step 2^((n >> 4) + 1) (eax after), the direction's entry n & 0xF
/// (ebx), the shift (ecx), and each part's `imul` of the step by the table's entry (the
/// low dword added to the place; the high one the original's edx).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Move {
    pub step: u32,
    pub entry: u32,
    pub shift: u32,
}

pub fn try_move(n: u32) -> Move {
    let shift = (n >> 4) + 1;
    Move {
        step: 1u32 << (shift & 0x1F),
        entry: n & 0xF,
        shift,
    }
}

/// One part of a try's move: (the low dword added to the place, the high dword).
pub fn move_part(step: u32, direction: u32) -> (u32, u32) {
    let product = i64::from(step as i32) * i64::from(direction as i32);
    (product as u32, (product >> 32) as u32)
}

/// The tries from `first`: `attempt(n)` (the move and its contacts; true when nothing
/// was restored) until one is clear: Some(n), or None after 0xFF.
pub fn tries<E>(
    first: u32,
    mut attempt: impl FnMut(u32) -> Result<bool, E>,
) -> Result<Option<u32>, E> {
    let mut n = first;
    loop {
        if attempt(n)? {
            return Ok(Some(n));
        }
        n += 1;
        if n == TRIES_END {
            return Ok(None);
        }
    }
}

/// The height (`g_car` +0x138) after a clear try: with the body's deepest point below 0
/// (`dword_6D3068`, by the sign) lifted by it and 0x800 more.
pub fn lifted(height: u32, depth: u32) -> Option<u32> {
    ((depth as i32) < 0).then(|| height.wrapping_sub(depth).wrapping_add(0x800))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn moves() {
        assert_eq!(first_try(0x1234), 0x74);
        assert_eq!(
            try_move(0x75),
            Move {
                step: 0x100,
                entry: 5,
                shift: 8
            }
        );
        assert_eq!(try_move(0xFF).step, 0x10000);
        assert_eq!(
            move_part(0x100, (-3i32) as u32),
            ((-0x300i32) as u32, u32::MAX)
        );
        assert_eq!(move_part(0x10000, 0x10000), (0, 1));
        assert_eq!(tries::<()>(0x7E, |n| Ok(n == 0x80)), Ok(Some(0x80)));
        let mut count = 0;
        assert_eq!(
            tries::<()>(0x70, |_| {
                count += 1;
                Ok(false)
            }),
            Ok(None)
        );
        assert_eq!(count, 0x90);
        assert_eq!(lifted(0x1000, (-0x10i32) as u32), Some(0x1810));
        assert_eq!(lifted(0x1000, 0), None);
    }
}
