//! `marks::load_edges` (0x4C0F2E): `init_marks`' step 1, second half: the kinds' records
//! scaled once, the kind's edge picture read and packed, the shape tilt, then the masks
//! and the two item kinds.
//!
//! In:  nothing
//! Out: as `marks::set_up_squares` leaves them, the rest as the earlier steps
//!
//! See `marks/edges.rs`. One caller, `sub_4C1235`. Shadow mode runs the original alone (it
//! reads a file and takes memory). Refuses a once-only mark neither 0 nor `loc_4C0FFD`, and
//! a picture that is not read. Self-test: a redirected copy whose five calls go to
//! stand-ins noting every register (the picture a pattern of the test's, settings of the
//! test's), against the Rust
//! version on the same stand-ins, with the mark 0 and set; everything put back afterwards.

use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_write, hook_asm, rva, shadow};
use crate::marks::edges::*;
use crate::marks::masks::{NOTED, note_step, test_step};
use crate::service::harness::{Region, at, capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("marks::load_edges", rva(0x4C0F2E), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    let resume = game_addr(RESUME_RVA) as u32;
    match unsafe { load_edges(r, resume, &mut game_part) } {
        Ok(left) => {
            *r = Regs {
                esp: r.esp,
                eflags: r.eflags,
                ..left
            }
        }
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let resume = game_addr(RESUME_RVA) as u32;
    {
        let left = unsafe { load_edges(r, resume, &mut game_part) }?;
        *r = Regs {
            esp: r.esp,
            eflags: r.eflags,
            ..left
        }
    }
    Ok(())
}

const LEN: usize = 0x205;
/// Where the copy goes on once done (`loc_4C0FFD` in it).
const RESUME_AT: u32 = 0xCF;

fn self_test() -> (usize, usize) {
    use crate::gfx::picture::{HEIGHT_RVA, PICTURE_RVA, ROW_BYTES_RVA, WIDTH_RVA};
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // cmp dword_8BC3D6, 0; jz +7
    let entry: &[u8] = &[0x83, 0x3D, 0xD6, 0xC3, 0x8B, 0x00, 0x00];
    let copy = match crate::race::mud::copy(
        0x4C0F2E,
        LEN,
        entry,
        &[0x16F, 0x1F0, 0x1F5, 0x1FA, 0x1FF],
        &[
            (0x4119F4, 0),
            (0x4BDE9A, 1),
            (0x4BAA55, 2),
            (0x4BF529, 3),
            (0x4C08B9, 4),
        ],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    crate::install::stand_ins::set_handler(note_step);
    let picture: u32 = unsafe { crate::framework::game_read(PICTURE_RVA) };
    // Settings of the test's: the game's are not there before a stage is loaded.
    let scene = crate::marks::lines::TestScene::new();
    let settings = scene.settings.as_ptr() as u32;
    let mut regions: Vec<Region> = edge_regions();
    regions.extend([
        (picture, 4 * EDGE_LEN as usize),
        (settings + 0x2708, 4),
        at(WIDTH_RVA),
        at(HEIGHT_RVA),
        at(ROW_BYTES_RVA),
        at(crate::marks::kind_values::KIND_RVA),
    ]);
    let saved = capture(&regions);
    let regs = |r: &Regs| [r.eax, r.ebx, r.ecx, r.edx, r.esi, r.edi, r.ebp];
    let resume_copy = copy.addr() as u32 + RESUME_AT;
    let resume_game = game_addr(RESUME_RVA) as u32;
    for (n, kind) in [3u32, 0x30, 0xFFFF_FFFF].into_iter().enumerate() {
        for done in [false, true] {
            let prepare = |mark: u32| unsafe {
                restore(&regions, &saved);
                for k in 0..EDGE_LEN {
                    let v = (k.wrapping_mul(0x9E37_79B9) >> 7) & (0x0101_0101 * (n as u32 + 1));
                    ((picture + 4 * k) as *mut u32).write_unaligned(if k % 5 == 0 { 0 } else { v });
                }
                game_write(crate::marks::kind_values::KIND_RVA, kind);
                game_write(DONE_RVA, mark);
                NOTED.lock().unwrap().clear();
            };
            // The copy goes on into itself once done; when not, it writes the game's address.
            prepare(if done { resume_copy } else { 0 });
            let mut theirs = Regs::SENTINEL;
            unsafe { crate::framework::call_game(&mut theirs, copy.addr()) };
            let theirs = (
                regs(&theirs),
                std::mem::take(&mut *NOTED.lock().unwrap()),
                capture(&regions),
            );
            prepare(if done { resume_copy } else { 0 });
            let resume = if done { resume_copy } else { resume_game };
            let mine = unsafe { load_edges(&Regs::SENTINEL, resume, &mut test_step) };
            let mine = (
                mine.as_ref().map(regs).unwrap_or_default(),
                std::mem::take(&mut *NOTED.lock().unwrap()),
                capture(&regions),
            );
            t.check(theirs == mine, || {
                format!(
                    "kind {kind:#x}, done {done}: original {:x?} {:x?}; rust {:x?} {:x?}",
                    theirs.0, theirs.1, mine.0, mine.1
                )
            });
        }
    }
    restore(&regions, &saved);
    drop(scene);
    t.finish()
}
