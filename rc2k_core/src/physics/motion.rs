//! The work car's step of motion in floats (converted from `physics::sum_forces`
//! 0x43BBCB, `move_car` 0x43CCB0, `spin_wheels` 0x43CD1A, `air_drag` 0x43F2DA,
//! `hold_spin_on_ground` 0x43BAA4, `blend_wheel_load` 0x440F70,
//! `lowest_body_point` 0x45D6DB, `count_down_timers` 0x43D4B5,
//! `count_stop_timers` 0x436978, `store_motion_length` 0x43BD2D; `update_car_angles`'
//! switch 0x43BEDC). What most of
//! the figures are is a reading (the hooks' headers say which).
//!
//! Floats: what the x87 keeps on its stack is an `f64`, rounded to `f32` where the
//! original stores (`physics::x87`): the same bits at the game's double precision.

use super::x87::{abs, below, equal, fistp64, loaded, negated};
use crate::Invalid;

/// `fld t0; fadd t1; ..; fdiv d0; fdiv d1; ..; fstp`: the terms' sum over the divisors.
pub fn sum_over(terms: &[f32], divisors: &[f32]) -> f32 {
    let mut acc = terms[0] as f64;
    for &t in &terms[1..] {
        acc += t as f64;
    }
    for &d in divisors {
        acc /= d as f64;
    }
    acc as f32
}

/// `physics::store_motion_length`: the motion's length (`physics::vector_length`, then
/// `fstp`: rounded to a float).
pub fn motion_length(motion: [f32; 3]) -> f32 {
    crate::math::vector::length(motion) as f32
}

/// `physics::sum_forces`: the step's five sums (`force_sums` ..) of the thirteen forces
/// (`drive_slip`, `614254`, `614258` + `61425C`, the four at 0x614834 + 0x20 k, the five
/// at 0x614774 + 0x20 k) over gravity, the first three also over the step.
pub fn sum_forces(f: &[f32; 13], gravity: f32, step: f32) -> [f32; 5] {
    [
        sum_over(&f[0..1], &[gravity, step]),
        sum_over(&f[1..2], &[gravity, step]),
        sum_over(&f[2..4], &[gravity, step]),
        sum_over(&f[4..8], &[gravity]),
        sum_over(&f[8..13], &[gravity]),
    ]
}

/// `physics::move_car`: the place (whole numbers) moved by the motion times `scale`
/// (65536) and the step, x, z, then y; each move is `fistp`ed (through `scratch`):
/// the last (y's).
pub fn move_place(motion: [f32; 3], place: &mut [u32; 3], scale: f32, step: f32) -> i32 {
    let mut last = 0;
    for k in [0, 2, 1] {
        last = fistp64(motion[k] as f64 * scale as f64 * step as f64);
        place[k] = place[k].wrapping_add(last as u32);
    }
    last
}

/// `physics::spin_wheels`: the two driven wheels' turns on by their drive speeds over the
/// wheel's size, times the step, 65536 and `scale` (`fistp`ed through `scratch`):
/// the second's step.
pub fn spin(
    speeds: [f32; 2],
    turns: &mut [u32; 2],
    wheel: f32,
    step: f32,
    k: f32,
    scale: f32,
) -> i32 {
    let mut last = 0;
    for (speed, turn) in speeds.iter().zip(turns.iter_mut()) {
        last = fistp64(*speed as f64 / wheel as f64 * step as f64 * k as f64 * scale as f64);
        *turn = turn.wrapping_add(last as u32);
    }
    last
}

/// `physics::air_drag`: for each of the motion's three, its square times `k` over `d`,
/// made against the motion (its size when the motion is below `zero` or no number, else
/// minus its size).
pub fn air_drag(motion: [f32; 3], k: f32, d: f32, zero: f32) -> [f32; 3] {
    motion.map(|m| {
        let size = abs((m as f64 * m as f64 * k as f64 / d as f64) as f32);
        if below(m as f64, zero as f64) {
            size
        } else {
            negated(size)
        }
    })
}

/// `physics::hold_spin_on_ground` with all four wheels down: the spin's x and z held to
/// -`limit` ..= `limit` (the limit negated in place twice for each, as the original); the
/// flag: either now has the limit's size (or no number).
pub fn hold_spin(limit: &mut f32, x: &mut f32, z: &mut f32) -> u32 {
    for s in [&mut *x, &mut *z] {
        if !below(*s as f64, *limit as f64) {
            *s = loaded(*limit);
        }
        *limit = negated(*limit);
        if below(*s as f64, *limit as f64) {
            *s = loaded(*limit);
        }
        *limit = negated(*limit);
    }
    let size = |v: f32| v.abs() as f64;
    u32::from(equal(size(*x), *limit as f64) || equal(size(*z), *limit as f64))
}

/// `physics::blend_wheel_load`'s figures (the constants as read).
pub struct LoadBlend {
    /// `contact_depth`.
    pub figure: f32,
    /// 0x71A1FC, 0x71A200, 0x71A204: their signs pick the slot.
    pub signs: [f32; 3],
    /// `wheel_load_limit`.
    pub threshold: f32,
    /// `sixty_four`.
    pub divisor: f32,
    /// `zero`, `one` (0, 1).
    pub zero: f32,
    pub one: f32,
}

/// `physics::blend_wheel_load`: unless the figure is below the threshold, the slot (4 +
/// 2 + 1 for each sign below zero) moved towards 1 by the figure over the divisor held to
/// 0 .. 1: (the slot, the share kept in `scratch`).
pub fn blend_load(b: &LoadBlend, slots: &mut [f32; 8]) -> Option<(u32, f32)> {
    if below(b.figure as f64, b.threshold as f64) {
        return None;
    }
    let mut slot = 0;
    for (sign, add) in b.signs.iter().zip([4, 2, 1]) {
        if below(*sign as f64, b.zero as f64) {
            slot += add;
        }
    }
    let mut share = (b.figure as f64 / b.divisor as f64) as f32;
    if below(share as f64, b.zero as f64) {
        share = loaded(b.zero);
    }
    if !below(share as f64, b.one as f64) {
        share = loaded(b.one);
    }
    let a = slots[slot as usize] as f64;
    slots[slot as usize] = ((1.0 - a) * share as f64 + a) as f32;
    Some((slot, share))
}

/// `physics::lowest_body_point`: with more than four probe points, the deepest of the
/// body's (points 4 .., their depths through `depth`, from 0x640000 down by the sign of
/// the difference) into `lowest` and its number into `number` (-1 for none): (the
/// last depth looked at, the count). A count above 0x40 (the depths' room) refused.
pub fn lowest_point(
    count: u32,
    depth: impl Fn(u32) -> u32,
    lowest: &mut u32,
    number: &mut u32,
) -> Result<Option<(u32, u32)>, Invalid> {
    if count > 0x40 {
        return Err(Invalid("more than 0x40 probe points (the depths' room)"));
    }
    if count <= 4 {
        return Ok(None);
    }
    *lowest = 0x64_0000;
    *number = u32::MAX;
    let mut eax = 0;
    for n in 4..count {
        eax = depth(n);
        if (eax.wrapping_sub(*lowest) as i32) < 0 {
            *lowest = eax;
            *number = n;
        }
    }
    Ok(Some((eax, count)))
}

/// `physics::lowest_body_point`'s share: the lowest (a whole number) over `scale`.
pub fn lowest_share(lowest: u32, scale: f32) -> f32 {
    ((lowest as i32) as f64 / scale as f64) as f32
}

/// `physics::count_down_timers`' counters (readings, all).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct StepCounters {
    /// `light_hold_steps`, `value_jump_steps`: down by one when not 0.
    pub first: [u32; 2],
    /// `stopped_steps`: the steps standing in neutral in mode 1.
    pub standing: u32,
    /// `tyre_change_state`: on by one unless 0, -1 or 1.
    pub running: u32,
    /// `wall_hit_timer`, `drive_cut_steps`, `hit_cooldown`: down by one when not 0.
    pub later: [u32; 3],
    /// `top_speed`: the highest of `engine_speed_scaled` so far.
    pub highest: f32,
}

/// What `physics::count_down_timers` reads besides its counters.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct StepFacts {
    /// `run_ticks`.
    pub steps: u32,
    /// 0x71A080, 0x71A07C, 0x71A078.
    pub switches: [u32; 3],
    /// `run_kind`.
    pub mode: u32,
    /// `parts_present`.
    pub bits: u32,
    /// `gear`: the gear.
    pub gear: u32,
    /// `engine_speed_scaled`.
    pub figure: f32,
}

/// `physics::count_down_timers`: the leftover value as the original leaves it (`parts_present` & 0x1E
/// when that is looked at).
pub fn count_down(c: &mut StepCounters, f: &StepFacts) -> Option<u32> {
    let down = |v: &mut u32| {
        if *v != 0 {
            *v -= 1;
        }
    };
    c.first.iter_mut().for_each(down);
    let mut eax = None;
    let reset = 'standing: {
        if f.steps < 1000 || f.switches.iter().any(|&s| s != 0) {
            break 'standing Some(true);
        }
        if f.mode != 1 {
            break 'standing None;
        }
        let bits = f.bits & 0x1E;
        eax = Some(bits);
        Some(bits == 0x1E || f.gear != 0)
    };
    match reset {
        Some(true) => c.standing = 0,
        Some(false) => c.standing = c.standing.wrapping_add(1),
        None => {}
    }
    if !matches!(c.running, 0 | 1 | u32::MAX) {
        c.running = c.running.wrapping_add(1);
    }
    c.later[..2].iter_mut().for_each(down);
    if !below(f.figure as f64, c.highest as f64) {
        c.highest = loaded(f.figure);
    }
    down(&mut c.later[2]);
    eax
}

/// `physics::count_stop_timers`: the three counters (`started_stand_timer` .. `finish_flag`) in
/// turn; one that is not 0: at 300 the result (-1 for the first two, 2 for the third,
/// into `stop_state`) and done; else while `speed` (`speed`) is below `one` (or
/// unordered) it goes up, and on reaching 300 (unsigned) the same.
pub fn stop_timers(counters: &mut [u32; 3], speed: f32, one: f32) -> Option<u32> {
    let slow = below(speed as f64, one as f64);
    for (c, result) in counters.iter_mut().zip([u32::MAX, u32::MAX, 2]) {
        if *c == 0 {
            continue;
        }
        if *c == 300 {
            return Some(result);
        }
        if !slow {
            continue;
        }
        *c = c.wrapping_add(1);
        if *c >= 300 {
            return Some(result);
        }
    }
    None
}

/// `physics::update_car_angles` (0x43BEDC): the car's angles are
/// made when `camera_mode_is_2_or_3` is set or `all_wheels_off` is 0.
pub fn angles_wanted(force: u32, hold: u32) -> bool {
    force != 0 || hold == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sums_and_moves() {
        let f = [
            1.0f32, 2.0, 3.0, 4.0, 1.0, 1.0, 1.0, 1.0, 2.0, 2.0, 2.0, 2.0, 2.0,
        ];
        assert_eq!(sum_forces(&f, 2.0, 0.5), [1.0, 2.0, 7.0, 2.0, 5.0]);
        let mut place = [10u32, 20, 30];
        assert_eq!(
            move_place([1.0, -2.0, 0.5], &mut place, 65536.0, 0.25),
            -32768
        );
        assert_eq!(place, [10 + 16384, 20u32.wrapping_sub(32768), 30 + 8192]);
        let mut turns = [0u32, 5];
        assert_eq!(spin([2.0, -1.0], &mut turns, 2.0, 1.0, 4.0, 0.5), -1);
        assert_eq!(turns, [2, 4]);
    }

    #[test]
    fn drag_and_holds() {
        assert_eq!(air_drag([2.0, -2.0, 0.0], 3.0, 4.0, 0.0), [-3.0, 3.0, -0.0]);
        assert!(air_drag([f32::NAN, 1.0, 1.0], 1.0, 1.0, 0.0)[0].is_nan());
        let (mut l, mut x, mut z) = (2.0f32, 3.0f32, -0.5f32);
        assert_eq!(hold_spin(&mut l, &mut x, &mut z), 1);
        assert_eq!((l, x, z), (2.0, 2.0, -0.5));
        let (mut l, mut x, mut z) = (2.0f32, -1.0f32, -5.0f32);
        assert_eq!(hold_spin(&mut l, &mut x, &mut z), 1);
        assert_eq!(z, -2.0);
        let (mut l, mut x, mut z) = (2.0f32, 1.0f32, 0.5f32);
        assert_eq!(hold_spin(&mut l, &mut x, &mut z), 0);
    }

    #[test]
    fn blends_and_marks() {
        let b = LoadBlend {
            figure: 5.0,
            signs: [-1.0, 1.0, -1.0],
            threshold: 1.0,
            divisor: 10.0,
            zero: 0.0,
            one: 1.0,
        };
        let mut slots = [0.0f32; 8];
        assert_eq!(blend_load(&b, &mut slots), Some((5, 0.5)));
        assert_eq!(slots[5], 0.5);
        let held = LoadBlend { figure: 50.0, ..b };
        assert_eq!(blend_load(&held, &mut slots), Some((5, 1.0)));
        assert_eq!(slots[5], 1.0);
        assert_eq!(
            blend_load(
                &LoadBlend {
                    figure: f32::NAN,
                    ..b
                },
                &mut slots
            ),
            None
        );
    }

    #[test]
    fn lowest() {
        let depths = [0u32, 0, 0, 0, 5, 3, 0xFFFF_FFF0, 7];
        let (mut l, mut n) = (0, 0);
        assert_eq!(
            lowest_point(8, |k| depths[k as usize], &mut l, &mut n),
            Ok(Some((7, 8)))
        );
        assert_eq!((l, n), (0xFFFF_FFF0, 6));
        assert_eq!(lowest_point(4, |_| 0, &mut l, &mut n), Ok(None));
        assert!(lowest_point(0x41, |_| 0, &mut l, &mut n).is_err());
        assert_eq!(lowest_share(0x8_0000, 65536.0), 8.0);
    }

    #[test]
    fn stops() {
        let mut c = [0u32, 299, 5];
        assert_eq!(stop_timers(&mut c, 0.5, 1.0), Some(u32::MAX));
        assert_eq!(c, [0, 300, 5]);
        let mut c = [0u32, 0, 300];
        assert_eq!(stop_timers(&mut c, 2.0, 1.0), Some(2));
        let mut c = [3u32, 0, 7];
        assert_eq!(stop_timers(&mut c, 2.0, 1.0), None);
        assert_eq!(c, [3, 0, 7]);
        assert_eq!(stop_timers(&mut c, f32::NAN, 1.0), None);
        assert_eq!(c, [4, 0, 8]);
    }

    #[test]
    fn counters() {
        let mut c = StepCounters {
            first: [1, 0],
            standing: 4,
            running: 2,
            later: [0, 3, 1],
            highest: 6.0,
        };
        let f = StepFacts {
            steps: 2000,
            mode: 1,
            bits: 0x10,
            figure: 7.5,
            ..Default::default()
        };
        assert_eq!(count_down(&mut c, &f), Some(0x10));
        assert_eq!(c.first, [0, 0]);
        assert_eq!(
            (c.standing, c.running, c.later, c.highest),
            (5, 3, [0, 2, 0], 7.5)
        );
        assert_eq!(count_down(&mut c, &StepFacts { steps: 10, ..f }), None);
        assert_eq!(c.standing, 0);
        assert_eq!(count_down(&mut c, &StepFacts { mode: 2, ..f }), None);
        assert_eq!(c.standing, 0);
    }
}
