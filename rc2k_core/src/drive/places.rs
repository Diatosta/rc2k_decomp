//! The cars' order: the two places of a two-car race, and the nearest gap in checkpoints to the
//! lead car.

/// The car block's figures `two_car_order` reads, by their place in the block.
pub const FINISHED: u32 = 0xA8;
pub const CHECKPOINT: u32 = 0x94;
/// The tie-break figure (`checkpoint_offset[1]`: the car's z in its checkpoint's frame,
/// `physics::checkpoint_offset`).
pub const TIE_BREAK: u32 = 0x624;

/// The places to give the two cars (`None`: left as they are) and the last figure compared
/// (`last_figure` as it came in when none was).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Order {
    pub places: Option<[u32; 2]>,
    pub last: u32,
}

/// The places of a two-car race: -1 for both outside mode 0 or with one view (`players` 1); left
/// as they are while either is finished; else 1 for the one ahead (by checkpoint, then the
/// tie-break, each by the sign of the difference), 2 for the other. `figure(car, offset)` reads a
/// car block's dword, asked only when needed.
pub fn two_car_order(
    mode: u32,
    players: u32,
    figure: impl Fn(usize, u32) -> u32,
    last_figure: u32,
) -> Order {
    if mode != 0 || players == 1 {
        return Order {
            places: Some([u32::MAX, u32::MAX]),
            last: last_figure,
        };
    }
    if figure(0, FINISHED) != 0 || figure(1, FINISHED) != 0 {
        return Order {
            places: None,
            last: last_figure,
        };
    }
    let mut last = figure(0, CHECKPOINT);
    let d = last.wrapping_sub(figure(1, CHECKPOINT));
    let behind = if (d as i32) < 0 {
        true
    } else if d != 0 {
        false
    } else {
        last = figure(0, TIE_BREAK);
        (last.wrapping_sub(figure(1, TIE_BREAK)) as i32) < 0
    };
    Order {
        places: Some(if behind { [2, 1] } else { [1, 2] }),
        last,
    }
}

/// The places (1 the first) of the `players` cars of a split screen of more than two, by the
/// order the two-car race works out: nobody changes while a car is finished (`None`); else a car is
/// ahead of another by its checkpoint, then the tie-break, each by the sign of the difference,
/// and the lower-numbered car on a tie. `figure` is as for `two_car_order`.
pub fn many_car_order(players: usize, figure: impl Fn(usize, u32) -> u32) -> Option<Vec<u32>> {
    if (0..players).any(|car| figure(car, FINISHED) != 0) {
        return None;
    }
    let signed = |a: u32, b: u32| a.wrapping_sub(b) as i32;
    let ahead = |j: usize, i: usize| {
        let d = signed(figure(j, CHECKPOINT), figure(i, CHECKPOINT));
        let d = if d == 0 {
            signed(figure(j, TIE_BREAK), figure(i, TIE_BREAK))
        } else {
            d
        };
        d > 0 || (d == 0 && j < i)
    };
    Some(
        (0..players)
            .map(|i| 1 + (0..players).filter(|&j| j != i && ahead(j, i)).count() as u32)
            .collect(),
    )
}

/// The gap ahead or behind: the car's checkpoint against the lead car's (ahead: car - lead,
/// behind: lead - car), plus one; it replaces `best` when below it (unsigned).
pub fn note_gap(car: u32, lead: u32, behind: bool, best: &mut u32) -> u32 {
    let d = if behind {
        lead.wrapping_sub(car)
    } else {
        car.wrapping_sub(lead)
    };
    let gap = d.wrapping_add(1);
    if gap < *best {
        *best = gap;
    }
    gap
}

#[cfg(test)]
mod tests {
    use super::*;

    fn blocks(a: [u32; 3], b: [u32; 3]) -> impl Fn(usize, u32) -> u32 {
        move |car, o| {
            let f = if car == 0 { a } else { b };
            match o {
                FINISHED => f[0],
                CHECKPOINT => f[1],
                _ => f[2],
            }
        }
    }

    #[test]
    fn many_cars_order() {
        // (finished, checkpoint, tie-break) of four cars: car 2 leads, 0 and 3 tie on the checkpoint
        // and 3 is ahead by the tie-break, 1 is last.
        let cars = [[0, 5, 1], [0, 2, 9], [0, 7, 0], [0, 5, 4]];
        let figure = |car: usize, o: u32| match o {
            FINISHED => cars[car][0],
            CHECKPOINT => cars[car][1],
            _ => cars[car][2],
        };
        assert_eq!(many_car_order(4, figure), Some(vec![3, 4, 1, 2]));
        // Equal cars keep their numbers' order; the first three only.
        let same = |_: usize, o: u32| u32::from(o == CHECKPOINT);
        assert_eq!(many_car_order(3, same), Some(vec![1, 2, 3]));
        // Nobody's place changes once a car has finished.
        let done = |car: usize, o: u32| u32::from(car == 1 && o == FINISHED);
        assert_eq!(many_car_order(4, done), None);
    }

    #[test]
    fn order() {
        let o = two_car_order(1, 2, blocks([0; 3], [0; 3]), 9);
        assert_eq!(
            o,
            Order {
                places: Some([u32::MAX; 2]),
                last: 9
            }
        );
        let o = two_car_order(0, 2, blocks([1, 0, 0], [0; 3]), 9);
        assert_eq!(
            o,
            Order {
                places: None,
                last: 9
            }
        );
        let o = two_car_order(0, 2, blocks([0, 3, 0], [0, 4, 0]), 9);
        assert_eq!(
            o,
            Order {
                places: Some([2, 1]),
                last: 3
            }
        );
        let o = two_car_order(0, 2, blocks([0, 4, 5], [0, 4, 6]), 9);
        assert_eq!(
            o,
            Order {
                places: Some([2, 1]),
                last: 5
            }
        );
        let o = two_car_order(0, 2, blocks([0, 4, 6], [0, 4, 6]), 9);
        assert_eq!(
            o,
            Order {
                places: Some([1, 2]),
                last: 6
            }
        );
    }

    #[test]
    fn gaps() {
        let mut best = 10;
        assert_eq!(note_gap(5, 3, false, &mut best), 3);
        assert_eq!(best, 3);
        assert_eq!(note_gap(5, 3, true, &mut best), u32::MAX);
        assert_eq!(best, 3);
        assert_eq!(note_gap(4, 4, true, &mut best), 1);
        assert_eq!(best, 1);
    }
}
