use super::*;

#[test]
fn split_points() {
    assert_eq!(near_window(0), (u32::MAX, 1));
    assert!(search_all(0xA0001) && !search_all(0xA0000));
    assert_eq!(check_split_point(0x40, || 5), Ok(5));
    assert!(check_split_point(0x41, || panic!()).is_err());
    assert!(check_split_point(0, || 0x1419).is_err());
    assert!(face_known(3, 4) && !face_known(4, 4));
    assert_eq!(passed_point(0x4000_0000, 0x8000_0000), (true, 0x4000_0000));
    assert!(!passed_point(0x3FFF_0000, 0x8000_0000).0);
}

#[test]
fn go() {
    let mut f = GoFacts {
        mode: 1,
        ..Default::default()
    };
    assert!(f.go_due());
    f.cheats = 1 << 16;
    assert!(!f.go_due());
    f.cheats = 1 << 15;
    f.countdown = 1;
    assert!(!f.go_due());
}

#[test]
fn splits() {
    assert_eq!(split_due(2, 3), Ok(true));
    assert_eq!(split_due(3, 3), Ok(false));
    assert!(split_due(0x40, 0x41).is_err());
    assert!(finished_at(5, 5));
    assert_eq!(split_bonus(30), 3000);
}

#[test]
fn ahead() {
    assert_eq!(checkpoint_ahead(3, 5), Ok(Some(4)));
    assert_eq!(checkpoint_ahead(4, 5), Ok(None));
    assert_eq!(
        checkpoint_ahead(u32::MAX, 5),
        Err(Invalid("a checkpoint below 0 or past 0x1418"))
    );
    assert_eq!(
        checkpoint_ahead(0x1419, 0x2000),
        Err(Invalid("a checkpoint below 0 or past 0x1418"))
    );
}

#[test]
fn warnings() {
    assert_eq!(spread(3, 7), 4);
    assert_eq!(spread(7, 3), 4);
    assert!(off_line(false, || unreachable!()));
    assert!(!off_line(true, || 3));
    assert!(off_line(true, || 4));
    let mut w = Warnings {
        off: 299,
        turned: 5,
        off_time: 300,
        turned_time: 300,
    };
    w.count_off(true);
    w.count_turned(10, 20);
    assert_eq!((w.off, w.turned, w.turned_time), (300, 0, 300));
    assert_eq!(cue_due(w.turned, w.turned_time), None);
    assert_eq!(cue_due(w.off, w.off_time), Some(3300));
    assert_eq!(cue_due(0x7FFF_FFFF, 0x8000_0000), None);
}
