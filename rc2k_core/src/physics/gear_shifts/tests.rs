use super::*;

#[test]
fn manual() {
    assert_eq!(manual_up(REVERSE, || 5), Some(Shift::Neutral));
    assert_eq!(manual_up(0, || 5), Some(Shift::Start));
    assert_eq!(manual_up(5, || 5), Some(Shift::Done));
    assert_eq!(manual_up(3, || 5), None);
    assert_eq!(manual_down(1), Some(Shift::Neutral));
    assert_eq!(manual_down(0), Some(Shift::Reverse));
    assert_eq!(manual_down(REVERSE), Some(Shift::Done));
    assert_eq!(manual_down(4), None);
    assert_eq!(start_line_keys(0, true, || true), Some(Shift::Reverse));
    assert_eq!(start_line_keys(3, true, || false), Some(Shift::Neutral));
    assert_eq!(
        start_line_keys(REVERSE, true, || true),
        Some(Shift::Neutral)
    );
    assert_eq!(start_line_keys(REVERSE, true, || false), None);
    assert_eq!(start_line_keys(0, false, || true), Some(Shift::Start));
    assert_eq!(start_line_keys(3, false, || true), None);
    assert_eq!(manual_keys(true, || true), Shift::Up);
    assert_eq!(manual_keys(false, || true), Shift::Down);
    assert!(forward(1) && !forward(0) && !forward(REVERSE));
    assert!(above_first(2) && !above_first(1));
}

#[test]
fn automatic() {
    assert_eq!(auto_up(5, 5), Some(Shift::Done));
    assert_eq!(auto_up(REVERSE, 5), Some(Shift::Start));
    assert_eq!(auto_up(2, 5), None);
    let never = || -> bool { panic!() };
    let no = || -> Result<bool, ()> { Ok(false) };
    assert_eq!(
        auto_tests::<()>(2, || false, never, || true, never, || panic!(), never),
        Ok(Shift::Up)
    );
    assert_eq!(
        auto_tests::<()>(REVERSE, never, || true, never, || true, || panic!(), never),
        Ok(Shift::Start)
    );
    assert_eq!(
        auto_tests(1, || true, || true, never, never, no, || true),
        Ok(Shift::Reverse)
    );
    assert_eq!(
        auto_tests(0, never, never, never, never, no, never),
        Ok(Shift::Done)
    );
    assert_eq!(change(2, 2), Change::Same);
    assert_eq!(change(0, 2), Change::FromNeutral);
    assert_eq!(change(2, 0), Change::ToNeutral);
    assert_eq!(change(2, 3), Change::Through);
    assert_eq!(pick_start::<()>(|g| Ok(g == 3)), Ok(Some(3)));
    assert_eq!(pick_start::<()>(|_| Ok(false)), Ok(None));
}
