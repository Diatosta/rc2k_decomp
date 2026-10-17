//! `physics::apply_torques` (0x43F6BF): ebx force records of 0x20 bytes from edi, each
//! by `physics::apply_torque` on the work car (`g_car`), inside `pusha` / `popa`.
//!
//! In:  edi the first record, ebx their number
//! Out: every register kept; no flags read
//!
//! Callers: `physics::apply_wheel_torques`, `physics::apply_contact_torques`.
//! Refuses (before anything) ebx 0 (the original would loop 2^32 times) or above 0x100.
//! Self-test: a copy of the original (its car the test's, its call to a copy of
//! `apply_torque`) on test records, counts 1 .. 7.

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, call_game, game_addr, hook_asm, rva};
use crate::physics::apply_force::BLOCK_LEN;
use crate::physics::apply_torque::{apply_torque, apply_torque_at, fill};
use crate::screens::harness::Left;
use crate::service::harness::{Region, capture, dispatch_state};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("physics::apply_torques", rva(0x43F6BF), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const CAR_RVA: usize = rva(0x719FD8); // g_car

/// Refused before anything?
pub fn refusal(count: u32) -> Option<&'static str> {
    (count == 0 || count > 0x100).then_some("a torque record count of 0 or above 0x100")
}

/// The Rust version: `count` records from `first` on the work car.
pub unsafe fn apply_torques(first: u32, count: u32) -> Result<(), Refused> {
    if let Some(why) = refusal(count) {
        return Err(BadInput(why).into());
    }
    let car = game_addr(CAR_RVA) as u32;
    for k in 0..count {
        unsafe { apply_torque(first + 0x20 * k, car) };
    }
    Ok(())
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { apply_torques(r.edi, r.ebx) }?;
    Ok(Left::nothing())
}

/// What `apply_torque` writes on the work car.
pub fn regions() -> Vec<Region> {
    crate::physics::apply_torque::regions(game_addr(CAR_RVA) as u32)
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

/// Copies for a test: this one (its `g_car` at `car`, its call to a copy of
/// `apply_torque` with its scratch vectors at `work`, `cross`).
pub fn test_copies(
    car: u32,
    work: u32,
    cross: u32,
) -> Result<(usize, Vec<crate::redirect::RedirectedCopy>), String> {
    let inner = crate::physics::apply_torque::test_copy(work, cross)?;
    let target = inner.addr();
    let own = crate::audio::test_copy::make(
        &HOOK,
        &crate::audio::test_copy::Copy {
            address: 0x43F6BF,
            len: 0x16,
            entry: &[0x60, 0x8D, 0x37, 0x8D, 0x3D, 0xD8, 0x9F, 0x71, 0x00],
            calls: &[9],
            jumps_out: &[],
        },
        &|r| (r == rva(0x45FB82)).then_some(target),
        &[(0x719FD8, car)],
        1,
    )?;
    Ok((own.addr(), vec![own, inner]))
}

/// Test cells: a car block, 0x17 records, the two scratch vectors.
pub struct Cells {
    pub cells: Vec<u32>,
    pub car: u32,
    pub recs: u32,
    pub work: u32,
    pub cross: u32,
}

impl Default for Cells {
    fn default() -> Cells {
        Cells::new()
    }
}

impl Cells {
    pub fn new() -> Cells {
        let mut cells = vec![0u32; (BLOCK_LEN as usize) / 4 + 8 * 0x17 + 6];
        let car = cells.as_mut_ptr() as u32;
        let recs = car + BLOCK_LEN;
        let work = recs + 0x20 * 0x17;
        Cells {
            cells,
            car,
            recs,
            work,
            cross: work + 12,
        }
    }

    /// The car and `count` records filled by `seed`.
    pub fn fill(&self, count: u32, seed: u32) {
        for k in 0..count {
            fill(self.recs + 0x20 * k, self.car, seed + k);
        }
    }

    /// The Rust core on `count` records.
    pub fn run_rust(&self, count: u32) {
        for k in 0..count {
            unsafe { apply_torque_at(self.recs + 0x20 * k, self.car, self.work, self.cross) };
        }
    }

    pub fn all(&self) -> [Region; 1] {
        [(self.car, 4 * self.cells.len())]
    }
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let c = Cells::new();
    let (copy, keep) = match test_copies(c.car, c.work, c.cross) {
        Ok(x) => x,
        Err(why) => return t.skip(&why),
    };
    for count in 1..=7u32 {
        c.fill(count, count * 3);
        let before = Regs {
            edi: c.recs,
            ebx: count,
            ..Regs::SENTINEL
        };
        let mut ro = before;
        unsafe { call_game(&mut ro, copy) };
        let theirs = capture(&c.all());
        c.fill(count, count * 3);
        c.run_rust(count);
        let mine = capture(&c.all());
        let regs_ok = ro.preserved_except(&before, 0);
        t.check(regs_ok && theirs == mine, || {
            format!("count {count}: regs_ok={regs_ok}; the memory differs")
        });
    }
    t.check(refusal(0).is_some(), || "count 0 not refused".into());
    drop((keep, c));
    t.finish()
}
