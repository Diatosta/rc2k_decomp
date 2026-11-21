use super::*;

#[test]
fn walks() {
    let faces = [(0x400, None), (0x400, Some(0x800)), (0, None)];
    let w = walk::<()>(3, |k| {
        let (now_word, before_word) = faces[k as usize];
        Ok(ProbeFaces {
            now_word,
            before_word,
        })
    });
    assert_eq!(
        w,
        Ok(Walk::Met {
            probe: 1,
            words: 0x400
        })
    );
    let w = walk::<()>(2, |_| {
        Ok(ProbeFaces {
            now_word: 0xC01,
            before_word: Some(0),
        })
    });
    assert_eq!(
        w,
        Ok(Walk::Met {
            probe: 0,
            words: 0xC01
        })
    );
    let w = walk::<()>(2, |k| {
        Ok(ProbeFaces {
            now_word: [0x401, 0x802][k as usize],
            before_word: None,
        })
    });
    assert_eq!(w, Ok(Walk::Clear { bits: 0xC00 }));
    assert!(check_probe_count(0).is_err() && check_probe_count(0x41).is_err());
    assert!(check_probe(0x3F).is_ok() && check_probe(0x40).is_err());
    assert_eq!(raised_hit(-3.0, 2.0), Some(3.0));
    assert_eq!(raised_hit(2.0, 2.0), Some(2.0));
    assert_eq!(raised_hit(1.0, 2.0), None);
    assert_eq!(raised_hit(f32::NAN, 2.0), None);
    assert_eq!(raised_hit(1.0, f32::NAN), None);
}

#[test]
fn edges() {
    let wall = |j: usize| [1, 2, 3, 4][j];
    assert_eq!(shared_edge(wall, [9, 9, 9, 9]), (false, 2, 4));
    // (p3, p1) = (3, 1) is the other face's (p2, p1).
    assert_eq!(shared_edge(wall, [1, 3, 8, 9]), (true, 3, 1));
    assert_eq!(path_side((0, 0), (10, 0), (3, 2)), (0, 20));
    let mut asked = Vec::new();
    let e = crossed_edge(|j| {
        asked.push(j);
        j == 2
    });
    assert_eq!(e, (0xC, 4));
    assert_eq!(asked, [0, 1, 3, 2, 0]);
    assert_eq!(crossed_edge(|j| j == 0), (4, 8));
    assert_eq!(crossed_edge(|j| j == 1), (8, 0x10));
}

#[test]
fn wheels() {
    let w = WheelsOff::of([0, 0, 1, 0]);
    assert!(w.front && !w.back && !w.all);
    assert!(w.driven(0) && !w.driven(1));
    let mut c = ProbeCounts::default();
    c.tried();
    c.searched(true, 5, 5);
    c.searched(false, 5, 5);
    assert_eq!(
        c,
        ProbeCounts {
            tries: 1,
            found: 1,
            same: 1
        }
    );
}
