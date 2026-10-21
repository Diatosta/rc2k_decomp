//! Moving the highlight about a menu (converted 2026-10-21 from the four
//! `menu::move_highlight_*`): each item names its neighbours by number; a link of 0x6F is
//! none, 0xFFFFFC1A "ask the item" (its callback for that direction runs instead). The two
//! vertical moves go on along the same link until an item that can be chosen (its +0x48
//! 0xFFFFFC1B); the two horizontal ones take one step and may first turn a counter.

use crate::Invalid;

pub const NO_LINK: u32 = 0x6F;
pub const ASK_ITEM: u32 = 0xFFFF_FC1A;
pub const CAN_BE_CHOSEN: u32 = 0xFFFF_FC1B;
/// No menu has more items; a walk past it is refused.
pub const STEP_LIMIT: u32 = 256;

/// The counter left and right turn while it is on (`dword_75AA78` its count,
/// `dword_75AA7C` the value, `dword_75A660` the way, `dword_75AA80` the step).
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
