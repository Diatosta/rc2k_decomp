use super::*;

#[test]
fn counted_and_started() {
    let mut c = CarStart {
        timer: START,
        ..Default::default()
    };
    assert_eq!(c.count_start_second(), Some(0));
    assert_eq!(c.time, 1);
    c.finished = 1;
    assert_eq!(c.count_start_second(), Some(0));
    assert_eq!(c.time, 1);
    c.timer = 0;
    c.finished = 0;
    assert_eq!(c.count_start_second(), Some(START.wrapping_neg()));
    assert_eq!(c.time, 1);
    c.state = 3;
    assert_eq!(c.count_start_second(), None);
    c.mark_started();
    assert_eq!(c.started, 0);
    c.moving = 1;
    c.mark_started();
    assert_eq!(c.started, 1);
}

#[test]
fn count_down() {
    let mut c = CarStart {
        timer: START,
        countdown: 801,
        ..Default::default()
    };
    assert_eq!(
        c.count_down_start(),
        StartCount::Counted {
            hundreds: 8,
            rest: 0,
            beep: true
        }
    );
    assert_eq!(
        c.count_down_start(),
        StartCount::Counted {
            hundreds: 7,
            rest: 99,
            beep: false
        }
    );
    c.countdown = 0;
    assert_eq!(c.count_down_start(), StartCount::Waiting(0));
    c.finished = 1;
    assert_eq!(c.count_down_start(), StartCount::Stopped);
    let mut s = 698;
    assert!(!stuck_tick(1, &mut s));
    assert!(stuck_tick(3, &mut s));
    assert!(!stuck_tick(2, &mut s));
    assert_eq!(s, 0);
}

#[test]
fn jump_start() {
    let mut c = CarStart {
        timer: 5,
        ..Default::default()
    };
    let mut n = Notice::default();
    let d = c.false_start(2, &mut n);
    assert_eq!(
        d,
        FalseStart {
            timer: Some(5),
            put_back: false
        }
    );
    assert_eq!(
        (c.held, c.count, n.id, n.time),
        (1, 499, JUMP_START_NOTICE, u32::MAX)
    );
    c.count = 1;
    let d = c.false_start(2, &mut n);
    assert_eq!(
        d,
        FalseStart {
            timer: None,
            put_back: true
        }
    );
    assert_eq!(
        (c.held, c.timer, c.time, n.id, n.time),
        (0, 0, PENALTY, PUT_BACK_NOTICE, 300)
    );
    let mut late = CarStart {
        timer: START,
        held: 1,
        ..Default::default()
    };
    let d = late.false_start(0, &mut n);
    assert_eq!(
        d,
        FalseStart {
            timer: Some(START),
            put_back: false
        }
    );
    assert_eq!(late.count, 0);
}
