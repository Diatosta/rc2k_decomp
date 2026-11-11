//! The pictures the carousel's entries show (from
//! `scene::choose_carousel_pictures` 0x4901B9 and `scene::update_carousel_pictures`
//! 0x49029C, with `keep_held_pictures` 0x49036F and `assign_picture_slots` 0x4903E3
//! inlined, `scene::draw_carousel` 0x490649 and the cars' lists of `scene::set_up_carousel`
//! 0x4904DD): the four entries that can be seen, the four pictures held in the four
//! places, each place's eight texture slots, the entries drawn, and the lists of cars the
//! cars' carousels show.

use crate::Invalid;

/// No picture.
pub const NONE: u32 = u32::MAX;

/// The menus whose carousel shows pictures (`g_carouselMenu` 0xB .. 0xE).
pub fn shows_pictures(menu: u32) -> bool {
    (0xB..=0xE).contains(&menu)
}

/// The four entries that can be seen: the one before the front one and the three after
/// it, going round `count` (wrapping as the original's compares do).
pub fn visible_entries(front: u32, count: u32) -> [u32; 4] {
    let mut entry = if front == 0 { count } else { front }.wrapping_sub(1);
    let mut entries = [entry; 4];
    for next in &mut entries[1..] {
        entry = entry.wrapping_add(1);
        if entry == count {
            entry = 0;
        }
        *next = entry;
    }
    entries
}

/// `keep_held_pictures`: the held pictures still wanted stay where they are (and are no
/// longer wanted), the others give their places to the newly wanted, in order. Gives
/// what is left wanted and the held.
pub fn keep_held(mut wanted: [u32; 4], mut held: [u32; 4]) -> ([u32; 4], [u32; 4]) {
    for held in &mut held {
        match wanted.iter().position(|w| w == held) {
            Some(place) => wanted[place] = NONE,
            None => *held = NONE,
        }
    }
    for &w in &wanted {
        if w != NONE
            && let Some(free) = held.iter().position(|&h| h == NONE)
        {
            held[free] = w;
        }
    }
    (wanted, held)
}

/// `assign_picture_slots`: each place's eight slots want its picture's eight textures
/// (the picture times 8 from the kind's first texture, wrapping).
pub fn picture_textures(held: [u32; 4], first_texture: u32) -> [u32; 32] {
    std::array::from_fn(|n| {
        (held[n / 8] << 3)
            .wrapping_add(first_texture)
            .wrapping_add(n as u32 % 8)
    })
}

/// The entries `draw_carousel` draws.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Drawn {
    /// None (no entries, or a count past 0x80000000, signed).
    Nothing,
    /// A carousel of one: entry 0, the place whatever the caller left.
    Lone,
    /// (entry, place): the one before the front at -1, the front at 0, the next at 1, and
    /// while the carousel turns the one after at 2.
    Row(Vec<(u32, u32)>),
}

/// `draw_carousel`'s entries: `front` (`g_carouselFront`) and `turning`
/// (`g_carouselAngle` not 0) asked only for a row.
pub fn drawn(count: u32, front: impl FnOnce() -> u32, turning: impl FnOnce() -> bool) -> Drawn {
    if count == 1 {
        return Drawn::Lone;
    }
    if (count.wrapping_sub(1) as i32) < 0 {
        return Drawn::Nothing;
    }
    let entries = visible_entries(front(), count);
    let shown = if turning() { 4 } else { 3 };
    Drawn::Row(
        entries[..shown]
            .iter()
            .zip([u32::MAX, 0, 1, 2])
            .map(|(&e, place)| (e, place))
            .collect(),
    )
}

/// A list of cars' entries looked at for its end (-1), and the car numbers a kind is
/// picked by (`dword_75A1FE`).
pub const LIST_ENTRIES: usize = 0x100;
pub const CAR_NUMBERS: u32 = 0x400;
pub const END: u32 = u32::MAX;

/// Where a list's end is from its entry `from` on (`entry(n)` the list's), within
/// `LIST_ENTRIES`.
pub fn list_end(entry: impl Fn(usize) -> u32, from: usize) -> Option<usize> {
    (from..LIST_ENTRIES).find(|&n| entry(n) == END)
}

/// The checks of a list of cars (each entry a car, its first dword its number:
/// `number(car)`), before anything is written: refuses a list without its end, a
/// car that is not there and a car's number past the kinds' table. With `from_second`
/// (menu 0xE) the list as it will be after a list of one has had its car copied over its
/// end.
pub fn check_cars(
    entry: impl Fn(usize) -> u32,
    number: impl Fn(u32) -> u32,
    from_second: bool,
) -> Result<(), Invalid> {
    let copied = if from_second {
        let count = list_end(&entry, 1).ok_or(Invalid("a list of cars without its end"))?;
        count == 1
    } else {
        false
    };
    for n in 0..LIST_ENTRIES {
        let car = if copied && n == 1 { entry(0) } else { entry(n) };
        if car == END {
            return Ok(());
        }
        if car == 0 {
            return Err(Invalid("a car that is not there"));
        }
        if number(car) >= CAR_NUMBERS {
            return Err(Invalid("a car's number past the table"));
        }
    }
    Err(Invalid("a list of cars without its end"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn carousel() {
        assert!(!shows_pictures(0xA) && shows_pictures(0xB) && !shows_pictures(0xF));
        assert_eq!(visible_entries(0, 5), [4, 0, 1, 2]);
        assert_eq!(visible_entries(3, 5), [2, 3, 4, 0]);
        assert_eq!(visible_entries(0, 0), [u32::MAX, 0, 1, 2]);
        assert_eq!(
            keep_held([3, 9, 9, 0x41], [9, 7, NONE, 3]),
            ([NONE, NONE, 9, 0x41], [9, 9, 0x41, 3])
        );
        let t = picture_textures([1, NONE, 0, 2], 0x100);
        assert_eq!(
            (t[0], t[7], t[8], t[15], t[16], t[31]),
            (0x108, 0x10F, 0xF8, 0xFF, 0x100, 0x117)
        );
    }

    #[test]
    fn drawn_entries() {
        assert_eq!(drawn(1, || unreachable!(), || unreachable!()), Drawn::Lone);
        assert_eq!(
            drawn(0, || unreachable!(), || unreachable!()),
            Drawn::Nothing
        );
        assert_eq!(
            drawn(5, || 0, || false),
            Drawn::Row(vec![(4, u32::MAX), (0, 0), (1, 1)])
        );
        assert_eq!(
            drawn(3, || 2, || true),
            Drawn::Row(vec![(1, u32::MAX), (2, 0), (0, 1), (1, 2)])
        );
    }

    #[test]
    fn car_lists() {
        let list = [0x100u32, 0x200, END, 0x300, END];
        let entry = |n: usize| list.get(n).copied().unwrap_or(0);
        assert_eq!(list_end(entry, 1), Some(2));
        assert!(check_cars(entry, |c| c / 0x100, false).is_ok());
        assert!(check_cars(entry, |c| c * 4, false).is_err());
        // A list of one has its car over its end: then read on to the next end.
        let one = [0x100u32, END, 0x200, END];
        let entry = |n: usize| one.get(n).copied().unwrap_or(0);
        assert!(check_cars(entry, |_| 1, true).is_ok());
        assert!(check_cars(entry, |_| 1, false).is_ok());
        let one = [0x100u32, END, 0, END];
        let entry = |n: usize| one.get(n).copied().unwrap_or(0);
        assert!(check_cars(entry, |_| 1, true).is_err());
        let two = [0x100u32, 0x100, 0, END];
        let entry = |n: usize| two.get(n).copied().unwrap_or(0);
        assert!(check_cars(entry, |_| 1, false).is_err());
    }
}
