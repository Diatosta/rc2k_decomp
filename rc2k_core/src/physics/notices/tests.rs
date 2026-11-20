use super::*;

#[test]
fn value_and_count_down() {
    let mut n = Notice::default();
    n.value_notice(300);
    assert_eq!((n.id, n.time), (VALUE_NOTICE, 0));
    let mut m = Notice {
        shown: 1,
        last: VALUE_NOTICE,
        ..Default::default()
    };
    m.value_notice(299);
    assert_eq!((m.shown, m.last), (0, 0));
    let mut big = Notice::default();
    big.value_notice(0x8000_0000);
    // A negative figure is below 300 (the difference's sign counted 0x80000000 ..
    // 0x8000012B as past it).
    assert_eq!(big.id, 0);
    let mut edge = Notice::default();
    edge.value_notice(0x8000_012B);
    assert_eq!(edge.id, 0);
    edge.value_notice(0x7FFF_FFFF);
    assert_eq!(edge.id, VALUE_NOTICE);
    let mut c = Notice {
        shown: 1,
        time: 2,
        ..Default::default()
    };
    assert_eq!(c.count_down(), Some(2));
    assert_eq!(c.count_down(), Some(1));
    assert_eq!((c.shown, c.time), (0, 0));
    assert_eq!(c.count_down(), None);
    let mut z = Notice {
        shown: 1,
        ..Default::default()
    };
    assert_eq!(z.count_down(), Some(0));
    assert_eq!(z.shown, 1);
}

#[test]
fn queue_and_take() {
    let mut q = NoticeQueue::default();
    assert_eq!(q.queue(0, 1, 1, 1), Ok(None));
    assert_eq!(q.queue(4, 0xCF, 1, 7), Ok(Some(0)));
    assert_eq!(q.queue(4, 0xD0, 3, 7), Ok(Some(0)));
    assert_eq!((q.ids[0], q.levels[0], q.count), (0xD0, 3, 1));
    assert_eq!(q.queue(4, 0xD1, 2, 7), Ok(Some(0)));
    assert_eq!(q.ids[0], 0xD0);
    assert_eq!(q.queue(4, 0xD2, 0, 7), Ok(Some(1)));
    let mut n = Notice::default();
    assert_eq!(n.take(&mut q), Some(0xD0));
    assert_eq!(
        (n.id, n.time, n.w708, q.count, q.ids[0]),
        (0xD0, 0x12C, 0x32, 1, 0xD2)
    );
    assert_eq!(n.take(&mut q), None);
    q.count = QUEUE_ROOM;
    assert_eq!(q.queue(4, 1, 1, 1), Ok(None));
    q.count = QUEUE_ROOM + 1;
    assert!(q.queue(4, 1, 1, 1).is_err());
}
