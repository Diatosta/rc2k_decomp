//! A network race's messages between the cars (converted 2026-10-21 from `net::send_turn`
//! 0x46415B, `net::send_inputs` 0x464194, `net::send_position` 0x464201,
//! `net::send_extra` 0x4643FA, `net::apply_message` 0x4644EC and `net::follow_remote`
//! 0x464883). Each step of the own car sends, by its step count, its inputs ('JGSM'),
//! its place and motion ('PGSM') or nine more dwords ('XGSM'): a tag and dwords of its car
//! block; the others' are put into their cars' blocks, a place received far from where the
//! car is taken at once, a near one glided to over 32 steps.

/// A race message's kind.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    /// 'JGSM': the four inputs.
    Inputs,
    /// 'PGSM': the place, the matrix and 25 more dwords.
    Position,
    /// 'XGSM': nine dwords.
    Extra,
}

/// The gear in a car's block (-1 reverse, 0 neutral, 1 .. 7).
const GEAR: u32 = 0x1D4;

/// The car block's dwords each kind carries, in order.
const INPUTS: [u32; 4] = [0xFD0, 0xFD4, 0xFD8, 0xFDC];
const POSITION: [u32; 40] = [
    0x134, 0x138, 0x13C, 0xB4, 0xB8, 0xBC, 0xC0, 0xC4, 0xC8, 0xCC, 0xD0, 0xD4, 0xD8, 0xDC, 0xE0,
    0x198, 0x19C, 0x1A0, 0x1A4, 0x1A8, 0x1AC, 0x184, 0x188, 0x18C, 0x98, 0x94, 0x1D4, 0x1D8, 0x1B8,
    0x1BC, 0x600, 0x604, 0x608, 0x60C, 0x638, 0x63C, 0x640, 0x644, 0x648, 0x1EC,
];
const EXTRA: [u32; 9] = [0x64, 0x68, 0x6C, 0x70, 0x74, 0x78, 0x7C, 0x80, 0x84];

/// A car's place (the block's +0x134 .. +0x13C) and where a place received goes, to be
/// glided to (+0xFC0 .. +0xFC8; then the glide's 32nd), and the glide's steps left
/// (+0xFCC).
pub const PLACE: u32 = 0x134;
pub const WANTED: u32 = 0xFC0;
pub const GLIDE_STEPS: u32 = 0xFCC;
/// The block's step count (+8).
pub const STEPS: u32 = 8;

impl Kind {
    pub fn tag(self) -> u32 {
        match self {
            Kind::Inputs => 0x4D53_474A,
            Kind::Position => 0x4D53_4750,
            Kind::Extra => 0x4D53_4758,
        }
    }

    /// The kind of a message's first dword, if it is one.
    pub fn of(tag: u32) -> Option<Kind> {
        [Kind::Position, Kind::Inputs, Kind::Extra]
            .into_iter()
            .find(|k| k.tag() == tag)
    }

    /// Whether a car could hold what a message of this kind carries (`values`: its dwords
    /// after the tag, in `fields`' order). Another machine's message is not to be believed
    /// by a game that refuses such values and so ends: one that fails is not applied. The
    /// original took any, and read its tables with them.
    pub fn believable(self, values: &[u32]) -> bool {
        let within = |value: u32, most: f32| f32::from_bits(value).abs() <= most; // NaN: false
        self.fields()
            .iter()
            .zip(values)
            .all(|(&field, &value)| match field {
                GEAR => (-1..=7).contains(&(value as i32)),
                // the gauges, the orientation, the springs: about 1 at most
                0x64..=0x80 | 0xB4..=0xE0 | 0x600..=0x60C => within(value, 2.0),
                // spin, velocities, driven wheels, revs
                0x184..=0x18C | 0x198..=0x1AC | 0x1B8 | 0x1BC | 0x1D8 => within(value, 1e7),
                // the pedals, the steering
                0x638..=0x648 => (value as i32).unsigned_abs() <= 0x4000_0000,
                _ => true,
            })
    }

    /// The car block's dwords it carries, in order.
    pub fn fields(self) -> &'static [u32] {
        match self {
            Kind::Inputs => &INPUTS,
            Kind::Position => &POSITION,
            Kind::Extra => &EXTRA,
        }
    }

    /// Where the `n`th dword received goes in the block: as sent, but the place received
    /// goes where the car is to be.
    pub fn destination(self, n: usize) -> u32 {
        if self == Kind::Position && n < 3 {
            WANTED + 4 * n as u32
        } else {
            self.fields()[n]
        }
    }

    /// The message: the tag and the dwords (`field`, asked of each in order).
    pub fn pack(self, field: impl Fn(u32) -> u32) -> Vec<u32> {
        let mut out = vec![self.tag()];
        out.extend(self.fields().iter().map(|&f| field(f)));
        out
    }

    /// Whether this kind goes out at the own car's step `count` (read afresh for each:
    /// every fourth step the inputs, at & 0x1F = 2 the place, at & 0xFF = 1 the rest),
    /// with the masked count the original leaves in eax.
    pub fn due(self, count: u32) -> (bool, u32) {
        let (mask, value) = match self {
            Kind::Inputs => (3, 0),
            Kind::Position => (0x1F, 2),
            Kind::Extra => (0xFF, 1),
        };
        (count & mask == value, count & mask)
    }
}

/// The kinds in the order a step looks at them.
pub const SENT: [Kind; 3] = [Kind::Inputs, Kind::Position, Kind::Extra];

/// How far a place received may be from where the car is and still be glided to:
/// RAL.EXE's `flt_5C5D54`, 10.0.
pub const NEAR: f32 = 10.0;

/// What a place received does to the car.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Arrival {
    /// The car put there at once.
    Jump,
    /// A 32nd of the way to it a step, 32 steps.
    Glide([u32; 3]),
}

/// A place received (`wanted`) against the car's (`now`): in the car's first 1000 steps
/// (signed), or when it is less than `near` away (`fild; fisub; fmul` for each, the sum,
/// `fsqrt`, `fcomp`; unordered counts as less), glided to; else jumped to. The distance's
/// test is exact in `f64` at every precision the x87 runs: the differences are whole
/// numbers below 2^33 and the squares' sum is exact while it is anywhere near `near`
/// squared, rounding only far above it.
pub fn arrival(wanted: [u32; 3], now: [u32; 3], steps: u32, near: f32) -> Arrival {
    let early = (steps.wrapping_sub(1000) as i32) < 0;
    let d = |k: usize| wanted[k] as i32 as f64 - now[k] as i32 as f64;
    // The original's order: the last two squares' sum, then the first's added.
    let distance = (d(0) * d(0) + (d(1) * d(1) + d(2) * d(2))).sqrt();
    let near = near as f64;
    let closer = distance < near || distance.is_nan() || near.is_nan();
    if early || closer {
        Arrival::Glide(std::array::from_fn(|k| {
            ((wanted[k] as i32).wrapping_sub(now[k] as i32) >> 5) as u32
        }))
    } else {
        Arrival::Jump
    }
}

/// How far off a car's place received is snapped to rather than glided to: ten metres. The
/// original compares with 10.0 (`g_ten`) in the place's own units, 1/65536 of a metre, so
/// past a car's first 1000 steps every message snapped it; moved from outside the physics
/// that way, a remote car crossed face edges and was reset to the line.
pub const SNAP_FROM: f32 = 10.0 * 65536.0;

/// The steps of a glide.
pub const GLIDE: u32 = 0x20;

/// A remote car's glide step (`follow_remote`): with the network race on and steps left,
/// one less and a 32nd of the way added to the place; `None` otherwise.
pub fn glide(
    on: bool,
    steps_left: u32,
    place: [u32; 3],
    step: [u32; 3],
) -> Option<(u32, [u32; 3])> {
    (on && steps_left != 0).then(|| {
        (
            steps_left.wrapping_sub(1),
            std::array::from_fn(|k| place[k].wrapping_add(step[k])),
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_car_glides_to_a_place_metres_off_and_snaps_to_one_far_off() {
        let metre = 65536u32;
        let at = [1000 * metre, 0, 1000 * metre];
        let near = [at[0] + 3 * metre, 0, at[2]];
        assert!(matches!(
            arrival(near, at, 5000, SNAP_FROM),
            Arrival::Glide(_)
        ));
        let far = [at[0] + 11 * metre, 0, at[2]];
        assert!(matches!(arrival(far, at, 5000, SNAP_FROM), Arrival::Jump));
    }

    #[test]
    fn what_no_car_could_hold_is_not_believed() {
        let honest = |kind: Kind| -> Vec<u32> {
            kind.fields()
                .iter()
                .map(|&f| match f {
                    GEAR => 3,
                    0x638..=0x648 => 0x100,
                    _ => 0.5f32.to_bits(),
                })
                .collect()
        };
        for kind in [Kind::Inputs, Kind::Position, Kind::Extra] {
            assert!(kind.believable(&honest(kind)));
        }
        let at = |field: u32| {
            Kind::Position
                .fields()
                .iter()
                .position(|&f| f == field)
                .unwrap()
        };
        for (field, value) in [
            (GEAR, 8),
            (GEAR, (-2i32) as u32),
            (0xB4, f32::NAN.to_bits()),
            (0xB4, 3.0f32.to_bits()),
            (0x198, f32::INFINITY.to_bits()),
            (0x1D8, 1e8f32.to_bits()),
            (0x644, 0x7FFF_FFFF),
        ] {
            let mut values = honest(Kind::Position);
            values[at(field)] = value;
            assert!(!Kind::Position.believable(&values), "{field:#x} {value:#x}");
        }
        assert!(!Kind::Extra.believable(&[f32::NAN.to_bits(); 9]));
    }

    #[test]
    fn kinds() {
        assert_eq!(Kind::of(0x4D53_4750), Some(Kind::Position));
        assert_eq!(Kind::of(0x4D53_4751), None);
        assert_eq!(Kind::Position.fields().len(), 40);
        assert_eq!(Kind::Position.destination(2), 0xFC8);
        assert_eq!(Kind::Position.destination(3), 0xB4);
        assert_eq!(Kind::Inputs.destination(0), 0xFD0);
        assert_eq!(Kind::Extra.pack(|f| f + 1)[..3], [0x4D53_4758, 0x65, 0x69]);
        assert_eq!(Kind::Inputs.due(8), (true, 0));
        assert_eq!(Kind::Position.due(0x22), (true, 2));
        assert_eq!(Kind::Extra.due(0x102), (false, 2));
    }

    #[test]
    fn places() {
        assert_eq!(arrival([100, 0, 0], [0, 0, 0], 2000, NEAR), Arrival::Jump);
        assert_eq!(
            arrival([100, 0, 0], [0, 0, 0], 999, NEAR),
            Arrival::Glide([3, 0, 0])
        );
        assert_eq!(arrival([6, 8, 0], [0, 0, 0], 2000, NEAR), Arrival::Jump);
        assert_eq!(
            arrival([6, 7, 0], [0, 0, 0], 2000, NEAR),
            Arrival::Glide([0, 0, 0])
        );
        assert_eq!(
            arrival([0, 0, 0], [64, 0, 0], 2000, f32::NAN),
            Arrival::Glide([u32::MAX - 1, 0, 0])
        );
        assert_eq!(
            glide(true, 3, [1, 2, 3], [1, u32::MAX, 0]),
            Some((2, [2, 1, 3]))
        );
        assert_eq!(glide(false, 3, [1, 2, 3], [1, 1, 1]), None);
        assert_eq!(glide(true, 0, [1, 2, 3], [1, 1, 1]), None);
    }
}
