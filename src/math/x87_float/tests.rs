use super::*;
use crate::math::x87::{control_word, set_control_word};
use crate::test_support::vectors::{CONTROL_WORDS, Rng};

/// A value as a double: a float's (the edges among them), a whole number, or any bits.
fn value(rng: &mut Rng) -> f64 {
    match rng.below(4) {
        0 => f64::from(f32::from_bits(rng.fl())),
        1 => f64::from(f32::from_bits(rng.moderate())),
        2 => f64::from(rng.small()),
        _ => f64::from_bits(rng.next()),
    }
}

/// `Nearest` is had exactly where `X` rounds to nearest, and there its steps, stores and
/// whole numbers are `X`'s to the bit; `at_rounding!` runs the other words at `Ambient`.
#[test]
fn near_steps_are_x_steps_at_nearest() {
    let _g = crate::test_support::test_lock();
    let saved = control_word();
    let mut rng = Rng::new(87);
    for cw in CONTROL_WORDS.into_iter().chain([0x037F]) {
        set_control_word(cw);
        let nearest = rounds_to_nearest();
        assert_eq!(Nearest::now().is_some(), nearest, "control word {cw:#06X}");
        let took_nearest = at_rounding!(|r| std::any::type_name_of_val(&r).ends_with("Nearest"));
        assert_eq!(took_nearest, nearest, "control word {cw:#06X}");
        let Some(n) = Nearest::now() else { continue };
        for case in 0..200_000 {
            let (a, b) = (value(&mut rng), value(&mut rng));
            let (xa, xb, na, nb) = (X(a), X(b), n.of(a), n.of(b));
            let pairs = [
                ((xa + xb).0, (na + nb).0),
                ((xa - xb).0, (na - nb).0),
                ((xa * xb).0, (na * nb).0),
                ((xa / xb).0, (na / nb).0),
                ((-xa).0, (-na).0),
                (xa.sqrt().0, n.sqrt(na).0),
            ];
            for (k, (x, y)) in pairs.into_iter().enumerate() {
                assert_eq!(
                    x.to_bits(),
                    y.to_bits(),
                    "{cw:#06X} case {case} step {k}: {a:e} {b:e}"
                );
            }
            assert_eq!(st(xa), n.st(na), "{cw:#06X} case {case}: st {a:e}");
            assert_eq!(
                to_int(xa),
                n.to_int(na),
                "{cw:#06X} case {case}: to_int {a:e}"
            );
            assert_eq!(
                compare(xa, xb),
                n.compare(na, nb),
                "{cw:#06X} case {case}: compare"
            );
        }
    }
    set_control_word(saved);
}
