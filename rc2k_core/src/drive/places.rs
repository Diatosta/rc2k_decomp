//! The cars' order (converted 2026-10-21 from `drive::two_car_order` 0x460CDA,
//! `note_gap_ahead` 0x4373A4 and `note_gap_behind`): the two places of a two-car race,
//! and the nearest gap in checkpoints to the lead car.

/// A car block's figures `two_car_order` reads.
pub const FINISHED: u32 = 0xA8;
pub const CHECKPOINT: u32 = 0x94;
/// The tie-break (+0x624; a reading: the distance along the stage).
pub const TIE_BREAK: u32 = 0x624;

/// `two_car_order`'s result: the places to give the two cars (`None`: left as they are)
/// and the last figure compared (the original's ebx; as it came in when none was).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Order {
    pub places: Option<[u32; 2]>,
    pub last: u32,
}

/// `drive::two_car_order`: -1 for both outside mode 0 or with one view (`players` 1);
/// left as they are while either is finished; else 1 for the one ahead (by checkpoint,
/// then the tie-break, each by the sign of the difference), 2 for the other. `figure(car,
/// offset)` reads a car block's dword, asked as the original reads them.
pub fn two_car_order(
    mode: u32,
    players: u32,
    figure: impl Fn(usize, u32) -> u32,
    ebx: u32,
) -> Order {
    if mode != 0 || players == 1 {
        return Order {
            places: Some([u32::MAX, u32::MAX]),
            last: ebx,
        };
    }
    if figure(0, FINISHED) != 0 || figure(1, FINISHED) != 0 {
        return Order {
            places: None,
            last: ebx,
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

/// `drive::note_gap_ahead` / `note_gap_behind`: the car's checkpoint against the lead
/// car's (ahead: car - lead, behind: lead - car), plus one (the original's edx); it
/// replaces `best` when below it (unsigned).
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
