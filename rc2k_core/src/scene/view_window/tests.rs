use super::*;

#[test]
fn wide_screens_keep_the_vertical_view() {
    assert_eq!(design_width(800, 600), 800);
    assert_eq!(design_width(1280, 1024), 1280);
    assert_eq!(design_width(1920, 1080), 1440);
    assert_eq!(design_width(3840, 1080), 1440);
}

#[test]
fn widening_is_one_up_to_four_by_three() {
    assert_eq!(side_widening(800, 600), 1.0);
    assert_eq!(side_widening(1280, 1024), 1.0);
    assert_eq!(side_widening(0, 0), 1.0);
    assert_eq!(side_widening(1920, 1080), 1920.0 / 1440.0);
    assert_eq!(side_widening(3840, 1080), 3840.0 / 1440.0);
}

#[test]
fn windows() {
    let s = |framed, wide, mode| ViewSettings { framed, wide, mode };
    assert_eq!(
        choose(s(false, true, 2), || [640, 480]),
        ViewWindow {
            size: [640, 480],
            offset: [0, 0],
            scale: 0x100
        }
    );
    assert_eq!(
        choose(s(true, false, 0), || unreachable!()).offset,
        [0x50, 0]
    );
    assert_eq!(
        choose(s(true, false, 2), || unreachable!()).size,
        [800, 600]
    );
    assert_eq!(choose(s(true, true, 1), || unreachable!()).size, [800, 600]);
    assert_eq!(
        choose(s(true, false, 1), || unreachable!()).size,
        [720, 540]
    );
}

#[test]
fn settings() {
    assert_eq!(nearer_share(0), 0x64);
    assert_eq!(nearer_share(5), 0x5F);
    assert_eq!(nearer_share(11), 0x5A);
    assert_eq!(nearer_share(0x8000_0000), 0x5A);
    assert_eq!(nearer_share(0x8000_0064), 0x64);
    let i = WindowInputs {
        reach: 0x100,
        size: [0x320, 0x258],
        offsets: [0x10, 0x20],
        in_race: false,
        screen_width: 0x320,
        scale: (1.0, 0x100, 0.002),
        nearer: 0,
        nearer_factor: 0.5,
        fade: [1.0, 0.5],
        turn: 65536.0,
        clip_factor: 0.9,
        one: 1.0,
    };
    let s = window_settings(&i);
    assert_eq!((s.depth, s.middle), (0x2000, [0x190, 0x12C]));
    assert_eq!(s.edges, [0xFFFF_FFE0, 0x238, 0xFFFF_FFF0, 0x310]);
    assert_eq!((s.divisor, s.product), (50.0, 0x2000_0000));
    assert_eq!(s.farthest, 0x2000_0000 as f32 / 50.0);
    assert_eq!(s.fade_depth, 328);
    assert_eq!(s.less_one, [799.0, 599.0]);
    assert_eq!(s.edge_floats[0], -32.0);
    let s = window_settings(&WindowInputs { in_race: true, ..i });
    assert_eq!(s.edges, [0, 0x258, 0, 0x320]);
}
