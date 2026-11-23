//! `ByteView::cell_run` and the copies on it (`load_cells`, `store_cells`) against every store:
//! the runs tile each record, each atomic on its own width's boundary, and the copies give
//! the bytes `memory::shared_memory`'s raw copies give. A store with no byte view is checked on what
//! `Cells` alone promises.

use super::*;

/// Fills every run of primitive cells with varied values (each byte a function of its place in
/// the record and `seed`); with `flip`, changes every byte of the runs numbered `0 mod 3`
/// instead and leaves the others.
struct Filler {
    seed: u32,
    flip: bool,
    place: u32,
    runs: usize,
}

impl Filler {
    fn new(seed: u32, flip: bool) -> Self {
        Filler {
            seed,
            flip,
            place: 0,
            runs: 0,
        }
    }
}

impl CellVisitor for Filler {
    fn enter(&mut self, _: &'static str) {}
    fn index(&mut self, _: usize) {}
    fn leave(&mut self) {}
    fn leaf(&mut self, _: &[u8]) {}
    fn run(&mut self, run: CellRun<'_>) {
        let mut bytes = vec![0; run.len()];
        if self.flip {
            run.load(0, &mut bytes);
            if self.runs.is_multiple_of(3) {
                bytes.iter_mut().for_each(|b| *b ^= 0xFF);
            }
        } else {
            for b in &mut bytes {
                *b = self
                    .place
                    .wrapping_mul(2_654_435_761)
                    .wrapping_add(self.seed)
                    .rotate_left(self.seed % 7) as u8;
                self.place += 1;
            }
        }
        run.store(0, &bytes);
        self.runs += 1;
    }
}

/// The bytes of each run of primitive cells of `record`, one entry per run.
fn runs_of<T: Cells>(record: &T) -> Vec<Vec<u8>> {
    struct Runs(Vec<Vec<u8>>);
    impl CellVisitor for Runs {
        fn enter(&mut self, _: &'static str) {}
        fn index(&mut self, _: usize) {}
        fn leave(&mut self) {}
        fn leaf(&mut self, bytes: &[u8]) {
            self.0.push(bytes.to_vec());
        }
    }
    let mut runs = Runs(Vec::new());
    record.visit(&mut runs);
    runs.0
}

/// What `Cells` promises of a record that has no byte view: `visit` reports non-empty runs
/// that hold no more than the record does, `copy_from` gives equal cells, and `merge_changes`
/// takes exactly the cells that changed (here whole runs, every byte of them) and leaves the
/// others as they are.
fn check_cells<T: Cells>(record: &T, name: &str) {
    let runs = runs_of(record);
    assert!(runs.iter().all(|run| !run.is_empty()), "{name}: empty run");
    let total: usize = runs.iter().map(Vec::len).sum();
    assert!(
        total <= size_of::<T>(),
        "{name}: the runs hold more than the record"
    );
    assert_eq!(flatten(record).len(), total, "{name}: flatten and the runs");
    // A zeroed twin filled with varied values, and a copy of it.
    let from = boxed_zeroed_cells::<T>();
    let mut filler = Filler::new(1, false);
    from.visit(&mut filler);
    assert_eq!(filler.runs, runs.len(), "{name}: the runs when filled");
    let copy = boxed_zeroed_cells::<T>();
    copy.copy_from(&from);
    assert_eq!(flatten(&*copy), flatten(&*from), "{name}: copy_from");
    // `now` is `from` with every third run changed; merged onto another filled record it
    // brings those runs and no other.
    let (start, now) = (boxed_zeroed_cells::<T>(), boxed_zeroed_cells::<T>());
    start.copy_from(&from);
    now.copy_from(&from);
    now.visit(&mut Filler::new(0, true));
    let to = boxed_zeroed_cells::<T>();
    to.visit(&mut Filler::new(2, false));
    let (before, changed) = (runs_of(&*to), runs_of(&*now));
    to.merge_changes(&now, &start);
    let expected: Vec<u8> = before
        .into_iter()
        .zip(changed)
        .enumerate()
        .flat_map(|(n, (kept, new))| if n % 3 == 0 { new } else { kept })
        .collect();
    assert_eq!(flatten(&*to), expected, "{name}: merge_changes");
}

macro_rules! cells_stores {
    ($($store:expr,)*) => {
        $( check_cells(&$store, stringify!($store)); )*
    };
}

#[test]
fn every_store_s_cells_tile_it_and_copy_as_raw_copies_do() {
    let menus = crate::test_support::own::menus();
    let _race = crate::test_support::own::race();
    let _lock = crate::test_support::test_lock();
    cells_stores! {
        menus.play.slots.pointers,
    }
}

#[test]
fn a_byte_written_into_a_dword_keeps_its_other_bytes() {
    use crate::memory::session::{Byte, Dword, Word};
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

#[derive(rc2k_derive::Cells, rc2k_derive::ZeroCells)]
#[repr(C)]
struct ZeroRecord {
    count: crate::memory::session::Dword,
    scale: crate::memory::session::PlainFloat,
    flags: [crate::memory::session::Word; 2],
    gap: crate::memory::session::Bytes<4>,
}

#[test]
fn a_zero_record_is_all_zero_and_can_be_built_on() {
    let zero = ZeroRecord::ZERO;
    let mut bytes = [0xAA; size_of::<ZeroRecord>()];
    load_cells(&zero, 0, &mut bytes).unwrap();
    assert_eq!(bytes, [0; size_of::<ZeroRecord>()]);
    let built = ZeroRecord {
        count: crate::memory::session::Dword::new(3),
        ..ZeroCells::ZERO
    };
    load_cells(&built, 0, &mut bytes).unwrap();
    assert_eq!(&bytes[..4], &[3, 0, 0, 0]);
    assert!(bytes[4..].iter().all(|&b| b == 0));
}

/// Varied values in every cell of a record: each byte a function of its place and `seed`.
fn varied<T: Cells>(record: &T, seed: u32) {
    record.visit(&mut Filler::new(seed, false));
}

fn bytes_of<T: Cells>(record: &T) -> Vec<u8> {
    flatten(record)
}

/// `SameShape` between plain and atomic arrays gives what the byte-run copies give.
#[test]
fn same_shape_arrays_match_the_run_copies() {
    use crate::memory::session::{Dword, Float, PlainDword, PlainFloat};
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
    plain: &[crate::memory::session::PlainDword; N],
) -> [crate::memory::session::Dword; N] {
    let out: [crate::memory::session::Dword; N] =
        std::array::from_fn(|_| crate::memory::session::Dword::new(0));
    plain.copy_cells_to(&out);
    out
}

/// The derived `SameShape` of a record over a kind of cells copies and merges as the derived
/// `Cells` copy and merge do.
#[test]
fn same_shape_records_match_the_run_copies() {
    use crate::memory::session::{CellKind, PlainCells, PlainDword, PlainFloat};
    #[derive(rc2k_derive::Cells)]
    struct Shape<K: CellKind> {
        scale: K::Float,
        count: K::Dword,
        place: [K::Dword; 3],
        place_f: [K::Float; 3],
        kind: K::Dword,
    }
    let make = || Shape::<PlainCells> {
        scale: PlainFloat::new(8.0),
        count: PlainDword::new(0),
        place: [const { PlainDword::new(0) }; 3],
        place_f: [const { PlainFloat::new(0.0) }; 3],
        kind: PlainDword::new(1),
    };
    for seed in 1..6 {
        let (from, start, a, b) = (make(), make(), make(), make());
        varied(&from, seed);
        varied(&start, seed ^ 0x55);
        varied(&a, seed + 9);
        varied(&b, seed + 9);
        a.merge_changes(&from, &start);
        from.merge_cells_to(&start, &b);
        assert_eq!(bytes_of(&a), bytes_of(&b));
        let (c, d) = (make(), make());
        c.copy_from(&from);
        from.copy_cells_to(&d);
        assert_eq!(bytes_of(&c), bytes_of(&d));
    }
}

#[test]
fn a_record_without_a_fixed_layout_is_copied_merged_and_visited_by_field() {
    use crate::memory::session::{Byte, Dword, Word};
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
