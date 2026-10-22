//! `scene::sort_listed_faces` (0x45098C): puts an object's listed faces in the order
//! they are drawn in, the largest key first.
//!
//! In:  nothing
//! Out: the original leaves eax, ebx, ecx, edx, esi, edi and ebp changed when it
//!      sorts; its caller goes on without them, and the Rust version keeps them
//!
//! Nothing with fewer than two faces listed. Else the list's entries (0x10 bytes; the
//! key is the dword +4, compared unsigned) are sorted by `sub_4509BF`, a quicksort
//! written out here step for step, since which of two equal keys comes first is its
//! doing: the key in the middle is the pivot; the left place moves on one entry if
//! its key is above the pivot, the right place moves back while its key is below;
//! if the two places are not the same the entries change places, and the left one
//! moves on once more if their keys are now equal; when they meet, the part before
//! and the part after are sorted the same way.
//! One caller, `sub_45045D`.
//! Refuses a list longer than 0x1000 entries, and a sort that would go past the list
//! (never met in the core's tests), before changing anything.
//! Shadow mode: the Rust version, the list put back, the original; the lists compared.

use crate::errors::BadInput;
use crate::framework::{
    RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva, shadow,
};
use crate::scene::draw_object_faces::{LIST_RVA, LISTED_RVA};
use crate::service::harness::{Region, at, capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("scene::sort_listed_faces", rva(0x45098C), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const ENTRY_LEN: usize = 0x10;
const MOST: u32 = 0x1000;

// ---- Rust version ----

/// Converted: `rc2k_core::scene::face_order::sort_listed_faces`.
pub unsafe fn sort_listed_faces() -> Result<(), BadInput> {
    unsafe {
        let listed: u32 = game_read(LISTED_RVA);
        if listed > MOST {
            return Err(BadInput("more faces listed than the list holds"));
        }
        let list = std::slice::from_raw_parts_mut(
            game_addr(LIST_RVA) as *mut rc2k_core::scene::face_order::ListedFace,
            listed as usize,
        );
        Ok(rc2k_core::scene::face_order::sort_listed_faces(list)?)
    }
}

// ---- hook ----

fn regions() -> Vec<Region> {
    let listed = unsafe { game_read::<u32>(LISTED_RVA) }.min(MOST) as usize;
    vec![(game_addr(LIST_RVA) as u32, listed * ENTRY_LEN)]
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if !shadow() {
        if let Err(e) = unsafe { sort_listed_faces() } {
            unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) };
        }
        return;
    }
    let regions = regions();
    let before = capture(&regions);
    let mine = unsafe { sort_listed_faces() }.map(|()| capture(&regions));
    restore(&regions, &before);
    unsafe { call_original(r) };
    let Ok(mine) = mine else {
        return;
    };
    let theirs = capture(&regions);
    if mine != theirs {
        HOOK.mismatch(|| {
            format!(
                "{} faces: first different byte {:?}; before {before:x?}",
                before.len() / ENTRY_LEN,
                mine.iter().zip(&theirs).position(|(a, b)| a != b)
            )
        });
    }
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(_: &mut Regs) -> Result<(), crate::errors::Refused> {
    if let Err(e) = unsafe { sort_listed_faces() } {
        return Err(e.into());
    }
    Ok(())
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    const ROOM: usize = 0x80;
    let inputs = [
        at(LISTED_RVA),
        (game_addr(LIST_RVA) as u32, ROOM * ENTRY_LEN),
    ];
    let saved = capture(&inputs);
    let mut seed = 0x1234_5678u32;
    let mut next = move || {
        seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        seed >> 8
    };
    for listed in [0usize, 1, 2, 3, 4, 5, 7, 8, 16, 17, 31, 64, 100, ROOM] {
        // Keys of few different values, of many, all the same, already in order,
        // in the other order, with the top bit set.
        for kind in 0..6u32 {
            let keys: Vec<u32> = (0..listed as u32)
                .map(|n| match kind {
                    0 => next() % 4,
                    1 => next(),
                    2 => 7,
                    3 => 1000 - n,
                    4 => n,
                    _ => (next() << 24) | (next() % 3),
                })
                .collect();
            let run = |rust: bool| unsafe {
                restore(&inputs, &vec![0xAA; saved.len()]);
                game_write(LISTED_RVA, listed as u32);
                for (n, &key) in keys.iter().enumerate() {
                    // Each entry can be told from every other by its first dword.
                    ((game_addr(LIST_RVA) + n * ENTRY_LEN) as *mut [u32; 4]).write_unaligned([
                        n as u32,
                        key,
                        0xB000_0000 + n as u32,
                        0xC000_0000 + n as u32,
                    ]);
                }
                let done = if rust {
                    sort_listed_faces().is_ok()
                } else {
                    call_original(&mut Regs::default());
                    true
                };
                (done, capture(&inputs))
            };
            let (theirs, mine) = (run(false), run(true));
            t.check(theirs == mine, || {
                format!(
                    "{listed} faces, keys of kind {kind}: rust done {}; first different byte {:?}; keys {keys:x?}",
                    mine.0,
                    theirs.1.iter().zip(&mine.1).position(|(a, b)| a != b)
                )
            });
        }
    }
    restore(&inputs, &saved);
    t.finish()
}
