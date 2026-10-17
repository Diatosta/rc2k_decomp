//! `marks::tidy_marks` (0x4D146D): the marks' memory given back at a stage's unload. Its
//! steps, each a call (the nullsubs left out): `sub_4D06F3` (for each of the settings'
//! +0x5C cars from the first, its list at +0x2E8 freed and its +0x328 made
//! `g_marksTaken`), then `memory::free_list` of `g_marksRingList` (`sub_4CFB3D`),
//! `stru_935620` (`sub_4CB5A8`), `g_marksBMaskList` and `stru_934D76` (`sub_4C8F4D`),
//! `g_marksPlacesList` (`sub_4C38F5`) and `g_marksLinksList` (`sub_4C4E22` ->
//! `sub_4C3EE3` -> `sub_4C5409`). Those steps are not hooked of their own: each is a
//! `lea ebx` and a call.
//!
//! In:  nothing (`g_marksSettings`)
//! Out: eax the cars' count, ebx `g_marksLinksList`, ebp the last car's +0x29C (the
//!      settings when there is none); the rest kept
//!
//! `sub_4D06F3` tests `memory::free_list`'s carry (`jb`), which is always clear (its last
//! flag-setting instruction is a `test`), so its other path (a `push` left on the stack
//! and a count of 0x1FFFFFFF: section 16) is never taken. Caller: the stage's unload
//! (0x4449AE). It begins with its call: a private copy (0x47 bytes, its 14 calls
//! relocated) with a manual patch. Shadow mode runs the original alone (it frees).
//! Refuses (before anything) null settings, a cars' count below 0 or above 8 (the
//! original's loop runs until it meets the count), and the game's own heap with a list
//! that holds blocks (`free_list`'s refusal). Self-test: with every list empty in the game,
//! blocks of the test's put in each, freed by the original and by the Rust version; the
//! lists, the memory counters, the cars' +0x328 and the registers compared.

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_read, hook_asm, rva, shadow};
use crate::marks::lines::{SETTINGS_RVA, TAKEN_RVA};
use crate::memory::free_list::free_list;
use crate::memory::{HELD_RVA, MOST_HELD_RVA, MemoryList, OWN_HEAP_RVA, TURNOVER_RVA};
use crate::service::harness::{Region, at, capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new("marks::tidy_marks", rva(0x4D146D), thunk, &ORIGINAL)
    .private_copy(0x47)
    .relocate_calls(&[
        0x0, 0x5, 0xA, 0xF, 0x14, 0x19, 0x1E, 0x23, 0x28, 0x2D, 0x32, 0x37, 0x3C, 0x41,
    ])
    .manual_patch()
    .self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const CAR_LEN: u32 = 0x4C8;
const CAR_PART: u32 = 0x29C; // `sub_4D06F3`'s ebp: the car's +0x29C
const MOST_CARS: i32 = 8;
/// The six lists, in the order freed.
const LISTS_RVA: [usize; 6] = [
    rva(0x93A580), // g_marksRingList
    rva(0x935620), // stru_935620
    rva(0x934DBA), // g_marksBMaskList
    rva(0x934D76), // stru_934D76
    rva(0x8DB570), // g_marksPlacesList
    rva(0x9206F0), // g_marksLinksList
];
const LIST_LEN: usize = std::mem::size_of::<MemoryList>();

/// Each car's list, worked out without writing (none for a count past the refusals).
unsafe fn car_lists(settings: u32) -> Vec<u32> {
    unsafe {
        let count = (*((settings + 0x5C) as *const i32)).clamp(0, MOST_CARS) as u32;
        (0..count)
            .map(|n| settings + n * CAR_LEN + CAR_PART + 0x4C)
            .collect()
    }
}

/// Every list it frees, the cars' first.
unsafe fn all_lists(settings: u32) -> Vec<u32> {
    unsafe {
        let mut r = car_lists(settings);
        r.extend(LISTS_RVA.map(|a| game_addr(a) as u32));
        r
    }
}

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn tidy_marks(entry: &Regs) -> Result<Regs, Refused> {
    let mut g = *entry;
    unsafe {
        let settings: u32 = game_read(SETTINGS_RVA);
        if settings == 0 {
            return Err(BadInput("no marks settings").into());
        }
        let count = *((settings + 0x5C) as *const i32);
        if !(0..=MOST_CARS).contains(&count) {
            return Err(BadInput("a cars' count below 0 or above 8").into());
        }
        if game_read::<u32>(OWN_HEAP_RVA) == 1
            && all_lists(settings)
                .iter()
                .any(|&l| (*(l as *const MemoryList)).head != 0)
        {
            return Err(
                BadInput("the game's own heap is in use with a list holding blocks").into(),
            );
        }
        g.ebp = settings;
        for n in 0..count as u32 {
            g.ebp = settings + n * CAR_LEN + CAR_PART;
            g.ebx = g.ebp + 0x4C;
            free_list(g.ebx as *mut MemoryList)?;
            ((g.ebp + 0x8C) as *mut u32).write_unaligned(game_read(TAKEN_RVA));
        }
        g.eax = count as u32;
        for a in LISTS_RVA {
            g.ebx = game_addr(a) as u32;
            free_list(g.ebx as *mut MemoryList)?;
        }
    }
    Ok(g)
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    match unsafe { tidy_marks(r) } {
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

/// What the self-test compares: the lists, the counters, the cars' +0x328.
unsafe fn compared(settings: u32) -> Vec<Region> {
    unsafe {
        let mut r: Vec<Region> = all_lists(settings)
            .into_iter()
            .map(|l| (l, LIST_LEN))
            .collect();
        r.extend([HELD_RVA, MOST_HELD_RVA, TURNOVER_RVA].map(at));
        r.extend(
            car_lists(settings)
                .into_iter()
                .map(|l| (l - 0x4C + 0x8C, 4)),
        );
        r
    }
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    unsafe {
        let settings: u32 = game_read(SETTINGS_RVA);
        if settings == 0 {
            return t.skip("no marks settings");
        }
        let count = *((settings + 0x5C) as *const i32);
        if !(0..=MOST_CARS).contains(&count) || game_read::<u32>(OWN_HEAP_RVA) == 1 {
            return t.skip("the cars' count or the game's own heap");
        }
        let lists = all_lists(settings);
        if lists.iter().any(|&l| (*(l as *const MemoryList)).head != 0) {
            return t.skip("a marks list holds blocks");
        }
        let vars = compared(settings);
        let saved = capture(&vars);
        let fill = |sizes: &[u32]| {
            restore(&vars, &saved);
            for (k, &l) in lists.iter().enumerate() {
                for &bytes in sizes.iter().skip(k % 2) {
                    let _ = crate::memory::allocate::allocate(l as *mut MemoryList, bytes);
                }
            }
        };
        for sizes in [&[][..], &[0x100][..], &[0x80, 0x2000, 0x40][..]] {
            fill(sizes);
            let mut ro = Regs::SENTINEL;
            call_original(&mut ro);
            let theirs = (
                capture(&vars),
                [ro.eax, ro.ebx, ro.ecx, ro.edx, ro.esi, ro.edi, ro.ebp],
            );
            fill(sizes);
            let mine = tidy_marks(&Regs::SENTINEL);
            let ours = (
                capture(&vars),
                mine.as_ref()
                    .map(|g| [g.eax, g.ebx, g.ecx, g.edx, g.esi, g.edi, g.ebp])
                    .unwrap_or_default(),
            );
            t.check(mine.is_ok() && theirs == ours, || {
                format!(
                    "{sizes:x?}: original {theirs:x?}; rust {:?} {ours:x?}",
                    mine.err()
                )
            });
        }
        restore(&vars, &saved);
    }
    t.finish()
}
