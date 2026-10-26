//! `marks::pick_shapes` (0x4BE8C3): `g_marksShapesA` and `g_marksShapesB` both get
//! |`dword_844610[dword_84460C]`| (the same entry read twice: section 16), with
//! `dword_844604` / `dword_844608` 2, or 0 when the entry is negative.
//!
//! In:  nothing
//! Out: eax the entry made positive, ebp `g_marksShapesB`; the rest kept
//!
//! Callers: 0x4BDF82, 0x4BE09D, 0x4C067D. Refuses (before anything) an index of 0x100 or
//! more (the table's 256 entries; its writer masks it with 0xFF). Self-test: the index,
//! the entry and the flags by lot (negative entries and 0x80000000 among them), compared
//! on the four results and the registers; put back afterwards.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva};
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::{
    Region, at, capture, check_state, dispatch_state, restore, state_entry,
};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("marks::pick_shapes", rva(0x4BE8C3), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const INDEX_RVA: usize = rva(0x84460C); // dword_84460C
const TABLE_RVA: usize = rva(0x844610); // dword_844610: 256 entries
use rc2k_core::marks::steps::SHAPE_ENTRIES as ENTRIES;
/// (the flag, the shape): `dword_844604` / `g_marksShapesA`, `dword_844608` /
/// `g_marksShapesB`.
const PAIRS: [(usize, usize); 2] = [
    (rva(0x844604), rva(0x82FC56)),
    (rva(0x844608), rva(0x82FC7A)),
];

/// The Rust version (see the header): the registers it leaves.
// Converted: `rc2k_core::marks::steps::{shape_entry, pick_shapes}`
pub unsafe fn pick_shapes(entry: &Regs) -> Result<Regs, Refused> {
    use rc2k_core::marks::steps;
    let mut g = *entry;
    unsafe {
        let index = steps::shape_entry(game_read(INDEX_RVA))?;
        for (flag, shape) in PAIRS {
            let picked = steps::pick_shapes(game_read(TABLE_RVA + 4 * index));
            game_write(flag, picked.flag);
            game_write(shape, picked.shapes);
            g.eax = picked.shapes;
            g.ebp = game_addr(shape) as u32;
        }
    }
    Ok(g)
}

pub fn regions() -> Vec<Region> {
    PAIRS.iter().flat_map(|&(f, s)| [at(f), at(s)]).collect()
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { pick_shapes(r) }?))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let regions = regions();
    let mut vars = regions.clone();
    vars.push(at(INDEX_RVA));
    vars.push((game_addr(TABLE_RVA) as u32, 4 * ENTRIES as usize));
    let saved = capture(&vars);
    for (index, value) in [
        (0u32, 5i32),
        (1, -7),
        (0xFF, 0),
        (0x80, i32::MIN),
        (3, i32::MAX),
        (0x10, -1),
    ] {
        check_state(
            &mut t,
            &format!("index {index:#x}, entry {value:#x}"),
            call_original,
            Regs::SENTINEL,
            &|| unsafe {
                game_write(INDEX_RVA, index);
                game_write(TABLE_RVA + 4 * index as usize, value);
                for &(f, s) in &PAIRS {
                    game_write(f, 0xAAAA_AAAAu32);
                    game_write(s, 0xAAAA_AAAAu32);
                }
            },
            &regions,
            &rust,
        );
    }
    // The refusal.
    unsafe { game_write(INDEX_RVA, ENTRIES) };
    let refused = unsafe { pick_shapes(&Regs::SENTINEL) }.is_err();
    t.check(refused, || "index 0x100 not refused".into());
    restore(&vars, &saved);
    t.finish()
}
