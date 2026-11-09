//! `Cells::cell_run` and the copies on it (`load_cells`, `store_cells`) against every store:
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
fn check_tiling<T: Cells>(record: &T, name: &str) {
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

fn check_load<T: Cells>(record: &T, name: &str) {
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
fn check_store<T: Cells>(name: &str) {
    let size = size_of::<T>();
    // SAFETY: any bits make a record of cells (`Cells`).
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

fn check<T: Cells>(record: &T, name: &str) {
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
    let _lock = crate::store::test_lock();
    every_store! {
        audio.state,
        audio.sample_records,
        menus.play.players,
        menus.cars.table,
        race.replay.finish,
        race.replay.replay_frames,
        effects.frame.view_vars,
        effects.paths.spline_head,
        effects.setup.work,
        menus.tables.captions,
        menus.tables.draw_lists,
        menus.tables.lists,
        menus.play.results,
        menus.screens.state,
        *race.physics.camera,
        race.shared.car,
        race.physics.contact,
        race.physics.ground_probe,
        *race.physics.ground,
        race.physics.pace,
        race.physics.split_tables,
        *race.shared.physics_tables,
        race.shared.buffers,
        race.physics.car_specs,
        race.frame.clock_ticks,
        *race.course.lists,
        *race.course.course,
        race.frame.draw_batches,
        race.frame.error_list,
        race.course.flare_tables,
        race.frame.counter,
        *race.frame.frame_work,
        race.replay.cars_and_replay,
        race.frame.lights_and_gear,
        race.frame.panel_and_overlay,
        race.replay.viewer_and_kinds,
        *race.frame.view_and_records,
        race.course.kind_records,
        race.course.league,
        *race.course.level_grid,
        race.course.level_paths,
        race.course.lights_data,
        *race.frame.matrices,
        race.course.numbers,
        race.frame.panel_clock,
        *race.course.stage_prefs,
        race.shared.replay_file,
        *race.frame.scratch,
        *race.frame.shadow_skids,
        *race.frame.sound_spots,
        *race.shared.switches,
        race.shared.tables,
        race.frame.text_state,
        *race.shared.vars,
        *race.shared.view,
        race.replay.replay_view,
        *race.shared.work,
        *scene.drawing.scene,
        *crate::scene::owner::SCENE.draw_switches,
        crate::scene::owner::SCENE.file_numbers,
        scene.textures.kinds,
        *scene.lights.state,
        crate::scene::owner::SCENE.light_tables,
        *scene.lights.work,
        scene.objects.slots,
        *scene.drawing.object_work,
        *scene.objects.tables,
        scene.textures.places,
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
    let field = &race.shared.work.no_precipitation;
    let at = GamePtr::to(field);
    let n = 2_000_000;
    let t = std::time::Instant::now();
    let mut sum = 0u32;
    for _ in 0..n {
        let mut b = [0; 4];
        load_cells(
            &*race.shared.work,
            std::mem::offset_of!(crate::race::work_store::RaceWork, no_precipitation),
            &mut b,
        )
        .unwrap();
        sum = sum.wrapping_add(u32::from_le_bytes(b));
    }
    let cells = t.elapsed();
    let t = std::time::Instant::now();
    for _ in 0..n {
        sum = sum.wrapping_add(at.read::<u32>("timing").unwrap());
    }
    let ptr = t.elapsed();
    let t = std::time::Instant::now();
    for _ in 0..n {
        sum = sum.wrapping_add(std::hint::black_box(field).get());
    }
    let typed = t.elapsed();
    eprintln!(
        "per read: load_cells {:?}, GamePtr::read {:?}, typed {:?} ({sum})",
        cells / n,
        ptr / n,
        typed / n
    );
}

#[test]
#[ignore = "a timing, run by hand in release"]
fn timing_of_a_raw_read() {
    let block = Box::into_raw(Box::new([7u32; 64]));
    let start = map(block, 256);
    register(start, 256);
    let at = GamePtr::new(start + 60);
    let n = 2_000_000;
    let t = std::time::Instant::now();
    let mut sum = 0u32;
    for _ in 0..n {
        sum = sum.wrapping_add(std::hint::black_box(at).read::<u32>("timing").unwrap());
    }
    eprintln!("raw GamePtr::read {:?} ({sum})", t.elapsed() / n);
}
