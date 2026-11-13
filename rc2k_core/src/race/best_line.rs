//! The best line's later steps: distances along the line, each checkpoint's nearest replay
//! camera, the share of the line on each surface, link angles, and the line's stretches between
//! map anchors. Distances and angles come from closures asked in a fixed order;
//! the share's sum keeps `f64` intermediates rounded to `f32` where stored.

/// The most faces a set of replay cameras lists.
pub const MOST_CAMERAS: u32 = 0x400;

/// Distance along the line at each checkpoint 0 ..= `last`: the running sum, `length(k)` asked
/// in order for the link k to k + 1.
pub fn distances(last: u32, mut length: impl FnMut(u32) -> u32) -> Vec<u32> {
    let mut sums = Vec::with_capacity(last as usize + 1);
    let mut sum = 0u32;
    for k in 0..last {
        let l = length(k);
        sums.push(sum);
        sum = sum.wrapping_add(l);
    }
    sums.push(sum);
    sums
}

/// The replay cameras' faces from face 0 on (`is_camera`), up to `MOST_CAMERAS`. Gives the list
/// and the face the search stopped at.
pub fn camera_faces(faces: u32, mut is_camera: impl FnMut(u32) -> bool) -> (Vec<u32>, u32) {
    let mut list = Vec::new();
    let mut face = 0u32;
    loop {
        if is_camera(face) {
            list.push(face);
            if list.len() as u32 == MOST_CAMERAS {
                return (list, face);
            }
        }
        face += 1;
        if face == faces {
            return (list, face);
        }
    }
}

/// The nearest of `count` cameras (`distance(e)` asked in order; the last wins a tie): its
/// distance and number.
pub fn nearest_camera(count: u32, mut distance: impl FnMut(u32) -> u32) -> (u32, u32) {
    let mut best = (u32::MAX, u32::MAX);
    for e in 0..count {
        let d = distance(e);
        if d <= best.0 {
            best = (d, e);
        }
    }
    best
}

/// The weight of a face's second texture in the surface shares (of 0x100): a quarter of the
/// face's four blend bytes; 0 without a second texture or when its kind's surface is 4 or 5.
pub fn second_weight(blend: [u8; 4], second: u8, second_surface: u32) -> u32 {
    let weight = blend.iter().map(|&b| b as u32).sum::<u32>() >> 2;
    if second == 0 || second_surface == 5 || second_surface == 4 {
        0
    } else {
        weight
    }
}

/// The first texture's weight: what the second leaves of 0x100.
pub fn first_weight(second: u32) -> u32 {
    0x100u32.wrapping_sub(second)
}

/// The surface share's sum: the share plus weight / `per` x `quarter` / checkpoints x
/// `percent`, in that order of operations.
pub fn surface_share(
    weight: i32,
    per: f32,
    quarter: f32,
    checkpoints: i32,
    percent: f32,
    share: f32,
) -> f32 {
    let v = weight as f64 / per as f64 * quarter as f64 / checkpoints as f64 * percent as f64;
    (v + share as f64) as f32
}

/// The checkpoint angles table, in write order: with two checkpoints or more,
/// `angle(b)` (from 1 on) goes to b + 1, the last again to count + 1, then 0 and 1 as 2 (fewer
/// checkpoints: `at_2`). Gives the writes (place, angle) and the angle at 2.
pub fn link_angles(
    count: u32,
    mut angle: impl FnMut(u32) -> u32,
    at_2: impl FnOnce() -> u32,
) -> (Vec<(u32, u32)>, u32) {
    let mut writes = Vec::new();
    let second = if count >= 2 {
        let mut last = 0;
        for b in 1..count {
            last = angle(b);
            writes.push((b + 1, last));
        }
        writes.push((count + 1, last));
        writes[0].1
    } else {
        at_2()
    };
    writes.push((1, second));
    writes.push((0, second));
    (writes, second)
}

/// The line cut at its map anchors from checkpoint 0: `next_anchor` ends a `stretch(from, to)`,
/// until there is none.
pub fn map_stretches<E>(
    mut next_anchor: impl FnMut(u32) -> Result<Option<u32>, E>,
    mut stretch: impl FnMut(u32, u32) -> Result<(), E>,
) -> Result<(), E> {
    let mut from = 0u32;
    while let Some(to) = next_anchor(from.wrapping_add(1))? {
        stretch(from, to)?;
        from = to;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn line() {
        assert_eq!(distances(3, |k| [5, 7, 9][k as usize]), [0, 5, 12, 21]);
        assert_eq!(distances(0, |_| unreachable!()), [0]);
        let (list, stopped) = camera_faces(10, |f| f % 3 == 2);
        assert_eq!((list, stopped), (vec![2, 5, 8], 10));
        let (list, stopped) = camera_faces(5000, |_| true);
        assert_eq!(
            (list.len() as u32, stopped),
            (MOST_CAMERAS, MOST_CAMERAS - 1)
        );
        assert_eq!(nearest_camera(4, |e| [9, 3, 7, 3][e as usize]), (3, 3));
        assert_eq!(nearest_camera(1, |_| u32::MAX), (u32::MAX, 0));
    }

    #[test]
    fn angles_and_stretches() {
        let (w, a) = link_angles(3, |b| 10 * b, || unreachable!());
        assert_eq!(w, [(2, 10), (3, 20), (4, 20), (1, 10), (0, 10)]);
        assert_eq!(a, 10);
        let (w, a) = link_angles(2, |b| 7 + b, || unreachable!());
        assert_eq!((w, a), (vec![(2, 8), (3, 8), (1, 8), (0, 8)], 8));
        let (w, a) = link_angles(1, |_| unreachable!(), || 5);
        assert_eq!((w, a), (vec![(1, 5), (0, 5)], 5));
        let anchors = [3u32, 4, 9];
        let mut seen = Vec::new();
        let r: Result<(), ()> = map_stretches(
            |from| Ok(anchors.iter().copied().find(|&a| a >= from)),
            |a, b| {
                seen.push((a, b));
                Ok(())
            },
        );
        assert_eq!((r, seen), (Ok(()), vec![(0, 3), (3, 4), (4, 9)]));
    }

    #[test]
    fn shares() {
        assert_eq!(second_weight([0x40, 0x40, 0x40, 0x40], 3, 0), 0x40);
        assert_eq!(second_weight([0x40; 4], 0, 0), 0);
        assert_eq!(second_weight([0x40; 4], 3, 4), 0);
        assert_eq!(second_weight([0xFF; 4], 3, 5), 0);
        assert_eq!(first_weight(0x40), 0xC0);
        assert_eq!(surface_share(0x100, 256.0, 0.25, 1, 100.0, 1.0), 26.0);
        assert_eq!(
            surface_share(0x80, 256.0, 0.25, 7, 100.0, 0.0),
            (0.5f64 * 0.25 / 7.0 * 100.0) as f32
        );
    }
}
