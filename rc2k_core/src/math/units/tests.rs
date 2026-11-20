use super::*;

#[test]
fn celsius() {
    assert_eq!(celsius_to_fahrenheit(0), 32);
    assert_eq!(celsius_to_fahrenheit(100), 212);
    assert_eq!(celsius_to_fahrenheit(-40), -40);
    // rounds toward 0
    assert_eq!(celsius_to_fahrenheit(-1), 31);
    assert_eq!(celsius_to_fahrenheit(1), 33);
}

#[test]
fn hours_and_minutes() {
    assert_eq!(hhmm_to_minutes(1300), 780);
    assert_eq!(hhmm_to_minutes(59), 59);
    assert_eq!(hhmm_to_minutes(2359), 1439);
}

fn text(f: Figure) -> (String, Option<u16>) {
    (String::from_utf8(f.text).unwrap(), f.label)
}

#[test]
fn figures() {
    use Units::*;
    assert_eq!(
        text(speed(1234, Imperial).unwrap()),
        ("123.4".into(), Some(0x573))
    );
    assert_eq!(
        text(speed(1000, Metric).unwrap()),
        ("160.9".into(), Some(0x574))
    );
    assert_eq!(speed(MAX_MILES + 1, Metric), None);
    assert_eq!(distance_2dp(1205, Imperial).unwrap(), b"12.05");
    assert_eq!(distance_2dp(100, Metric).unwrap(), b"1.61");
    assert_eq!(text(temperature(-5, Metric)), ("-5".into(), Some(0x575)));
    assert_eq!(
        text(temperature(-40, Imperial)),
        ("-40".into(), Some(0x576))
    );
    assert_eq!(text(temperature(20, Imperial)), ("68".into(), Some(0x576)));
    assert_eq!(text(temperature(i32::MIN, Metric)).0, "-2147483648");
    assert_eq!(text(power(3005, Imperial)), ("300".into(), Some(0x557)));
    assert_eq!(text(power(3000, Metric)), ("223".into(), Some(0x558)));
    assert_eq!(
        text(length(0x1_FF_00, Metric)),
        ("".into(), Some(NOT_KNOWN_LABEL))
    );
    assert_eq!(text(length(100, Imperial)), ("0.36\"".into(), None));
    assert_eq!(text(length(1000, Metric)), ("91.4".into(), Some(MM_LABEL)));
    assert_eq!(text(weight(25_000, Metric)), ("1134".into(), Some(0x55E)));
    assert_eq!(text(torque(2000, Imperial)), ("200".into(), Some(0x55C)));
    assert_eq!(text(torque(2000, Metric)), ("27.7".into(), Some(0x55D)));
    assert_eq!(Units::from_setting(2), None);
}

#[test]
fn miles() {
    assert_eq!(miles_to_km(0), Some(0));
    assert_eq!(miles_to_km(1), Some(2));
    assert_eq!(miles_to_km(100), Some(161));
    assert!(miles_to_km(MAX_MILES).is_some());
    assert_eq!(miles_to_km(MAX_MILES + 1), None);
    assert!(miles_to_km(MAX_MILES).unwrap() >= u32::MAX - 1);
}
