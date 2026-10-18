//! `race::clear_point_records` (0x4500FF): `dword_5C6FC4` 0, then each of the 0x1000
//! point records (0x20 bytes) at `g_pointRecords`: its first dword -1, or with `g_useSse`
//! set its first 16 bytes `xmmword_62DAE0` (`movntps`; the Rust version writes the same
//! bytes with an ordinary store).
//!
//! In:  nothing
//! Out: ecx 0, edi past the records (callers: a `retn` at 0x4500F9, a `call` in
//!      `sub_4557AB`; no flags read)
//!
//! See `race/frame_draw.rs`. Shadow mode: by state (the records' 0x20000 bytes and the
//! dword). Self-test: a copy of the original whose three variables are the test's (4
//! references), on a 16-aligned test area filled with 0xAA, with and without SSE, against
//! the Rust core on the same.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, call_game, game_addr, game_read, hook_asm, rva};
use crate::race::frame_draw::*;
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, dispatch_state};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::clear_point_records", rva(0x4500FF), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn rust(_: &Regs) -> Result<Left, Refused> {
    Ok(unsafe { clear_point_records() })
}

extern "C" fn dispatch(r: &mut Regs) {
    let records: u32 = unsafe { game_read(POINT_RECORDS_RVA) };
    let regions: Vec<Region> = vec![at(CLEAR_COUNT_RVA), (records, 0x20 * 0x1000)];
    dispatch_state(&HOOK, r, call_original, &regions, &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // Three variables, then the area (16-aligned).
    let mut space = vec![0u8; 0x20 * 0x1000 + 0x40];
    let base = (space.as_mut_ptr() as u32 + 15) & !15;
    let vars = [base, base + 4, base + 8];
    let area = base + 0x10;
    let copy = match crate::audio::test_copy::make(
        &HOOK,
        &crate::audio::test_copy::Copy {
            address: 0x4500FF,
            len: 0x48,
            entry: &[0xC7, 0x05, 0xC4, 0x6F, 0x5C, 0x00, 0x00, 0x00],
            calls: &[],
            jumps_out: &[],
        },
        &|_| None,
        &[
            (0x5C6FC4, vars[0]),
            (0x621A58, vars[1]),
            (0x5C6FB8, vars[2]),
        ],
        4,
    ) {
        Ok(c) => c,
        Err(e) => return t.skip(&format!("copy: {e}")),
    };
    let fill = game_addr(SSE_FILL_RVA) as u32;
    for sse in [0u32, 1] {
        let prepare = || unsafe {
            std::ptr::write_bytes(base as *mut u8, 0xAA, 0x20 * 0x1000 + 0x10);
            (vars[1] as *mut u32).write(sse);
            (vars[2] as *mut u32).write(area);
        };
        let snapshot = || unsafe {
            std::slice::from_raw_parts(base as *const u8, 0x20 * 0x1000 + 0x10).to_vec()
        };
        prepare();
        let mut ro = Regs::SENTINEL;
        unsafe { call_game(&mut ro, copy.addr()) };
        let theirs = snapshot();
        prepare();
        let left = unsafe { clear_point_records_at(vars[0], vars[1], vars[2], fill) };
        let mine = snapshot();
        let regs_ok = left.matches(&Regs::SENTINEL, &ro);
        t.check(theirs == mine && regs_ok, || {
            let diff = theirs.iter().zip(&mine).position(|(a, b)| a != b);
            format!("sse {sse}: memory first differs at {diff:x?}, registers ok {regs_ok}")
        });
    }
    drop(space);
    t.finish()
}
