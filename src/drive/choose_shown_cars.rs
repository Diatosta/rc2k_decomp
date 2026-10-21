//! `drive::choose_shown_cars` (0x4560EA): which car blocks the two views show
//! (`off_63DBB0`, `off_63DBB4` = 0x70F3EC + n 0xFE0 for `dword_63DBA8`, `dword_63DBAC`),
//! `dword_63DB98` (the views' number, from `dword_63DB94`, by `dword_63DB88` when
//! `dword_5C5CE0` is 2, also setting `dword_6238A4`), the replay's car when
//! `dword_52A910` is set (`dword_63DBA0`, then the second view 0), and the eight of
//! `dword_641680` .. -1 but the shown ones' 0 (the first, and the second when there are
//! two views). By the look of it the split screen's set-up (a reading).
//!
//! In:  nothing
//! Out: eax `dword_63DBA0` (with `dword_52A910` set) or the second view's block; the
//!      rest kept; no flags read (callers go on with `mov`, `lea`, calls)
//!
//! Callers (5): race init's loop (`sub_4608BB`), `sub_460835`, `sub_460B2F`,
//! `sub_43A649`, `sub_45429B`.
//! Refuses (before anything) a replay car (`dword_63DBA0`) above 7 with `dword_52A910`
//! set (it would write past `dword_641680`'s eight).
//! Self-test: `physics/copy_hooks_test.rs` (a copy on test variables), 40 cases over the
//! switches; the refusal.

use crate::errors::Refused;
use crate::framework::{EAX, RawHook, Regs, SelfTest, game_addr, hook_asm, rva};
use crate::physics::copy_hooks_test::{lot, test_moves};
use crate::screens::harness::Left;
use crate::service::harness::dispatch_state;
use rc2k_core::drive::shown_cars::{ShownCars, ShownSetup};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("drive::choose_shown_cars", rva(0x4560EA), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The variables (IDA addresses), in the order the core uses them.
const VARIABLES: [usize; 19] = [
    0x63DBA8, 0x63DBAC, 0x63DB94, 0x63DB98, 0x5C5CE0, 0x63DB88, 0x6238A4, 0x52A910, 0x63DBA0,
    0x63DBB0, 0x63DBB4, 0x641680, 0x641684, 0x641688, 0x64168C, 0x641690, 0x641694, 0x641698,
    0x64169C,
];
const FIRST_BLOCK: usize = 0x70F3EC; // byte_70F3EC: car block 0's start (an address)

unsafe fn get(v: &[u32], k: usize) -> u32 {
    unsafe { (v[k] as *const u32).read_unaligned() }
}

unsafe fn put(v: &[u32], k: usize, x: u32) {
    unsafe { (v[k] as *mut u32).write_unaligned(x) }
}

/// The Rust version on the variables at `v` (`VARIABLES`' order).
// Converted: `rc2k_core::drive::shown_cars::ShownCars::choose`.
pub unsafe fn choose_at(v: &[u32]) -> Result<Left, Refused> {
    unsafe {
        let setup = ShownSetup {
            players: get(v, 2),
            screen_mode: get(v, 4),
            layout: get(v, 5),
            replay: get(v, 7),
            replay_car: get(v, 8),
        };
        let split = get(v, 6);
        let mut shown = ShownCars {
            split,
            ..ShownCars::default()
        };
        let eax = shown.choose(&setup, game_addr(rva(FIRST_BLOCK)) as u32)?;
        put(v, 0, shown.cars[0]);
        put(v, 1, shown.cars[1]);
        put(v, 3, shown.views);
        if shown.split != split {
            put(v, 6, shown.split);
        }
        put(v, 9, shown.blocks[0]);
        put(v, 10, shown.blocks[1]);
        for (k, &x) in shown.listed.iter().enumerate() {
            put(v, 11 + k, x);
        }
        Ok(Left::nothing().with(EAX, eax))
    }
}

fn game() -> Vec<u32> {
    VARIABLES
        .iter()
        .map(|&x| game_addr(rva(x)) as u32)
        .collect()
}

/// The Rust version.
pub unsafe fn choose_shown_cars() -> Result<Left, Refused> {
    unsafe { choose_at(&game()) }
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { choose_shown_cars() }
}

extern "C" fn dispatch(r: &mut Regs) {
    let v = game();
    let regions: Vec<_> = [0usize, 1, 3, 6, 9, 10]
        .iter()
        .map(|&k| (v[k], 4))
        .chain([(v[11], 0x20)])
        .collect();
    dispatch_state(&HOOK, r, call_original, &regions, &rust);
}

fn fill(seed: u32, out: &mut [u32]) {
    lot(seed, out);
    out[2] = [1, 2, 0, 3][(seed % 4) as usize];
    out[4] = if seed % 5 < 3 { 2 } else { seed };
    out[5] = seed / 5 % 5;
    out[7] = u32::from(seed.is_multiple_of(3));
    out[8] = seed % 8;
}

fn self_test() -> (usize, usize) {
    let mut result = test_moves(
        &HOOK,
        0x4560EA,
        0x1C8,
        &[0xC7, 0x05, 0xA8, 0xDB, 0x63, 0x00, 0x00, 0x00, 0x00, 0x00],
        &VARIABLES,
        47,
        40,
        &fill,
        &|v, _| unsafe { choose_at(v) }.unwrap_or_else(|_| Left::nothing()),
    );
    // The refusal, on cells of the test's.
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return result;
    };
    let mut cells = [0u32; 19];
    cells[7] = 1;
    cells[8] = 8;
    let at: Vec<u32> = (0..19).map(|k| &cells[k] as *const u32 as u32).collect();
    let refused = unsafe { choose_at(&at) }.is_err();
    t.check(refused, || "replay car 8 not refused".into());
    let (c, f) = t.finish();
    result.0 += c;
    result.1 += f;
    result
}
