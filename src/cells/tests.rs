//! `ByteView::cell_run` and the copies on it (`load_cells`, `store_cells`) against every store:
//! the runs tile each record, each atomic on its own width's boundary, and the copies give
//! the bytes `shared_memory`'s raw copies give.

use super::*;
use crate::audio::owner::audio;
use crate::effects::owner::effects;
use crate::menu::owner::menus;
use crate::race::owner::race;
use crate::scene::owner::scene;
use crate::shared_memory;

/// The runs of `record`, checked to tile it: each starts where the last ended, lies inside,
/// starts on a multiple of its width, and is the run every byte in it names (sampled in a
/// store of megabytes, a debug build).
fn check_tiling<T: ByteView>(record: &T, name: &str) {
    let base = std::ptr::from_ref(record) as usize;
    let size = size_of::<T>();
    let step = (size / 0x4000).max(1);
    let mut at = 0;
    while at < size {
        let (start, run) = record.cell_run(at);
        assert_eq!(start, at, "{name}: a run begins at byte {at:#x}");
        assert!(
            run.len() > 0 && at + run.len() <= size,
            "{name}: the run at {at:#x}"
        );
        assert_eq!(
            run.first(),
            base + start,
            "{name}: the run at {at:#x} lies there"
        );
        assert!(
            (base + at).is_multiple_of(run.width()),
            "{name}: the run at {at:#x} unaligned"
        );
        for inside in (at..at + run.len()).step_by(step) {
            let (s, r) = record.cell_run(inside);
            assert!(
                s <= inside && inside < s + r.len() && r.first() + inside - s == base + inside,
                "{name}: byte {inside:#x} in its run"
            );
        }
        at += run.len();
    }
}

/// The runs the copies are tried on: short and longer lengths, at every offset of a small
/// record and further apart in one of megabytes (a debug build).
fn runs(size: usize) -> impl Iterator<Item = (usize, usize)> {
    let step = (size / 0x4000).max(1);
    (0..size)
        .filter(move |at| at % step == 0)
        .flat_map(move |at| {
            [1, 2, 3, 4, 5, 7, 8, 16, 33]
                .into_iter()
                .map(move |len| (at, len))
                .filter(move |&(at, len)| at + len <= size)
        })
}

fn check_load<T: ByteView>(record: &T, name: &str) {
    let base = std::ptr::from_ref(record).cast::<u8>();
    for (at, len) in runs(size_of::<T>()) {
        let (mut mine, mut raw) = (vec![0; len], vec![0; len]);
        load_cells(record, at, &mut mine).unwrap();
        // SAFETY: inside the record.
        unsafe { shared_memory::load(base.add(at), raw.as_mut_ptr(), len) };
        assert_eq!(mine, raw, "{name}: {len} bytes at {at:#x}");
    }
    let mut past = [0; 2];
    assert!(
        load_cells(record, size_of::<T>() - 1, &mut past).is_err(),
        "{name}: past its end"
    );
}

/// `store_cells` into a zeroed twin of `T` against `shared_memory::store` into another, a
/// pattern written first and the two compared whole after the writes.
fn check_store<T: ByteView>(name: &str) {
    let size = size_of::<T>();
    // SAFETY: any bits make a record of cells (`ByteView`).
    let (mine, raw) = unsafe {
        (
            Box::<T>::new_zeroed().assume_init(),
            Box::<T>::new_zeroed().assume_init(),
        )
    };
    let bytes = |r: &T| {
        let mut out = vec![0; size];
        // SAFETY: inside the record.
        unsafe { shared_memory::load(std::ptr::from_ref(r).cast(), out.as_mut_ptr(), size) };
        out
    };
    let raw_store = |at: usize, new: &[u8]| {
        let to = std::ptr::from_ref(&*raw).cast::<u8>().cast_mut();
        // SAFETY: inside the record.
        unsafe { shared_memory::store(to.add(at), new.as_ptr(), new.len()) };
    };
    let pattern: Vec<u8> = (0..size).map(|n| (n * 7 + 3) as u8).collect();
    store_cells(&*mine, 0, &pattern).unwrap();
    raw_store(0, &pattern);
    assert_eq!(bytes(&mine), pattern, "{name}: the pattern");
    let step = (size / 0x400).max(1);
    for (at, len) in runs(size).filter(|&(at, _)| at % step == 0) {
        let new: Vec<u8> = (0..len).map(|n| (at + n) as u8 ^ 0xA5).collect();
        store_cells(&*mine, at, &new).unwrap();
        raw_store(at, &new);
    }
    assert_eq!(bytes(&mine), bytes(&raw), "{name}: after the writes");
    assert!(
        store_cells(&*mine, size, &[0]).is_err(),
        "{name}: past its end"
    );
}

fn check<T: ByteView>(record: &T, name: &str) {
    check_tiling(record, name);
    check_load(record, name);
    check_store::<T>(name);
}

macro_rules! every_store {
    ($($store:expr,)*) => {
        $( check(&$store, stringify!($store)); )*
    };
}

#[test]
fn every_store_s_cells_tile_it_and_copy_as_raw_copies_do() {
    let audio = audio();
    let scene = scene();
    let effects = effects();
    let menus = menus();
    let race = race();
    let _lock = crate::test_support::test_lock();
    every_store! {
        audio.state,
        audio.sample_records,
        menus.play.players,
        menus.cars.table,
        *race.replay.finish.get(),
        *race.replay.replay_frames(),
        effects.frame.view_vars,
        effects.paths.spline_head,
        effects.setup.work,
        menus.tables.captions,
        menus.tables.lists,
        menus.play.results,
        menus.screens.state,
        *race.physics.camera(),
        *race.shared.car(),
        *race.physics.contact.get(),
        *race.physics.ground_probe.get(),
        *race.physics.ground(),
        *race.physics.pace(),
        *race.physics.split_tables(),
        *race.shared.physics_tables.get(),
        crate::workers::game_scratch().vertices,
        crate::workers::game_scratch().polygon,
        *race.shared.buffers.reads.get(),
        *race.shared.viewer_buffers(),
        *race.shared.text_buffers(),
        *race.physics.car_specs(),
        *race.frame.clock_ticks(),
        *race.course.lists.get(),
        *race.course.course.get(),
        *race.frame.draw_batches(),
        *race.frame.error_list(),
        *race.course.flare_tables(),
        *race.frame.counter(),
        *race.frame.frame_work.get(),
        *race.replay.car_blocks.get(),
        *race.replay.replay_work(),
        *race.frame.lights_and_gear.get(),
        *race.frame.panel_and_overlay.get(),
        race.replay.viewer_and_kinds,
        *race.frame.view_and_records.get(),
        *race.course.kind_records(),
        *race.course.league(),
        *race.course.level_grid.get(),
        *race.course.level_paths(),
        *race.course.lights_data(),
        *race.frame.matrices.get(),
        *race.course.numbers.get(),
        *race.frame.panel_clock(),
        *race.course.stage_prefs.get(),
        *race.shared.replay_file(),
        *race.frame.scratch(),
        *race.frame.shadow_skids.get(),
        *race.frame.sound_spots(),
        *race.shared.switches.get(),
        *race.shared.tables(),
        *race.frame.text_state(),
        *race.shared.vars.get(),
        *race.shared.view.get(),
        *race.replay.replay_view(),
        *race.shared.work.get(),
        *scene.drawing.scene.get(),
        *crate::scene::owner::SCENE.draw_switches.get(),
        *scene.file_numbers(),
        *scene.textures.kinds.get(),
        *scene.lights.state.get(),
        *crate::scene::owner::SCENE.light_tables.get(),
        *scene.lights.work.get(),
        *scene.objects.slots.get(),
        *scene.object_work(),
        *scene.objects.tables.get(),
        *scene.textures.places.get(),
    }
}

#[test]
fn a_byte_written_into_a_dword_keeps_its_other_bytes() {
    use crate::session::{Byte, Dword, Word};
    #[repr(C)]
    #[derive(rc2k_derive::Cells)]
    struct Mixed {
        a: Dword,
        b: Word,
        c: Byte,
        d: Byte,
        e: [Dword; 2],
    }
    let m = Mixed {
        a: Dword::new(0x4433_2211),
        b: Word::new(0x6655),
        c: Byte::new(0x77),
        d: Byte::new(0x88),
        e: [Dword::new(0xCCBB_AA99), Dword::new(0x1122_33DD)],
    };
    // Each scalar its own run; the array one run of dwords.
    let (start, run) = m.cell_run(5);
    assert_eq!((start, run.len(), run.width()), (4, 2, 2));
    let (start, run) = m.cell_run(13);
    assert_eq!((start, run.len(), run.width()), (8, 8, 4));
    store_cells(&m, 1, &[0xEE]).unwrap();
    assert_eq!(m.a.get(), 0x4433_EE11);
    store_cells(&m, 3, &[0x01, 0x02, 0x03, 0x04, 0x05, 0x06]).unwrap();
    assert_eq!(
        (m.a.get(), m.b.get(), m.c.get(), m.d.get()),
        (0x0133_EE11, 0x0302, 0x04, 0x05)
    );
    assert_eq!(m.e[0].get(), 0xCCBB_AA06);
    let mut out = [0; 6];
    load_cells(&m, 7, &mut out).unwrap();
    assert_eq!(out, [0x05, 0x06, 0xAA, 0xBB, 0xCC, 0xDD]);
}

#[test]
#[ignore = "a timing, run by hand in release"]
fn timing_of_a_deep_read() {
    let race = race();
    let field = &race.shared.work.get().no_precipitation;
    let n = 2_000_000;
    let t = std::time::Instant::now();
    let mut sum = 0u32;
    for _ in 0..n {
        let mut b = [0; 4];
        load_cells(
            race.shared.work.get(),
            std::mem::offset_of!(crate::race::work_store::RaceWork, no_precipitation),
            &mut b,
        )
        .unwrap();
        sum = sum.wrapping_add(u32::from_le_bytes(b));
    }
    let cells = t.elapsed();
    let t = std::time::Instant::now();
    for _ in 0..n {
        sum = sum.wrapping_add(std::hint::black_box(field).get());
    }
    let typed = t.elapsed();
    eprintln!(
        "per read: load_cells {:?}, typed {:?} ({sum})",
        cells / n,
        typed / n
    );
}

#[derive(rc2k_derive::Cells, rc2k_derive::ZeroCells)]
#[repr(C)]
struct ZeroRecord {
    count: crate::session::Dword,
    scale: crate::session::PlainFloat,
    flags: [crate::session::Word; 2],
    gap: crate::session::Bytes<4>,
}

#[test]
fn a_zero_record_is_all_zero_and_can_be_built_on() {
    let zero = ZeroRecord::ZERO;
    let mut bytes = [0xAA; size_of::<ZeroRecord>()];
    load_cells(&zero, 0, &mut bytes).unwrap();
    assert_eq!(bytes, [0; size_of::<ZeroRecord>()]);
    let built = ZeroRecord {
        count: crate::session::Dword::new(3),
        ..ZeroCells::ZERO
    };
    load_cells(&built, 0, &mut bytes).unwrap();
    assert_eq!(&bytes[..4], &[3, 0, 0, 0]);
    assert!(bytes[4..].iter().all(|&b| b == 0));
}

/// Bytes of varied values for a record: each byte a function of its place and `seed`.
fn varied<T: ByteView>(record: &T, seed: u32) {
    let size = size_of::<T>();
    let bytes: Vec<u8> = (0..size)
        .map(|k| {
            (k as u32)
                .wrapping_mul(2_654_435_761)
                .wrapping_add(seed)
                .rotate_left(seed % 7) as u8
        })
        .collect();
    store_cells(record, 0, &bytes).unwrap();
}

fn bytes_of<T: ByteView>(record: &T) -> Vec<u8> {
    let mut out = vec![0; size_of::<T>()];
    load_cells(record, 0, &mut out).unwrap();
    out
}

/// `SameShape` between plain and atomic arrays gives what the byte-run copies give.
#[test]
fn same_shape_arrays_match_the_run_copies() {
    use crate::session::{Dword, Float, PlainDword, PlainFloat};
    for seed in 1..6 {
        let from: [PlainDword; 9] = std::array::from_fn(|_| PlainDword::new(0));
        let start: [PlainDword; 9] = std::array::from_fn(|_| PlainDword::new(0));
        varied(&from, seed);
        varied(&start, seed ^ (seed << 3)); // some cells equal, some not
        let (old, new): ([Dword; 9], [Dword; 9]) = (
            std::array::from_fn(|_| Dword::new(7)),
            std::array::from_fn(|_| Dword::new(7)),
        );
        copy_whole(&from, &old);
        from.copy_cells_to(&new);
        assert_eq!(bytes_of(&old), bytes_of(&new));
        // merge: onto a plain array
        let (to_old, to_new): ([PlainDword; 9], [PlainDword; 9]) = (
            std::array::from_fn(|_| PlainDword::new(0)),
            std::array::from_fn(|_| PlainDword::new(0)),
        );
        varied(&to_old, seed + 50);
        varied(&to_new, seed + 50);
        let start_atomic = new_start(&start);
        merge_cell_changes(&to_old, &old, &start_atomic);
        old.merge_cells_to(&start_atomic, &to_new);
        assert_eq!(bytes_of(&to_old), bytes_of(&to_new));
        // floats, atomic to plain
        let f: [Float; 4] = std::array::from_fn(|_| Float::new(0.0));
        let pf: [PlainFloat; 4] = std::array::from_fn(|_| PlainFloat::new(0.0));
        let pf2: [PlainFloat; 4] = std::array::from_fn(|_| PlainFloat::new(0.0));
        varied(&f, seed);
        copy_whole(&f, &pf);
        f.copy_cells_to(&pf2);
        assert_eq!(bytes_of(&pf), bytes_of(&pf2));
    }
}

/// The atomic copy of a plain array's values (the merge's "now" and "start" are atomic in the
/// frame views).
fn new_start<const N: usize>(
    plain: &[crate::session::PlainDword; N],
) -> [crate::session::Dword; N] {
    let out: [crate::session::Dword; N] = std::array::from_fn(|_| crate::session::Dword::new(0));
    plain.copy_cells_to(&out);
    out
}

/// The derived `SameShape` of a record over a kind (`ViewVars`) copies and merges as the byte
/// runs do.
#[test]
fn same_shape_records_match_the_run_copies() {
    use crate::race::view_vars_store::ViewVars;
    use crate::session::PlainCells;
    for seed in 1..6 {
        let make = || ViewVars::<PlainCells>::new();
        let (from, start, a, b) = (make(), make(), make(), make());
        varied(&from, seed);
        varied(&start, seed ^ 0x55);
        varied(&a, seed + 9);
        varied(&b, seed + 9);
        merge_cell_changes(&a, &from, &start);
        from.merge_cells_to(&start, &b);
        assert_eq!(bytes_of(&a), bytes_of(&b));
        let (c, d) = (make(), make());
        copy_whole(&from, &c);
        from.copy_cells_to(&d);
        assert_eq!(bytes_of(&c), bytes_of(&d));
    }
}

#[test]
fn a_record_without_a_fixed_layout_is_copied_merged_and_visited_by_field() {
    use crate::session::{Byte, Dword, Word};
    // No `repr(C)`: only `Cells`, no byte view.
    #[derive(rc2k_derive::Cells)]
    struct Free {
        a: Byte,
        b: Dword,
        c: [Word; 2],
        d: [Dword; 2],
    }
    let make = |seed: u32| Free {
        a: Byte::new(seed as u8),
        b: Dword::new(seed * 3),
        c: [Word::new(seed as u16 + 1), Word::new(seed as u16 + 2)],
        d: [Dword::new(seed + 7), Dword::new(seed + 8)],
    };
    struct Leaves(Vec<String>);
    impl CellVisitor for Leaves {
        fn enter(&mut self, name: &'static str) {
            self.0.push(format!("enter {name}"));
        }
        fn index(&mut self, i: usize) {
            self.0.push(format!("index {i}"));
        }
        fn leave(&mut self) {
            self.0.push("leave".into());
        }
        fn leaf(&mut self, bytes: &[u8]) {
            self.0.push(format!("{bytes:02x?}"));
        }
    }
    let mut seen = Leaves(Vec::new());
    make(1).visit(&mut seen);
    // An array of one primitive cell is one leaf, element 0 whole.
    assert_eq!(
        seen.0,
        [
            "enter a",
            "[01]",
            "leave",
            "enter b",
            "[03, 00, 00, 00]",
            "leave",
            "enter c",
            "index 0",
            "[02, 00, 03, 00]",
            "leave",
            "leave",
            "enter d",
            "index 0",
            "[08, 00, 00, 00, 09, 00, 00, 00]",
            "leave",
            "leave",
        ]
    );
    let (to, from) = (make(10), make(20));
    to.copy_from(&from);
    assert_eq!(flatten(&to), flatten(&from));
    let (now, start, to) = (make(20), make(20), make(40));
    now.b.set(99);
    to.merge_changes(&now, &start);
    assert_eq!(to.b.get(), 99);
    assert_eq!(to.a.get(), 40);
}
