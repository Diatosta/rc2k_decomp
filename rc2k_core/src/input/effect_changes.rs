//! What a force-feedback effect's function does with a change (from
//! `input::effect_0_change` 0x409708, `effect_1_change` 0x4097FA, `effect_2_change`
//! 0x409754, `effect_3_change` 0x4097B4, `effect_4_change` 0x409871 and `effect_5_change`
//! 0x4098BF).
//!
//! A change is 1 set up, 2 start, 3 values, 4 stop, with two signed values; each of the
//! six effects of a device record answers with stores into its effect and at most one call
//! of DirectInput's (start, set its parameters with flags, stop). A value v is stored as
//! `(v x 10000, as a signed dword) >> 8` (arithmetic).

use crate::Invalid;

/// Rows in effect 1's table (`unk_4F6E08`, four dwords each).
pub const ROWS: i32 = 14;

/// A place in an effect (`ForceEffect`) a change writes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Field {
    /// `DIEFFECT`'s duration.
    Duration,
    /// The first axis.
    FirstAxis,
    /// `DIEFFECT`'s envelope pointer: made to point at the effect's own envelope.
    EnvelopePointer,
    /// The envelope's fade time.
    FadeTime,
    /// A dword of the type-specific parameters.
    Specific(u32),
    /// The found flag (1 when the device listed one of its kind).
    Found,
}

/// DirectInput's call a change ends with.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Call {
    /// Download and Start.
    Start,
    /// SetParameters with these flags.
    Set(u32),
    Stop,
}

/// What a change does: its stores in order, then its call. For effect 1's values the
/// divisor and remainder of its `idiv` (the two operands of the division).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Plan {
    pub stores: Vec<(Field, u32)>,
    pub call: Option<Call>,
    pub division: Option<(u32, u32)>,
}

/// A value as stored.
pub fn value(v: u32) -> u32 {
    ((v as i32).wrapping_mul(10000) >> 8) as u32
}

/// `a` moved toward `b` by `by` / 256 (signed, wrapping).
fn blend(a: u32, b: u32, by: u32) -> u32 {
    ((((b.wrapping_sub(a) as i32).wrapping_mul(by as i32)) >> 8) as u32).wrapping_add(a)
}

/// Effect `number`'s answer to `change` with values `arg_a`, `arg_b`. `sensitivity` is the
/// record's +0x20 (asked only for effect 5's set-up), `row` effect 1's table row (asked
/// only for its values, with a row inside the table). Refuses (before anything) for effect
/// 1's values a row past the table and a blend of 0 (the original's `idiv` traps).
pub fn plan(
    number: u32,
    change: u32,
    arg_a: u32,
    arg_b: u32,
    sensitivity: impl FnOnce() -> u32,
    row: impl FnOnce(u32) -> [u32; 4],
) -> Result<Plan, Invalid> {
    let mut p = Plan::default();
    let s = &mut p.stores;
    match (change, number) {
        (1, 1) => s.push((Field::Specific(0), 0)),
        (1, 2 | 3 | 5) => {
            let long = [0, 0, 500_000, 250_000, 0, 200_000][number as usize];
            s.push((Field::Duration, long));
            if number == 5 {
                s.push((Field::FirstAxis, 4));
            }
            if number != 3 {
                s.push((Field::EnvelopePointer, 0));
                s.push((Field::FadeTime, long));
            }
            if number == 2 {
                s.push((Field::Specific(3), 100_000));
            }
            s.push((Field::Specific(0), 0));
            if number == 5 && sensitivity() > 1 {
                s.push((Field::Found, 0));
            }
        }
        (1, 4) => {
            s.push((Field::Specific(1), 0));
            s.push((Field::Specific(2), 0));
        }
        (2, 0 | 1 | 4) => p.call = Some(Call::Start),
        (3, 0) => {
            s.push((Field::Specific(1), value(arg_a)));
            s.push((Field::Specific(2), value(arg_a)));
            s.push((Field::Specific(0), value(arg_b)));
            p.call = Some(Call::Set(0x100));
        }
        (3, 1) => {
            if !(0..ROWS).contains(&(arg_a as i32)) {
                return Err(Invalid("a row past the effect's table"));
            }
            let r = row(arg_a);
            let first = blend(r[0], r[2], arg_b);
            let second = blend(r[1], r[3], arg_b);
            if second == 0 {
                return Err(Invalid("a blend of 0 (the original's idiv traps)"));
            }
            s.push((Field::Specific(0), value(first)));
            s.push((
                Field::Specific(3),
                10_000_000i32.wrapping_div(second as i32) as u32,
            ));
            p.division = Some((second, 10_000_000i32.wrapping_rem(second as i32) as u32));
            p.call = Some(Call::Set(0x100));
        }
        (3, 2 | 3 | 5) => {
            s.push((Field::Specific(0), value(arg_a)));
            p.call = Some(Call::Set(0x2000_0100));
        }
        (3, 4) => {
            s.push((Field::Specific(1), value(arg_a)));
            s.push((Field::Specific(2), value(arg_a)));
            p.call = Some(Call::Set(0x100));
        }
        (4, _) => p.call = Some(Call::Stop),
        _ => {}
    }
    Ok(p)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn changes() {
        let none = |_| unreachable!();
        let p = plan(0, 3, 0x80, 0xFFFF_FF00, || unreachable!(), none).unwrap();
        assert_eq!(p.call, Some(Call::Set(0x100)));
        assert_eq!(
            p.stores,
            [
                (Field::Specific(1), 5000),
                (Field::Specific(2), 5000),
                (Field::Specific(0), (-10000i32) as u32)
            ]
        );
        let p = plan(5, 1, 0, 0, || 2, none).unwrap();
        assert_eq!(p.stores.first(), Some(&(Field::Duration, 200_000)));
        assert_eq!(p.stores.last(), Some(&(Field::Found, 0)));
        assert_eq!(plan(5, 1, 0, 0, || 1, none).unwrap().stores.len(), 5);
        assert_eq!(plan(3, 1, 0, 0, || 9, none).unwrap().stores.len(), 2);
        assert_eq!(plan(3, 2, 0, 0, || 9, none).unwrap(), Plan::default());
        assert_eq!(
            plan(4, 2, 0, 0, || 9, none).unwrap().call,
            Some(Call::Start)
        );
        assert_eq!(plan(2, 4, 0, 0, || 9, none).unwrap().call, Some(Call::Stop));
        assert_eq!(plan(0, 7, 0, 0, || 9, none).unwrap(), Plan::default());
    }

    #[test]
    fn effect_1_rows() {
        let row = |n: u32| [n * 0x100, 0x200, n * 0x100 + 0x100, 0x400];
        let p = plan(1, 3, 2, 0x80, || 9, row).unwrap();
        // first 0x200 + 0x80, second 0x200 + 0x100.
        assert_eq!(p.stores[0], (Field::Specific(0), value(0x280)));
        assert_eq!(p.stores[1], (Field::Specific(3), 10_000_000 / 0x300));
        assert_eq!(p.division, Some((0x300, 10_000_000 % 0x300)));
        assert!(plan(1, 3, 14, 0, || 9, row).is_err());
        assert!(plan(1, 3, 0xFFFF_FFFF, 0, || 9, row).is_err());
        assert!(plan(1, 3, 0, 0, || 9, |_| [0, 0, 0, 0]).is_err());
    }
}
