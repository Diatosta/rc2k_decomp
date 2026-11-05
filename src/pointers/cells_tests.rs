//! `Cells::cell_at` and the copies on it (`load_cells`, `store_cells`) against every store:
//! the cells tile each record, each atomic on its own width's boundary, and the copies give
//! the bytes `shared_memory`'s raw copies give.

use super::*;
use crate::shared_memory;

/// The cells of `record`, checked to tile it: each holds the bytes it names, and an atomic
/// starts on a multiple of its width.
fn check_tiling<T: Cells>(record: &T, name: &str) {
    let base = std::ptr::from_ref(record) as usize;
    let mut at = 0;
    while at < size_of::<T>() {
        let cell = T::cell_at(at);
        assert_eq!(cell.start, at, "{name}: a cell begins at byte {at:#x}");
        assert!(
            cell.len > 0 && at + cell.len <= size_of::<T>(),
            "{name}: {cell:?}"
        );
        match cell.kind {
            CellKind::U8 => assert_eq!(cell.len, 1),
            CellKind::U16 => assert_eq!(cell.len, 2),
            CellKind::U32 => assert_eq!(cell.len, 4),
            CellKind::Bytes => {}
        }
        if cell.kind != CellKind::Bytes {
            assert!(
                (base + at).is_multiple_of(cell.len),
                "{name}: {cell:?} unaligned"
            );
        }
        for inside in at..at + cell.len {
            assert_eq!(T::cell_at(inside), cell, "{name}: byte {inside:#x}");
            assert_eq!(
                cell_of::<T>(inside),
                cell,
                "{name}: byte {inside:#x}, by its runs"
            );
        }
        at += cell.len;
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
    let _lock = crate::store::test_lock();
    every_store! {
        crate::drive::finish_store::FINISH,
        crate::drive::replay_frames_store::REPLAY_FRAMES,
        crate::effects::frame_state::VIEW_VARS,
        crate::effects::splines::SPLINE_HEAD,
        crate::effects::work_store::MARKS_WORK,
        crate::menu::captions_store::CAPTIONS,
        crate::menu::draw_lists_store::DRAW_LISTS,
        crate::menu::lists_store::MENU_LISTS,
        crate::menu::results_store::RESULT_FIGURES,
        crate::menu::state_store::MENU_STATE,
        crate::physics::camera_store::CAMERA_WORK,
        crate::physics::car_store::CAR,
        crate::physics::contact_store::CONTACT_WORK,
        crate::physics::ground_probe_store::GROUND_PROBE,
        crate::physics::ground_store::GROUND_WORK,
        crate::physics::pace_store::PACE,
        crate::physics::split_tables_store::SPLIT_TABLES,
        crate::physics::tables_store::PHYSICS,
        crate::race::buffers_store::BUFFERS,
        crate::race::car_specs_store::CAR_SPECS,
        crate::race::clock_ticks_store::CLOCK_TICKS,
        crate::race::course_lists_store::COURSE_LISTS,
        crate::race::course_store::COURSE,
        crate::race::draw_batches_store::DRAW_BATCHES,
        crate::race::error_list_store::ERROR_LIST,
        crate::race::flares_store::FLARE_TABLES,
        crate::race::frame_counter_store::FRAME_COUNTER,
        crate::race::frame_store::FRAME_WORK,
        crate::race::groups_store::CARS_AND_REPLAY,
        crate::race::groups_store::LIGHTS_AND_GEAR,
        crate::race::groups_store::PANEL_AND_OVERLAY,
        crate::race::groups_store::VIEWER_AND_KINDS,
        crate::race::groups_store::VIEW_AND_RECORDS,
        crate::race::kind_records_store::KIND_RECORDS,
        crate::race::league_store::LEAGUE,
        crate::race::level_grid_store::LEVEL_GRID,
        crate::race::level_paths_store::LEVEL_PATHS,
        crate::race::lights_store::LIGHTS_DATA,
        crate::race::matrices_store::MATRICES,
        crate::race::numbers_store::NUMBERS,
        crate::race::panel_clock_store::PANEL_CLOCK,
        crate::race::prefs_store::STAGE_PREFS,
        crate::race::replay_file_store::REPLAY_FILE,
        crate::race::scratch_store::FRAME_SCRATCH,
        crate::race::shadow_skids_store::SHADOW_SKIDS,
        crate::race::sound_spots_store::RACE_SOUND_SPOTS,
        crate::race::switches_store::FRAME_SWITCHES,
        crate::race::tables_store::RACE_TABLES,
        crate::race::text_state_store::TEXT_STATE,
        crate::race::vars_store::RACE_VARS,
        crate::race::view_store::RACE_VIEW,
        crate::race::viewer_state_store::REPLAY_VIEW,
        crate::race::work_store::RACE_WORK,
        crate::scene::draw_store::DRAW,
        crate::scene::draw_switches_store::DRAW_SWITCHES,
        crate::scene::file_numbers_store::FILE_NUMBERS,
        crate::scene::kind_texture_store::KIND_TEXTURES,
        crate::scene::light_store::LIGHT_STATE,
        crate::scene::light_tables_store::LIGHT_TABLES,
        crate::scene::light_work_store::LIGHT_WORK,
        crate::scene::object_slots_store::OBJECT_SLOTS,
        crate::scene::object_store::OBJECT_WORK,
        crate::scene::object_tables_store::OBJECT_TABLES,
        crate::scene::texture_places_store::TEXTURE_PLACES,
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
    assert_eq!(
        Mixed::cell_at(5),
        CellAt {
            start: 4,
            len: 2,
            kind: CellKind::U16,
            run_end: 6,
        }
    );
    assert_eq!(
        Mixed::cell_at(11),
        CellAt {
            start: 8,
            len: 4,
            kind: CellKind::U32,
            run_end: 16,
        }
    );
    assert_eq!(
        Mixed::cell_at(13),
        CellAt {
            start: 12,
            len: 4,
            kind: CellKind::U32,
            run_end: 16,
        }
    );
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
    use crate::race::work_store::RACE_WORK;
    let field = &RACE_WORK.no_precipitation;
    let at = GamePtr::to(field);
    let n = 2_000_000;
    let t = std::time::Instant::now();
    let mut sum = 0u32;
    for _ in 0..n {
        let mut b = [0; 4];
        load_cells(
            &RACE_WORK,
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
