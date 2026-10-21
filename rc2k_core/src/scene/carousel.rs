//! The pictures the carousel's entries show (converted 2026-10-21 from
//! `scene::choose_carousel_pictures` 0x4901B9 and `scene::update_carousel_pictures`
//! 0x49029C, with `keep_held_pictures` 0x49036F and `assign_picture_slots` 0x4903E3
//! inlined): the four entries that can be seen, the four pictures held in the four
//! places, and each place's eight texture slots.

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
}
