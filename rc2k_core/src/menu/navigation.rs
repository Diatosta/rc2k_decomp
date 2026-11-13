//! Moving the highlight about a menu: each item names its neighbours by number; a link of 0x6F is
//! none, 0xFFFFFC1A "ask the item" (its callback for that direction runs instead). The two vertical
//! moves go on along the same link until an item that can be chosen (its +0x48 0xFFFFFC1B); the two
//! horizontal ones take one step and may first turn a counter.

use crate::Invalid;

pub const NO_LINK: u32 = 0x6F;
pub const ASK_ITEM: u32 = 0xFFFF_FC1A;
pub const CAN_BE_CHOSEN: u32 = 0xFFFF_FC1B;
/// No menu has more items; a walk past it is refused.
pub const STEP_LIMIT: u32 = 256;

/// The counter left and right turn while it is on (`MenuState.carousel.count` its count,
/// `MenuState.carousel.front` the value, `MenuState.carousel.direction` the way,
/// `MenuState.carousel.angle` the step).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Turn {
    pub count: i32,
    pub value: i32,
    pub way: u32,
    pub step: u32,
}

/// The counter turned: right goes round up (only when the last way was right), left round
/// down (only when it was not); a count of 1 does not move it. The way and step: 1 and 0
/// (right) or -1 and -1 (left).
pub fn turn(t: Turn, right: bool) -> Turn {
    let mut t = t;
    if right {
        if t.count != 1 && t.way == 1 {
            let next = t.value.wrapping_add(1);
            t.value = if next >= t.count { 0 } else { next };
        }
        (t.way, t.step) = (1, 0);
    } else {
        if t.count != 1 && t.way != 1 {
            let next = t.value.wrapping_sub(1);
            t.value = if next < 0 {
                t.count.wrapping_sub(1)
            } else {
                next
            };
        }
        (t.way, t.step) = (u32::MAX, u32::MAX);
    }
    t
}

/// The longest time a turn moves by, in ms.
pub const LONGEST: u32 = 100;

/// The time since the last turn (`clock - last`), held to `LONGEST`.
pub fn passed(clock: u32, last: u32) -> u32 {
    clock.wrapping_sub(last).min(LONGEST)
}

/// How far the angle moves from `from` in `passed` ms. Refused: a time so
/// long that the division traps (above 0x71C71C6; `passed` keeps it from coming).
fn spin_step(from: u32, passed: u32) -> Result<u32, Invalid> {
    let moved = ((from >> 3) + 0x400_0000) as u64 * passed as u64 / 20;
    u32::try_from(moved).map_err(|_| Invalid("a time so long that the division traps"))
}

/// `menu::step_carousel`'s turn (`menu::turn_carousel`): while the way is not 0 the angle (`step`)
/// moves by `spin_step`, down for a negative way, up for a positive one. Past zero it is 0 and the
/// way 0; after a turn upwards the next object is in front (`value`), the first again after the
/// last (`count`).
pub fn spin(t: Turn, passed: u32) -> Result<Turn, Invalid> {
    let mut t = t;
    match t.way as i32 {
        0 => {}
        ..0 => {
            let (left, passed_zero) = t.step.overflowing_sub(spin_step(t.step, passed)?);
            (t.step, t.way) = if passed_zero { (0, 0) } else { (left, t.way) };
        }
        _ => {
            let (left, passed_zero) = t
                .step
                .overflowing_add(spin_step(t.step.wrapping_neg(), passed)?);
            if passed_zero {
                (t.step, t.way) = (0, 0);
                let front = t.value.wrapping_add(1);
                t.value = if front == t.count { 0 } else { front };
            } else {
                t.step = left;
            }
        }
    }
    Ok(t)
}

/// Where a link leads.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Link {
    None,
    Ask,
    To(u32),
}

pub fn link(v: u32) -> Link {
    match v {
        NO_LINK => Link::None,
        ASK_ITEM => Link::Ask,
        n => Link::To(n),
    }
}

/// A vertical move's end: from `next` on, along each item's link (`item` gives an item's
/// link and its +0x48) until one that can be chosen. Refused: what `item` refuses, no such
/// item within `STEP_LIMIT`.
pub fn walk(
    mut next: u32,
    item: impl Fn(u32) -> Result<(u32, u32), Invalid>,
) -> Result<u32, Invalid> {
    for _ in 0..STEP_LIMIT {
        let (link, chosen) = item(next)?;
        if chosen == CAN_BE_CHOSEN {
            return Ok(next);
        }
        next = link;
    }
    Err(Invalid("no item that can be chosen along this link"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spinning() {
        assert_eq!((passed(500, 300), passed(310, 300)), (100, 10));
        let t = Turn {
            count: 3,
            value: 2,
            way: 1,
            step: 0xFFFF_FF00,
        };
        // Up from just below zero: past it at once, the first object in front again.
        assert_eq!(
            spin(t, 100),
            Ok(Turn {
                count: 3,
                value: 0,
                way: 0,
                step: 0
            })
        );
        let t = Turn {
            way: u32::MAX,
            step: 0x8000_0000,
            ..t
        };
        let s = spin(t, 1).unwrap();
        assert_eq!((s.step, s.way, s.value), (0x7F00_0000, u32::MAX, 2));
        assert_eq!(spin(Turn { way: 0, ..t }, 100), Ok(Turn { way: 0, ..t }));
        assert!(spin_step(0, 0x71C_71C7 + 100).is_err());
    }

    #[test]
    fn navigation() {
        let t = Turn {
            count: 3,
            value: 2,
            way: 1,
            step: 5,
        };
        assert_eq!(
            turn(t, true),
            Turn {
                count: 3,
                value: 0,
                way: 1,
                step: 0
            }
        );
        assert_eq!(turn(t, false).value, 2);
        assert_eq!(
            turn(
                Turn {
                    way: 0,
                    value: 0,
                    ..t
                },
                false
            )
            .value,
            2
        );
        assert_eq!(
            (link(0x6F), link(ASK_ITEM), link(4)),
            (Link::None, Link::Ask, Link::To(4))
        );
        let items = |n: u32| Ok((n + 1, if n == 3 { CAN_BE_CHOSEN } else { 0 }));
        assert_eq!(walk(1, items), Ok(3));
        assert!(walk(4, items).is_err());
    }
}
