//! The options pages' small steps (converted 2026-10-21 from the eight volume steps
//! `menu::music_volume_down` 0x467F48 .. `menu_sounds_b_volume_up` 0x46810A,
//! `menu::next_screen_mode` 0x46FFEF, the two texture steps 0x47012B / 0x47015B and the
//! four colour steps 0x471AE4 .. 0x471B7A; the controller pages' steps, `menu::next_controller`
//! 0x46A3BF .. `raise_dead_zone_b` 0x46B3EA; the event pages' row steps 0x4A3736 .. 0x4A37B5;
//! the stage choice's `switch_stage_group`, `next_stage_sub` / `previous_stage_sub`,
//! `next_stage_number` / `previous_stage_number`; the stage pages' `previous_stage_page`
//! 0x46FAEE / `next_stage_page` 0x46FB27).

/// A volume one less (below 0: 0) or one more (above 0x40: 0x40), signed.
pub fn volume(value: i32, up: bool) -> i32 {
    if up {
        value.wrapping_add(1).min(0x40)
    } else {
        value.wrapping_sub(1).max(0)
    }
}

/// The screen mode chosen one more, 0 past the last (`count` - 1, compared signed).
pub fn next_mode(mode: u32, count: u32) -> u32 {
    let next = mode.wrapping_add(1);
    if next as i32 > count.wrapping_sub(1) as i32 {
        0
    } else {
        next
    }
}

/// The texture setting cycled through 0 .. 2 (a value past 2 just moves on by one).
pub fn texture(value: u32, up: bool) -> u32 {
    match (up, value) {
        (false, 0) => 2,
        (false, v) => v.wrapping_sub(1),
        (true, 2) => 0,
        (true, v) => v.wrapping_add(1),
    }
}

/// A colour's number cycled through 0 .. 6 (signed: a number past 6 going up is 0, one
/// below 0 going down 6).
pub fn colour(n: i32, up: bool) -> i32 {
    if up {
        let n = n.wrapping_add(1);
        if n < 7 { n } else { 0 }
    } else {
        let n = n.wrapping_sub(1);
        if n >= 0 { n } else { 6 }
    }
}

/// The controller after `c` (after the `count` listed ones, 0: none).
pub fn next_controller(c: u32, count: u32) -> u32 {
    if c == count { 0 } else { c.wrapping_add(1) }
}

/// The controller before `c` (0 or less, signed: the last listed).
pub fn previous_controller(c: u32, count: u32) -> u32 {
    if c as i32 <= 0 { count } else { c - 1 }
}

/// Whether controller `c` is a listed one (`c - 1` below the count, unsigned).
pub fn listed(c: u32, count: u32) -> bool {
    c.wrapping_sub(1) < count
}

/// What the two dead zones leave of 100, and whether a third may grow (above 10, signed).
pub fn dead_zone_room(a: u32, b: u32) -> (u32, bool) {
    let room = 100u32.wrapping_sub(a).wrapping_sub(b);
    (room, room as i32 > 10)
}

/// The first row shown ten back (at 10 or below, signed: `last`) or on (at `last` or past,
/// signed: 1); `last` is the rows' count less 9.
pub fn rows(row: u32, count: u32, on: bool) -> u32 {
    let last = count.wrapping_sub(9);
    if on {
        if row as i32 >= last as i32 {
            1
        } else {
            row.wrapping_add(10)
        }
    } else if row as i32 <= 10 {
        last
    } else {
        row - 10
    }
}

/// The stage choice's sub-table on (5 or more, unsigned, to 0) or back (below 1, signed,
/// to 5).
pub fn stage_sub(s: u32, up: bool) -> u32 {
    if up {
        if s >= 5 { 0 } else { s + 1 }
    } else if (s as i32) < 1 {
        5
    } else {
        s - 1
    }
}

/// The stage choice's number on (6 or more, unsigned, to 1) or back (below 2, signed, to 6).
pub fn stage_number(n: u32, up: bool) -> u32 {
    if up {
        if n >= 6 { 1 } else { n + 1 }
    } else if (n as i32) >= 2 {
        n - 1
    } else {
        6
    }
}

/// The stage's page on (past `pages`, signed: 0) or back (from 0: `pages`).
pub fn stage_page(page: u32, pages: u32, up: bool) -> u32 {
    if up {
        if (page as i32) < pages as i32 {
            page.wrapping_add(1)
        } else {
            0
        }
    } else if page == 0 {
        pages
    } else {
        page - 1
    }
}

/// A switch flipped (`xor 1`: only bit 0 changes).
pub fn flip(value: u32) -> u32 {
    value ^ 1
}

/// The advanced audio page's quality on (0 after 3) or back (3 before 0); other values
/// just move by one.
pub fn quality(q: u32, up: bool) -> u32 {
    match (up, q) {
        (true, 3) => 0,
        (true, q) => q.wrapping_add(1),
        (false, 0) => 3,
        (false, q) => q.wrapping_sub(1),
    }
}

/// A counter between `low` and `high` one on or back, past an end the other end (signed).
pub fn counter(value: i32, up: bool, low: i32, high: i32) -> i32 {
    if up {
        let n = value.wrapping_add(1);
        if n > high { low } else { n }
    } else {
        let n = value.wrapping_sub(1);
        if n < low { high } else { n }
    }
}

/// The screen mode chosen one back, before 0 (signed) the last (`count` - 1).
pub fn previous_mode(mode: i32, count: i32) -> i32 {
    let n = mode.wrapping_sub(1);
    if n < 0 { count.wrapping_sub(1) } else { n }
}

/// The best scores' page on (11 back to 0) or back (below 0 to 10), signed.
pub fn scores_page(page: u32, up: bool) -> u32 {
    if up {
        let v = page.wrapping_add(1);
        if (v as i32) < 11 { v } else { 0 }
    } else {
        let v = page.wrapping_sub(1);
        if (v as i32) < 0 { 10 } else { v }
    }
}

/// The replays' range on (past 4 to 0) or back (below 0 to 4), signed.
pub fn replay_range(range: u32, up: bool) -> u32 {
    if up {
        let v = range.wrapping_add(1);
        if (v as i32) > 4 { 0 } else { v }
    } else {
        let v = range.wrapping_sub(1);
        if (v as i32) < 0 { 4 } else { v }
    }
}

/// The saved games' list page on (above 9 to 0) or back (below 0 to 9), signed.
pub fn savegame_page(page: u32, up: bool) -> u32 {
    if up {
        let v = page.wrapping_add(1);
        if (v as i32) > 9 { 0 } else { v }
    } else {
        let v = page.wrapping_sub(1);
        if (v as i32) < 0 { 9 } else { v }
    }
}

/// `menu::flip_gearing`'s text: string 0x221 when the switch is then 0, else 0x21F.
pub fn gearing_switch_text(switch: u32) -> u32 {
    if switch == 0 { 0x221 } else { 0x21F }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn controllers_and_rows() {
        assert_eq!(
            (
                stage_page(2, 2, true),
                stage_page(0, 2, false),
                stage_page(1, 2, true)
            ),
            (0, 2, 2)
        );
        assert_eq!(
            (
                stage_sub(5, true),
                stage_sub(0, false),
                stage_number(6, true),
                stage_number(1, false)
            ),
            (0, 5, 1, 6)
        );
        assert_eq!(
            (stage_sub(u32::MAX, true), stage_number(u32::MAX, false)),
            (0, 6)
        );
        assert_eq!(
            (
                next_controller(3, 3),
                previous_controller(0, 3),
                listed(0, 3),
                listed(3, 3)
            ),
            (0, 3, false, true)
        );
        assert_eq!(dead_zone_room(40, 50), (10, false));
        assert_eq!(
            (
                rows(81, 90, true),
                rows(71, 90, true),
                rows(10, 90, false),
                rows(21, 90, false)
            ),
            (1, 81, 81, 11)
        );
    }

    #[test]
    fn steps() {
        assert_eq!(
            (
                volume(0x40, true),
                volume(0, false),
                volume(i32::MIN, false)
            ),
            (0x40, 0, i32::MAX)
        );
        assert_eq!(volume(i32::MAX, true), i32::MIN);
        assert_eq!(
            (next_mode(3, 4), next_mode(2, 4), next_mode(0, 0)),
            (0, 3, 0)
        );
        assert_eq!(
            (texture(0, false), texture(2, true), texture(5, true)),
            (2, 0, 6)
        );
        assert_eq!(
            (colour(6, true), colour(0, false), colour(9, true)),
            (0, 6, 0)
        );
    }

    #[test]
    fn option_items() {
        assert_eq!((flip(0), flip(1), flip(2)), (1, 0, 3));
        assert_eq!(
            (
                quality(3, true),
                quality(0, false),
                quality(1, true),
                quality(7, false)
            ),
            (0, 3, 2, 6)
        );
        assert_eq!(
            (
                counter(5, true, 0, 5),
                counter(0, false, 0, 5),
                counter(3, true, 1, 6)
            ),
            (0, 5, 4)
        );
        assert_eq!(counter(-4, false, 1, 6), 6);
        assert_eq!(
            (
                previous_mode(0, 7),
                previous_mode(3, 7),
                previous_mode(-2, 7)
            ),
            (6, 2, 6)
        );
    }

    #[test]
    fn pages_and_ranges() {
        assert_eq!(
            (
                scores_page(10, true),
                scores_page(0, false),
                scores_page(3, true)
            ),
            (0, 10, 4)
        );
        assert_eq!(scores_page(0x8000_0000, false), 0x7FFF_FFFF);
        assert_eq!(
            (
                replay_range(4, true),
                replay_range(0, false),
                replay_range(2, false)
            ),
            (0, 4, 1)
        );
    }

    #[test]
    fn savegame_pages() {
        assert_eq!(
            (
                savegame_page(9, true),
                savegame_page(0, false),
                savegame_page(4, true)
            ),
            (0, 9, 5)
        );
        assert_eq!(
            (gearing_switch_text(0), gearing_switch_text(1)),
            (0x221, 0x21F)
        );
    }
}
