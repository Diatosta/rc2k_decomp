//! The arcade race's (run kind 2) two computer opponents kept near the lead car (from
//! `drive::start_cars_behind` 0x43703E, its rules; the moves and calls stay the adapter's): their
//! first starts, the gaps by how many cars are ahead, the wanted gaps eased toward (what the arcade
//! HUD shows as the gaps to the cars ahead and behind), and the push apart of two cars too close.
//! Only two of the arcade race's opponents are simulated at a time (the rest are numbers in
//! `arcade_cars`). Every comparison takes the sign of the difference, as the original.

/// The first two starts: the last start less 2 and less 1, put 80 and 40
/// checkpoints on.
pub fn first_starts(last: u32) -> [(u32, u32); 2] {
    let a = last.wrapping_sub(2);
    [(a, 0x50), (a.wrapping_add(1), 0x28)]
}

/// A value held below `hi` (`cmp; jns`) and, with `lo_zero`, above 0 (`cmp 0; jns`).
pub fn hold_signs(v: u32, lo_zero: bool, hi: u32) -> u32 {
    let mut v = v;
    if lo_zero && (v as i32) < 0 {
        v = 0;
    }
    if (v.wrapping_sub(hi) as i32) >= 0 {
        v = hi;
    }
    v
}

/// The start whose word gives the gaps: the cars ahead less one, both
/// held to 0 ..= the start count.
pub fn gap_start(ahead: u32, count: u32) -> u32 {
    let v = hold_signs(ahead, true, count);
    hold_signs(v.wrapping_sub(1), true, count)
}

/// The gaps from the start's word (its low four bits b): (passed, 0x2C + b; behind, 0x28
/// + b; their sum), `dword_6051D4` .. `DC`.
pub fn gaps(word: u32) -> (u32, u32, u32) {
    let b = word & 0xF;
    let (pass, back) = (0x2C + b, 0x28 + b);
    (pass, back, back.wrapping_add(pass))
}

/// Whether a car `d` checkpoints from the lead is to be moved (`d` at least `pass`).
pub fn passed(d: u32, pass: u32) -> bool {
    (d.wrapping_sub(pass) as i32) >= 0
}

/// Whether a kept car is ahead of the lead: its checkpoint past the lead's, or the same
/// with the tie-break (+0x624, asked only then) past `CarBlocks.cars[0].checkpoint_offset[1]`.
pub fn ahead_of_lead(lead: u32, checkpoint: u32, lead_tie: u32, tie: impl FnOnce() -> u32) -> bool {
    let d = lead.wrapping_sub(checkpoint);
    (d as i32) < 0 || (d == 0 && (lead_tie.wrapping_sub(tie()) as i32) < 0)
}

/// A wanted gap: the sum less the other's, held to `back` ..= `sum`.
pub fn wanted_gap(sum: u32, other: u32, back: u32) -> u32 {
    let mut v = sum.wrapping_sub(other);
    if (v.wrapping_sub(back) as i32) < 0 {
        v = back;
    }
    if (v.wrapping_sub(sum) as i32) >= 0 {
        v = sum;
    }
    v
}

/// The eased gap's scale.
pub const EASE_SCALE: u32 = 0x14A;
/// How far the eased gap may be before it jumps.
pub const EASE_JUMP: u32 = 1000;

/// The first pass: (the distance made positive; the eased gap when it jumps to the
/// wanted one, which the distance then holds).
pub fn ease_jump(wanted: u32, eased: u32) -> (u32, Option<u32>) {
    let target = wanted.wrapping_mul(EASE_SCALE);
    let mut d = target.wrapping_sub(eased);
    if (d as i32) < 0 {
        d = d.wrapping_neg();
    }
    if d > EASE_JUMP {
        (target, Some(target))
    } else {
        (d, None)
    }
}

/// The second pass: (the step, 1/64 of the way by an arithmetic shift; the eased gap
/// after it).
pub fn ease_step(wanted: u32, eased: u32) -> (u32, u32) {
    let step = ((wanted.wrapping_mul(EASE_SCALE).wrapping_sub(eased) as i32) >> 6) as u32;
    (step, eased.wrapping_add(step))
}

/// The push of two cars `gap` checkpoints apart (the first's less the second's): None
/// unless within -8 ..= 7 and not 0; else the first's push (8 or -8 less the gap; the
/// second's is its negation), before the float factor.
pub fn push_apart(gap: u32) -> Option<u32> {
    if (gap.wrapping_sub(8) as i32) >= 0 || (gap.wrapping_add(8) as i32) < 0 || gap == 0 {
        return None;
    }
    let base: u32 = if (gap as i32) > 0 { 8 } else { 0xFFFF_FFF8 };
    Some(base.wrapping_sub(gap))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_and_gaps() {
        assert_eq!(first_starts(10), [(8, 0x50), (9, 0x28)]);
        assert_eq!(gap_start(0, 5), 0);
        assert_eq!(gap_start(3, 5), 2);
        assert_eq!(gap_start(9, 5), 4);
        assert_eq!(gaps(0x13), (0x2F, 0x2B, 0x5A));
        assert!(passed(0x2F, 0x2F));
        assert!(!passed(0x2E, 0x2F));
        assert!(ahead_of_lead(5, 6, 0, || unreachable!()));
        assert!(ahead_of_lead(5, 5, 3, || 4));
        assert!(!ahead_of_lead(5, 5, 4, || 4));
        assert_eq!(wanted_gap(0x5A, 0x40, 0x2B), 0x2B);
        assert_eq!(wanted_gap(0x5A, u32::MAX, 0x2B), 0x5A);
        assert_eq!(wanted_gap(0x5A, 0x20, 0x2B), 0x3A);
    }

    #[test]
    fn easing_and_pushes() {
        assert_eq!(ease_jump(10, 0), (3300, Some(3300)));
        assert_eq!(ease_jump(3, 0), (990, None));
        assert_eq!(ease_step(3, 0), (15, 15));
        assert_eq!(ease_step(0, 64), (u32::MAX, 63));
        assert_eq!(push_apart(0), None);
        assert_eq!(push_apart(8), None);
        assert_eq!(push_apart((-9i32) as u32), None);
        assert_eq!(push_apart(3), Some(5));
        assert_eq!(push_apart((-3i32) as u32), Some((-5i32) as u32));
    }
}
