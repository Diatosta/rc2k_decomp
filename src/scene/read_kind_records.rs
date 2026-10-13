//! `scene::read_kind_records` (0x439975): reads the four files of the object kinds'
//! records.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! A kind's record is 0x80 bytes at `g_objectMaterials` + kind * 0x80, and the four
//! files fill the four ranges of kinds, each with `files::read_whole_file`:
//! `files\a\objsinfo` kinds 0 .. 0x18F, `files\b\carsinfo` 0x190 .. 0x257,
//! `files\g\frnt\frntinfo` 0x258 .. 0x31F, `files\s\extr\extrinfo` 0x320 ... The
//! original reads them in the order a, g, b, s; a file that is missing leaves its
//! range as it is.
//! One caller, `scene::set_up_object_memory`.
//! Shadow mode reads twice and compares.
//! Self-test: the game's own files, the records put back afterwards.

use crate::errors::Refused;
use crate::files::read_whole_file::{BYTES_READ_RVA, read_whole_file};
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_ptr, hook_asm, rva};
use crate::scene::set_object_material::{MATERIAL_LEN, MATERIALS_RVA};
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("scene::read_kind_records", rva(0x439975), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

/// A file of records: its path, the first kind it holds and how many kinds.
struct KindFile {
    path: usize,
    first_kind: usize,
    kinds: usize,
}

/// In the order they are read.
const FILES: [KindFile; 4] = [
    KindFile {
        path: rva(0x60AE69), // "files\a\objsinfo"
        first_kind: 0,
        kinds: 0x190,
    },
    KindFile {
        path: rva(0x60AEA1), // "files\g\frnt\frntinfo"
        first_kind: 0x258,
        kinds: 0xC8,
    },
    KindFile {
        path: rva(0x60AE7A), // "files\b\carsinfo"
        first_kind: 0x190,
        kinds: 0xC8,
    },
    KindFile {
        path: rva(0x60AE8B), // "files\s\extr\extrinfo"
        first_kind: 0x320,
        kinds: 0x190,
    },
];

fn records(file: &KindFile) -> Region {
    (
        (game_addr(MATERIALS_RVA) + file.first_kind * MATERIAL_LEN) as u32,
        file.kinds * MATERIAL_LEN,
    )
}

// ---- Rust version ----

pub unsafe fn read_kind_records() -> Result<(), Refused> {
    for file in &FILES {
        let (to, room) = records(file);
        unsafe { read_whole_file(game_ptr(file.path), to as *mut u8, room as u32, 0) }?;
    }
    Ok(())
}

// ---- hook ----

fn regions() -> Vec<Region> {
    let mut regions: Vec<Region> = FILES.iter().map(records).collect();
    regions.push(at(BYTES_READ_RVA));
    regions
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { read_kind_records() }?;
    Ok(Left::nothing())
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let regions = regions();
    let saved = capture(&regions);
    for byte in [0xAAu8, 0x00] {
        check_state(
            &mut t,
            &format!("the records filled with {byte:#x} before"),
            call_original,
            Regs::SENTINEL,
            &|| restore(&regions, &vec![byte; saved.len()]),
            &regions,
            &rust,
        );
    }
    restore(&regions, &saved);
    t.finish()
}
