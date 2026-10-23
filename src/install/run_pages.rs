//! `install::run_pages` (0x4144AA): the installer's pages, from the entry esi on: each entry
//! is the page's list and the entries that follow its answers (+4 + 4 x answer); a page
//! answered 0 or 1 has its highlight put back to 0; an answer of no entry (0) or the game
//! quitting ends it. (The original calls itself with `jnz` to its start.)
//!
//! In:  esi the first entry (`off_51D594`, or `off_51D5D8` for an update)
//! Out: esi 0 (or the next entry when quitting), edi the last page's list, eax its answer;
//!      the flags of the last test
//! Caller `sub_412DE0` (`install::run_installer`, twice). Its calls through `part`.
//! Ungated. Shadow mode runs the original alone. Self-test: a copy whose page runs go to a
//! stand-in answering as the test says, against the Rust version, on entries of the test's.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::install::installer::{
    Part, QUITTING, ZF, answer, at, both, cmp, copy, g, quiet, rd, wd,
};
use crate::race::viewer::{dispatch_parts, regs_entry};
use rc2k_core::install::pages;
use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new("install::run_pages", rva(0x4144AA), thunk, &ORIGINAL)
    .private_copy(0x25)
    .relocate_calls(&[0x2])
    .manual_patch()
    .self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL,
    ungated
);

const RUN_PAGE: usize = 0x4144CF; // install::run_page

pub fn run_pages(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    loop {
        r.edi = rd(r.esi);
        part(at(RUN_PAGE), &mut r)?;
        let answer = r.eax;
        cmp(&mut r, answer, 1);
        // Converted: `rc2k_core::install::pages` (`resets_highlight`, `next_entry_at`).
        if pages::resets_highlight(answer) {
            wd(r.edi + 4, 0);
        }
        r.esi = rd(r.esi.wrapping_add(pages::next_entry_at(answer)));
        let quitting = g(QUITTING);
        cmp(&mut r, quitting, 1);
        if quitting == 1 {
            return Ok(r);
        }
        let next = r.esi;
        crate::install::installer::logic(&mut r, next);
        if r.esi == 0 {
            return Ok(r);
        }
    }
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_parts(&HOOK, r, call_original, &|r| {
        run_pages(r, &mut crate::install::installer::game_part)
    });
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    regs_entry(r, &|r| {
        run_pages(r, &mut crate::install::installer::game_part)
    })
}

static TURN: AtomicU32 = AtomicU32::new(0);

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // mov edi, [esi]; call sub_4144CF
    let copy = match copy(
        0x4144AA,
        0x25,
        &[0x8B, 0x3E, 0xE8, 0x1E, 0x00, 0x00, 0x00],
        &[(0x2, 0x4144CF)],
        &[0x4144CF],
        &[],
        &[],
        &[],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    // Three lists of 0x18 bytes and four entries of four dwords.
    let mut mem = vec![0u32; 0x40];
    let b = mem.as_mut_ptr() as u32;
    let entry = |n: u32| b + 0x60 + 0x10 * n;
    for answers in [[2u32, 1, 0, 0], [1, 2, 2, 2], [0, 0, 0, 0], [2, 2, 1, 1]] {
        let prepare = || {
            for n in 0..0x18 {
                wd(b + 4 * n, 0x1111_0000 + n);
            }
            // entry n: list n % 3, then on 0 none, on 1 entry n+1 (none after the last),
            // on 2 entry 0
            for n in 0..4 {
                wd(entry(n), b + 0x18 * (n % 3));
                wd(entry(n) + 4, 0);
                wd(entry(n) + 8, if n < 3 { entry(n + 1) } else { 0 });
                wd(entry(n) + 0xC, entry(0));
            }
            TURN.store(0, Ordering::SeqCst);
            answer(Box::new(move |_, r| {
                let n = TURN.fetch_add(1, Ordering::SeqCst) as usize;
                r.eax = if n < 8 { answers[n % 4] } else { 0 };
            }));
        };
        both(
            &mut t,
            &format!("answers {answers:?}"),
            &copy,
            Regs {
                esi: entry(0),
                ..Regs::SENTINEL
            },
            &[(b, 0x100)],
            ZF,
            &[],
            &prepare,
            &|r, part, _| run_pages(r, part),
        );
    }
    quiet();
    drop(mem);
    t.finish()
}
