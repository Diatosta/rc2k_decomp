//! Gathering a face's points into a list without repeats: each point looked for among those already
//! there, added at the end when new, until the list is full.

/// What adding a face's points leaves.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Collected {
    /// The last point read (none when no point was read).
    pub last_point: Option<u32>,
    /// Where the last search stopped.
    pub stopped: Option<u32>,
    /// The points stepped over (the cursor moves 0x10 a point; not once the list is
    /// full, when the rest are skipped unread).
    pub stepped: u32,
}

/// The face's `points` points (signed; `point(k)` asked for each looked at) added to
/// `ids[..count]` unless there already, while fewer than `ids.len()`; `added(slot, k)`
/// for each new one after its id is in.
pub(crate) fn add_points(
    count: &mut u32,
    ids: &mut [u32],
    points: u32,
    point: impl Fn(u32) -> u32,
    mut added: impl FnMut(usize, u32),
) -> Collected {
    let mut c = Collected::default();
    for k in 0..(points as i32).max(0) as u32 {
        let n = *count as usize;
        if n == ids.len() {
            continue;
        }
        let id = point(k);
        c.last_point = Some(id);
        let at = ids[..n].iter().position(|&i| i == id).unwrap_or(n);
        c.stopped = Some(at as u32);
        if at == n {
            ids[n] = id;
            added(n, k);
            *count += 1;
        }
        c.stepped += 1;
    }
    c
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adds() {
        let mut ids = [4u32, 0, 0];
        let mut count = 1;
        let mut new = Vec::new();
        let pts = [4u32, 5, 5, 6, 7];
        let c = add_points(
            &mut count,
            &mut ids,
            5,
            |k| pts[k as usize],
            |n, k| new.push((n, k)),
        );
        assert_eq!((count, ids, new), (3, [4, 5, 6], vec![(1, 1), (2, 3)]));
        assert_eq!(
            c,
            Collected {
                last_point: Some(6),
                stopped: Some(2),
                stepped: 4
            }
        );
    }
}
