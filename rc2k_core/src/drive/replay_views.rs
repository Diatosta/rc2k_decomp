//! The replay's cameras and the two-car race's sound levels (from
//! `drive::replay_cameras` 0x460949, `drive::pick_replay_camera` 0x460AA6 and
//! `drive::duel_sound_levels` 0x45C596): a replay frame's camera wishes made into views
//! (a wish 0x23 .. 0x25 a view of a kind drawn at random, at most every three frames),
//! and two levels 0 ..= 0x40 from the first car block's figures (a reading).

/// The camera modes `pick_replay_camera` draws from.
pub const MODES: u32 = 0x34;

/// The wishes that draw a mode, and the kind each stops at.
pub fn wish_kind(wish: u32) -> Option<u32> {
    match wish {
        0x23 => Some(0),
        0x24 => Some(1),
        0x25 => Some(2),
        _ => None,
    }
}

/// Whether a drawn mode of kind `mode_kind` ends the draws for `kind`: its kind, but for
/// kind 0 any other kind (0x23's loop is `cmp [ebx], 0 / jz`; notes/bugs.md).
pub fn fits(kind: u32, mode_kind: u32) -> bool {
    (mode_kind == kind) != (kind == 0)
}

/// A draw: a random number x `MODES` as `mul` leaves it, (eax, edx the mode).
pub fn draw(random: u32) -> (u32, u32) {
    let product = u64::from(random) * u64::from(MODES);
    (product as u32, (product >> 32) as u32)
}

/// The kinds past which a car has no views 2 and 3 (`g_carBlockStore` +0x24).
pub const LAST_VIEW_KIND: u32 = 0x16;

/// The mode drawn as kept: 2 and 3 become 4 when the first car's kind is past
/// `LAST_VIEW_KIND`, or (`two()` with two cars) the second's; asked only as needed.
pub fn kept_mode(
    mode: u32,
    first_kind: impl FnOnce() -> u32,
    two: impl FnOnce() -> bool,
    second_kind: impl FnOnce() -> u32,
) -> u32 {
    if matches!(mode, 2 | 3)
        && (first_kind() > LAST_VIEW_KIND || (two() && second_kind() > LAST_VIEW_KIND))
    {
        4
    } else {
        mode
    }
}

/// The frames between two draws for a car.
pub const DRAW_GAP: u32 = 3;

/// What `replay_cameras` does for one car's wish.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WishStep {
    /// Nothing (a drawing wish too soon, with the frames since the last draw: the
    /// original's ebx; or the wish already the view).
    Nothing(Option<u32>),
    /// A drawing wish: a mode drawn (the frames since, ebx).
    Draw(u32),
    /// Another wish: it is the view (and the wanted one), the last draw's frame 0.
    Set,
}

/// One car's wish at `frame`, its last draw at `last`, its view `view()` (asked only for a
/// wish that does not draw).
pub fn wish_step(wish: u32, frame: u32, last: u32, view: impl FnOnce() -> u32) -> WishStep {
    if wish_kind(wish).is_some() {
        let since = frame.wrapping_sub(last);
        if since >= DRAW_GAP {
            WishStep::Draw(since)
        } else {
            WishStep::Nothing(Some(since))
        }
    } else if wish != view() {
        WishStep::Set
    } else {
        WishStep::Nothing(None)
    }
}

/// The levels' top, and the first level from which a rare draw may set the flag.
pub const LEVEL_TOP: u32 = 0x40;
pub const RARE_FROM: u32 = 0x38;

/// A level held to 0 ..= `LEVEL_TOP` (by the signs).
pub fn held_level(v: u32) -> u32 {
    let v = if (v as i32) < 0 { 0 } else { v };
    if (v.wrapping_sub(LEVEL_TOP) as i32) < 0 {
        v
    } else {
        LEVEL_TOP
    }
}

/// The first level: the block's +0x900 / 4 - 4, held.
pub fn first_level(figure: u32) -> u32 {
    held_level((figure >> 2).wrapping_sub(4))
}

/// Whether a rare draw sets the flag: the random number (signed) x 500 has a high dword
/// of 0.
pub fn rare_draw(random: u32) -> bool {
    (i64::from(random as i32) * 500) >> 32 == 0
}

/// The second level from the block's +0x908: (its square's low dword >> 12 - 8, held;
/// the square's high dword, the original's edx).
pub fn second_level(figure: u32) -> (u32, u32) {
    let square = i64::from(figure as i32) * i64::from(figure as i32);
    (
        held_level(((square as u32) >> 12).wrapping_sub(8)),
        (square >> 32) as u32,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn picks() {
        assert_eq!(wish_kind(0x24), Some(1));
        assert_eq!(wish_kind(0x26), None);
        assert!(fits(0, 1));
        assert!(!fits(0, 0));
        assert!(fits(2, 2));
        assert_eq!(draw(u32::MAX), (u32::MAX.wrapping_mul(0x34), 0x33));
        assert_eq!(
            kept_mode(2, || 0x17, || unreachable!(), || unreachable!()),
            4
        );
        assert_eq!(kept_mode(3, || 1, || false, || unreachable!()), 3);
        assert_eq!(kept_mode(3, || 1, || true, || 0x17), 4);
        assert_eq!(
            kept_mode(5, || unreachable!(), || unreachable!(), || unreachable!()),
            5
        );
    }

    #[test]
    fn wishes() {
        assert_eq!(wish_step(0x23, 10, 7, || unreachable!()), WishStep::Draw(3));
        assert_eq!(
            wish_step(0x23, 10, 8, || unreachable!()),
            WishStep::Nothing(Some(2))
        );
        assert_eq!(wish_step(5, 10, 8, || 5), WishStep::Nothing(None));
        assert_eq!(wish_step(5, 10, 8, || 4), WishStep::Set);
    }

    #[test]
    fn levels() {
        assert_eq!(first_level(0), 0);
        assert_eq!(first_level(0x200), 0x40);
        assert_eq!(first_level(0x40), 0xC);
        assert!(rare_draw(5));
        assert!(!rare_draw(u32::MAX));
        assert!(!rare_draw(0x0100_0000));
        assert_eq!(second_level(0x100), (8, 0));
        assert_eq!(second_level(0x10000), (0, 1));
    }
}
