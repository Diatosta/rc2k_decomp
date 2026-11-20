use super::*;

#[test]
fn figures() {
    assert_eq!(split_figure(2), SplitFigure::Place);
    assert_eq!(split_figure(1), SplitFigure::Rank);
    assert_eq!(split_figure(3), SplitFigure::Compare);
    assert_eq!(split_figure(0), SplitFigure::Nothing);
}

#[test]
fn slots() {
    assert_eq!(kept_split_slot(0, 5), Ok(0));
    assert_eq!(kept_split_slot(3, 5), Ok(2));
    assert_eq!(kept_split_slot(9, 5), Ok(5));
    assert_eq!(kept_split_slot(0x8000_0001, 0x7FFF_FFFF), Ok(0));
    assert!(kept_split_slot(0x50, 0x45).is_err());
}

#[test]
fn ranked() {
    assert_eq!(default_rival(3), (15000, 3, 0x5A8));
    assert_eq!(split_scale(5, 2), Ok((2500, 0)));
    assert!(split_scale(5, 0).is_err());
    assert!(scaled_time(u32::MAX, u32::MAX).is_err());
    assert_eq!(scaled_time(12345, 500), Ok((6172, 500)));
    let t = |c: usize| 100 * c as u32;
    assert_eq!(rank(250, t), (4, 300));
    assert_eq!(rank(0, t), (1, 0));
    assert_eq!(rank(100_000, t), (90, 8800));
    assert_eq!(gap(300, 250), (-500i32) as u32);
}

#[test]
fn compared() {
    let kept = [100, 200, 300, 400, 500, 600, 700, 800, 900, 1000];
    let c = compare_split(450, &kept, |c| 0x100 + c as u32);
    assert_eq!(
        c.figures,
        SplitFigures {
            place: 5,
            gap_ahead: (-500i32) as u32,
            gap_behind: (-500i32) as u32,
            name_ahead: 0x103,
            name_behind: 0x104,
        }
    );
    assert_eq!(c.last, (-5500i32) as u32);
    let slow = compare_split(5000, &kept, |_| 7);
    assert_eq!(
        (slow.figures.place, slow.figures.name_behind),
        (u32::MAX, u32::MAX)
    );
    let fast = compare_split(0, &kept, |_| 7);
    assert_eq!((fast.figures.place, fast.figures.name_ahead), (1, u32::MAX));
    assert_eq!(fast.figures.gap_ahead, (-999_999i32) as u32);
}

#[test]
fn tracked() {
    assert!(check_tracked(0x1418, 0x40).is_ok());
    assert!(check_tracked(0x1419, 0).is_err());
    assert!(check_tracked(0, 0x41).is_err());
    assert!(at_split_end(3, 2) && !at_split_end(0, 0));
    assert!(split_ahead(1, 2) && !split_ahead(2, 2));
}

#[test]
fn timers() {
    assert_eq!(timer_run(2), TimerRun::Start);
    assert_eq!(timer_run(3), TimerRun::Race);
    assert_eq!(timer_run(4), TimerRun::None);
    assert_eq!(timer_rate(false, 0.5), 1.0);
    assert_eq!(timer_rate(true, 0.5), 0.5);
}
