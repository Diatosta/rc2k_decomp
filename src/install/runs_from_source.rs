//! `install::runs_from_source` (0x412DA3): whether the game was started from the
//! directory the registry names as its installation's source.
//!
//! In:  nothing
//! Out: zero set if so; every register preserved
//! `read_registry`; then, only if the status is 1 and a source is named and the current
//! directory can be had (`files::current_directory` into `byte_94E8B0`), the two are
//! compared with `strings::same_text`. Everything else is "no" (zero and carry clear).
//! Callers: the main loop `sub_415383` on every start, and `install::check_clio_disc`.
//! Manual patch with a private copy (0x3D bytes, three calls relocated): a call starts
//! in the first bytes.

use crate::errors::{BadInput, Refused};
use crate::framework::{
    RawHook, Regs, SelfTest, call_game, game_addr, game_read, game_write, hook_asm, rva, shadow,
};
use crate::install::check_test::{self, World};
use crate::install::normalise_path::current_directory;
use crate::install::outside::{Game, Outside, READ_REGISTRY, Tested};
use crate::install::read_registry::{SOURCE_RVA, STATUS_RVA};
use crate::install::stand_ins::stand_in;
use crate::redirect::RedirectedCopy;
use crate::service::harness::{Region, capture, restore};
use crate::strings::same_text::compare_texts;
use std::cmp::Ordering;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("install::runs_from_source", rva(0x412DA3), thunk, &ORIGINAL)
        .private_copy(0x3D)
        .relocate_calls(&[0x01, 0x1E, 0x2D])
        .manual_patch()
        .self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// byte_94E8B0: the scratch path (also `files::numbered_path`'s buffer).
pub const SCRATCH_PATH_RVA: usize = rva(0x94E8B0);

// ---- Rust version ----

/// None if the question does not arise, else how the source stands to the current
/// directory (`Equal`: started from there).
pub unsafe fn source_against_directory(
    outside: &mut dyn Outside,
) -> Result<Option<Ordering>, BadInput> {
    unsafe {
        outside.read_registry();
        if game_read::<u8>(STATUS_RVA) != 1 || game_read::<u8>(SOURCE_RVA) == 0 {
            return Ok(None);
        }
        let directory = game_addr(SCRATCH_PATH_RVA) as *mut u8;
        if !outside.current_directory(directory)? {
            return Ok(None);
        }
        Ok(Some(compare_texts(
            game_addr(SOURCE_RVA) as *const u8,
            directory,
        )))
    }
}

pub unsafe fn runs_from_source(outside: &mut dyn Outside) -> Result<bool, BadInput> {
    Ok(unsafe { source_against_directory(outside) }? == Some(Ordering::Equal))
}

// ---- hook ----

fn regions() -> [Region; 1] {
    [(game_addr(SCRATCH_PATH_RVA) as u32, 0x100)]
}

/// (zero, carry) as the original leaves them.
fn flags(found: Option<Ordering>) -> (bool, bool) {
    (
        found == Some(Ordering::Equal),
        found == Some(Ordering::Less),
    )
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    let regions = regions();
    let before = capture(&regions);
    let mine = match unsafe { source_against_directory(&mut Game) } {
        Ok(mine) => flags(mine),
        Err(e) => {
            restore(&regions, &before);
            unsafe { crate::fatal::raise::refused(&HOOK, Refused::from(e), r, call_original) };
            return;
        }
    };
    if !shadow() {
        r.set_zero(mine.0);
        r.set_carry(mine.1);
        return;
    }
    let left = capture(&regions);
    restore(&regions, &before);
    let mut o = *r;
    unsafe { call_original(&mut o) };
    let theirs = (o.eflags & 0x40 != 0, o.carry());
    if theirs != mine || capture(&regions) != left || !o.preserved_except(r, 0) {
        HOOK.mismatch(|| {
            format!(
                "original zero={} carry={}; rust {mine:?}",
                theirs.0, theirs.1
            )
        });
    }
    r.eflags = o.eflags;
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let regions = regions();
    let before = capture(&regions);
    let mine = match unsafe { source_against_directory(&mut Game) } {
        Ok(mine) => flags(mine),
        Err(e) => {
            restore(&regions, &before);
            return Err(Refused::from(e));
        }
    };
    r.set_zero(mine.0);
    r.set_carry(mine.1);
    Ok(())
}

// ---- self-test: a copy of the original that asks the test for the registry ----

const LEN: usize = 0x3D;
const ENTRY_BYTES: [u8; 8] = [0x60, 0xE8, 0x6B, 0x0F, 0x00, 0x00, 0x80, 0x3D];
const CALLS: [usize; 3] = [0x01, 0x1E, 0x2D];
const READ_REGISTRY_RVA: usize = rva(0x413D14);

/// The original with its registry reading pointed at the test's world; the current
/// directory is this machine's. Also for `install::check_clio_disc`'s test.
pub unsafe fn copy_asking_the_test() -> Result<RedirectedCopy, &'static str> {
    unsafe {
        RedirectedCopy::new(
            game_addr(rva(0x412DA3)),
            LEN,
            &ENTRY_BYTES,
            &CALLS,
            &|callee| (callee == READ_REGISTRY_RVA).then(|| stand_in(READ_REGISTRY)),
        )
    }
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let Ok(Some(mut here)) = (unsafe { current_directory() }) else {
        return t.skip("the current directory cannot be had");
    };
    if !matches!(here.last(), Some(b'\\' | b'/')) {
        here.push(b'\\');
    }
    let copy = match unsafe { copy_asking_the_test() } {
        Ok(copy) => copy,
        Err(why) => return t.skip(why),
    };
    let other_case: Vec<u8> = here.iter().map(|c| c.to_ascii_uppercase()).collect();
    let longer = [&here[..], b"x"].concat();
    let shorter = here[..here.len() - 1].to_vec();
    let state: [Region; 2] = [(game_addr(STATUS_RVA) as u32, 0x100), regions()[0]];
    let saved = capture(&state);
    let cases: [Option<(u8, Vec<u8>)>; 9] = [
        None,
        Some((0, here.clone())),
        Some((1, Vec::new())),
        Some((1, here.clone())),
        Some((1, other_case)),
        Some((1, longer)),
        Some((1, shorter)),
        Some((2, here.clone())),
        Some((1, b"Q:\\Somewhere else\\".to_vec())),
    ];
    for registry in cases {
        let world = World {
            registry: registry.clone(),
            ..World::default()
        };
        let before = Regs::SENTINEL;
        let run = |rust: bool| unsafe {
            restore(&state, &vec![0xAA; saved.len()]);
            game_write(SOURCE_RVA + 0xFE, 0u8);
            check_test::begin(&world);
            let result = if rust {
                source_against_directory(&mut Tested).ok().map(flags)
            } else {
                let mut ro = before;
                call_game(&mut ro, copy.addr());
                ro.preserved_except(&before, 0)
                    .then_some((ro.eflags & 0x40 != 0, ro.carry()))
            };
            (result, check_test::asked(), capture(&state))
        };
        let (theirs, mine) = (run(false), run(true));
        t.check(theirs == mine && theirs.0.is_some(), || {
            format!(
                "registry={:?}: original {:?}; rust {:?}; same state={}",
                registry
                    .as_ref()
                    .map(|(status, source)| (status, String::from_utf8_lossy(source))),
                theirs.0,
                mine.0,
                theirs.2 == mine.2
            )
        });
    }
    restore(&state, &saved);
    t.finish()
}
