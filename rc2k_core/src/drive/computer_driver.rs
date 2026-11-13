//! The computer's driving of a car (from `drive::drive_computer_car`
//! 0x462811; its x87 steps, the speed's root, the aim's normal and the steering's cross
//! product, stay with the caller): how far along the line it looks by its speed, the bends it
//! notes, the speed it wants and the pedals for it, the point it aims at, the steering's
//! sign, and the reverse when it is stuck. Speeds are whole numbers (twice the root of
//! the motion's x² + z²); what each value means is a reading at best.

/// The step count of the race from which the computer drives.
pub const FIRST_STEP: u32 = 0x320;

/// The index into a checkpoint table (`g_checkpointFaces`) for n + k,
/// held to the count less one (unsigned), as the original holds it.
pub fn held_index(n: u32, k: u32, count: u32) -> u32 {
    let i = (n.wrapping_add(k).wrapping_mul(4)) >> 2;
    if i >= count { count.wrapping_sub(1) } else { i }
}

/// Whether the road is narrow: the face's bytes +0xA and +0xB together below 0x12 (by the
/// sign).
pub fn narrow(a: u32, b: u32) -> bool {
    (a.wrapping_add(b).wrapping_sub(0x12) as i32) < 0
}

/// The checkpoints looked at for the turn ahead: (first, second, the bend's
/// checkpoint's offset), by the slow-driver switch (`slow`), the level and the speed.
pub fn lookahead(slow: bool, level: u32, speed: u32) -> (u32, u32, u32) {
    if slow {
        if level >= 3 {
            if speed >= 0x46 {
                (0xF, 0x10, 0xF)
            } else {
                (3, 4, 3)
            }
        } else if speed >= 0x5A {
            (0xE, 0xF, 0xE)
        } else {
            (4, 5, 4)
        }
    } else if speed >= 0x6E {
        (0xF, 0x10, 0xF)
    } else {
        (4, 5, 4)
    }
}

/// The turn between two of the line's headings: the difference >> 16
/// (arithmetic), made positive.
pub fn turn(from: u32, to: u32) -> u32 {
    (to.wrapping_sub(from) as i32 >> 16).unsigned_abs()
}

/// The turn from which a bend is noted: 0x780 slow, else 0x980; a sharp one 0xC00 (each
/// by the sign of the difference).
pub fn bend_noted(turn: u32, slow: bool) -> bool {
    let limit = if slow { 0x780 } else { 0x980 };
    (turn.wrapping_sub(limit) as i32) >= 0
}

/// Whether a bend is sharp (the car's slot then keeps it so).
pub fn sharp(turn: u32) -> bool {
    (turn.wrapping_sub(0xC00) as i32) >= 0
}

/// Whether the bend replaces the noted one (kept per slot): not below it, by the
/// sign.
pub fn bigger_bend(turn: u32, noted: u32) -> bool {
    (turn.wrapping_sub(noted) as i32) >= 0
}

/// Whether the noted bend (its checkpoint, kept per slot) is passed: the
/// checkpoint not below it, by the sign.
pub fn bend_passed(checkpoint: u32, bend: u32) -> bool {
    (checkpoint.wrapping_sub(bend) as i32) >= 0
}

/// The speed table's entry from the turn over four checkpoints: >> 25 (arithmetic),
/// made positive, + 10, at most 0x18.
pub fn table_entry(from: u32, to: u32) -> u32 {
    let e = ((to.wrapping_sub(from) as i32) >> 25).unsigned_abs() + 10;
    e.min(0x18)
}

/// An entry of 0x10 or more: a tight stretch.
pub fn tight(entry: u32) -> bool {
    entry >= 0x10
}

/// Which of the three speed tables is used: the third with the level 3 or
/// more, else the second slow, else the first.
pub fn speed_table(slow: bool, level: u32) -> usize {
    if level >= 3 {
        2
    } else if slow {
        1
    } else {
        0
    }
}

/// The speed wanted: near a noted bend 25 (sharp), else 50 below a speed of 40 and 40
/// above; else the table's.
pub fn wanted_speed(table: u32, near_bend: bool, sharp: bool, speed: u32) -> u32 {
    if !near_bend {
        table
    } else if sharp {
        0x19
    } else if (speed.wrapping_sub(0x28) as i32) < 0 {
        0x32
    } else {
        0x28
    }
}

/// Whether the wanted speed is eased (the car's wanted speed moves toward it by a step): 50 or
/// more.
pub fn eased(want: u32) -> bool {
    want >= 0x32
}

/// The pedals against the wanted speed: (throttle, brake, hard braking), None where not set.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct Pedals {
    pub throttle: Option<u32>,
    pub brake: Option<u32>,
    pub hard: bool,
}

/// Below the wanted speed full throttle; 5 or more over it the brake, 15 or more hard.
pub fn pedals(speed: u32, want: u32) -> Pedals {
    if speed < want {
        Pedals {
            throttle: Some(0x100),
            ..Pedals::default()
        }
    } else if speed >= want.wrapping_add(5) {
        Pedals {
            throttle: None,
            brake: Some(0xFF),
            hard: speed >= want.wrapping_add(0xF),
        }
    } else {
        Pedals::default()
    }
}

/// The entry of the side-offset table (0x20 of them) for the checkpoint and slot.
pub fn offset_entry(checkpoint: u32, slot: u32) -> u32 {
    checkpoint.wrapping_add(slot) & 0x1F
}

/// The checkpoint aimed at, ahead of the car's: near a sharp bend 5, 4 or 2 by the speed
/// (0x46, 0x28); else 2, 4 in a tight stretch at 0x46 or more, and then 5 from 0x5A, 3
/// from 0x3C.
pub fn aim_ahead(sharp: bool, tight: bool, speed: u32) -> u32 {
    if sharp {
        return if speed >= 0x46 {
            5
        } else if speed >= 0x28 {
            4
        } else {
            2
        };
    }
    let mut k = 2;
    if tight && speed >= 0x46 {
        k = 4;
    }
    if speed >= 0x5A {
        k = 5;
    } else if speed >= 0x3C {
        k = 3;
    }
    k
}

/// The steering: the cross product's y x 256 as a whole number made
/// positive, negated when the product was below 0 (`negative`) and again in reverse.
pub fn steering(whole: u32, negative: bool, reverse: bool) -> u32 {
    let mut s = if (whole as i32) < 0 {
        whole.wrapping_neg()
    } else {
        whole
    };
    if negative {
        s = s.wrapping_neg();
    }
    if reverse {
        s = s.wrapping_neg();
    }
    s
}

/// The reverse count's step (kept per slot; gear -1 is the reverse): (the count left,
/// reversing (and the brake 0x100), the handbrake (gear below 1 with no count)).
pub fn reverse_step(count: u32, gear: i32) -> (Option<u32>, bool, bool) {
    if count != 0 {
        (Some(count - 1), gear != -1, false)
    } else {
        (None, false, gear.wrapping_sub(1) < 0)
    }
}

/// Whether the steps since the last stuck check (kept per slot) make another due.
pub fn check_due(steps: u32) -> bool {
    (steps.wrapping_sub(0xC8) as i32) >= 0
}

/// How far the car moved since the last check (x, z now and then): each part made
/// positive, the larger halved (by the sign of their difference), their sum.
pub fn moved(now: (u32, u32), then: (u32, u32)) -> u32 {
    let size = |v: u32| if (v as i32) < 0 { v.wrapping_neg() } else { v };
    let mut dx = size(now.0.wrapping_sub(then.0));
    let mut dz = size(now.1.wrapping_sub(then.1));
    if (dx.wrapping_sub(dz) as i32) < 0 {
        dz >>= 1;
    } else {
        dx >>= 1;
    }
    dx.wrapping_add(dz)
}

/// Whether the car is stuck: moved less than 0x8000 (by the sign).
pub fn stuck(moved: u32) -> bool {
    (moved.wrapping_sub(0x8000) as i32) < 0
}

/// Stuck while not in reverse: reversing for this many steps.
pub const REVERSE_STEPS: u32 = 0xFA;

/// Stuck in reverse: the reverse tries (kept per slot) one up; at 4 back to 0 and
/// the fourth setting on: (the tries, the setting).
pub fn reverse_tries(tries: u32) -> (u32, bool) {
    let t = tries.wrapping_add(1);
    if t >= 4 { (0, true) } else { (t, false) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn looking() {
        assert_eq!(held_index(5, 2, 10), 7);
        assert_eq!(held_index(9, 2, 10), 9);
        assert_eq!(held_index(0x4000_0000, 0, 10), 0);
        assert!(narrow(8, 9) && !narrow(9, 9));
        assert_eq!(lookahead(true, 3, 0x46), (0xF, 0x10, 0xF));
        assert_eq!(lookahead(true, 0, 0x59), (4, 5, 4));
        assert_eq!(lookahead(false, 0, 0x6E), (0xF, 0x10, 0xF));
        assert_eq!(turn(0, 0xFFFF_0000), 1);
        assert_eq!(turn(0, 0x0780_0000), 0x780);
        assert!(bend_noted(0x780, true) && !bend_noted(0x780, false));
        assert!(sharp(0xC00) && !sharp(0xBFF));
        assert_eq!(table_entry(0, 0x7000_0000), 0x18);
        assert_eq!(table_entry(0, 0x0400_0000), 12);
        assert_eq!(speed_table(true, 3), 2);
        assert_eq!(speed_table(true, 0), 1);
    }

    #[test]
    fn driving() {
        assert_eq!(wanted_speed(7, false, true, 0), 7);
        assert_eq!(wanted_speed(7, true, true, 0), 0x19);
        assert_eq!(wanted_speed(7, true, false, 0x27), 0x32);
        assert_eq!(wanted_speed(7, true, false, 0x28), 0x28);
        assert_eq!(pedals(10, 11).throttle, Some(0x100));
        assert_eq!(pedals(12, 11), Pedals::default());
        assert_eq!(
            pedals(26, 11),
            Pedals {
                throttle: None,
                brake: Some(0xFF),
                hard: true
            }
        );
        assert_eq!(aim_ahead(true, false, 0x30), 4);
        assert_eq!(aim_ahead(false, true, 0x46), 3);
        assert_eq!(aim_ahead(false, true, 0x3B), 2);
        assert_eq!(aim_ahead(false, false, 0x5A), 5);
        assert_eq!(steering((-5i32) as u32, false, false), 5);
        assert_eq!(steering(5, true, true), 5);
        assert_eq!(steering(5, true, false), (-5i32) as u32);
    }

    #[test]
    fn stuck_cars() {
        assert_eq!(reverse_step(3, 1), (Some(2), true, false));
        assert_eq!(reverse_step(3, -1), (Some(2), false, false));
        assert_eq!(reverse_step(0, 0), (None, false, true));
        assert_eq!(reverse_step(0, 1), (None, false, false));
        assert!(check_due(0xC8) && !check_due(0xC7));
        assert_eq!(moved((10, 0), (0, 4)), 9);
        assert_eq!(moved((0, 10), (4, 0)), 9);
        assert!(stuck(0x7FFF) && !stuck(0x8000));
        assert_eq!(reverse_tries(2), (3, false));
        assert_eq!(reverse_tries(3), (0, true));
    }
}
