//! The hook and self-test machinery for functions that only change game variables. Shadow
//! mode cannot simply run such a function twice, so it runs the Rust version, notes what it
//! left, puts the variables back, runs the original and compares.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, shadow};
use crate::input::take_key::{notes_taken_keys, noting_taken_key, replaying_key};
use crate::screens::harness::{CallOriginal, Left};
use crate::service::state::{
    FLAG_TABLE_LEN, MENU_ITEM_RVA, REPAIR_FLAGS_RVA, REPAIR_MARKS_RVA, REPAIR_TOTAL_RVA,
    STATUS_RVA, TIME_BUDGET_RVA,
};
use std::slice::{from_raw_parts, from_raw_parts_mut};

/// A piece of game memory a function may change: (address, bytes).
pub type Region = (u32, usize);

pub fn at(rva: usize) -> Region {
    (game_addr(rva) as u32, 4)
}

pub fn capture(regions: &[Region]) -> Vec<u8> {
    let mut bytes = Vec::new();
    for &(addr, len) in regions {
        bytes.extend_from_slice(unsafe { from_raw_parts(addr as *const u8, len) });
    }
    bytes
}

pub fn restore(regions: &[Region], mut bytes: &[u8]) {
    for &(addr, len) in regions {
        unsafe { from_raw_parts_mut(addr as *mut u8, len) }.copy_from_slice(&bytes[..len]);
        bytes = &bytes[len..];
    }
}

/// The Rust version as the harness runs it: it gets the registers at entry and says what it
/// leaves in them. It must not have written anything when it returns an error.
pub type Rust<'a> = &'a dyn Fn(&Regs) -> Result<Left, Refused>;

/// Shadow comparisons per hook and second at most: a hook called thousands of times a
/// frame (per face, per polygon, per particle) would otherwise slow a race to a crawl. The
/// calls past it run the original alone, which is what shadow mode gives the game anyway.
pub const SHADOW_PER_SECOND: u32 = 20;

/// Whether this shadow-mode call of `hook` is compared (see `SHADOW_PER_SECOND`).
pub fn shadow_due(hook: &RawHook) -> bool {
    use std::collections::HashMap;
    use std::sync::{Mutex, OnceLock};
    use std::time::Instant;
    static SEEN: OnceLock<Mutex<HashMap<usize, (u64, u32)>>> = OnceLock::new();
    static START: OnceLock<Instant> = OnceLock::new();
    let second = START.get_or_init(Instant::now).elapsed().as_secs();
    let key = hook as *const RawHook as usize;
    let mut seen = SEEN
        .get_or_init(|| Mutex::new(HashMap::new()))
        .lock()
        .unwrap();
    let entry = seen.entry(key).or_insert((second, 0));
    if entry.0 != second {
        *entry = (second, 0);
    }
    entry.1 += 1;
    entry.1 <= SHADOW_PER_SECOND
}

/// A short account of where two captures differ: how many bytes, the first, and 16 bytes
/// from there on each side (a whole capture can be megabytes).
pub fn diff_summary(theirs: &[u8], mine: &[u8]) -> String {
    let differing = theirs.iter().zip(mine).filter(|(a, b)| a != b).count();
    match theirs.iter().zip(mine).position(|(a, b)| a != b) {
        None if theirs.len() == mine.len() => "same".into(),
        None => format!("lengths {} / {}", theirs.len(), mine.len()),
        Some(n) => {
            let end = (n + 16).min(theirs.len()).min(mine.len());
            format!(
                "{differing} bytes differ, the first at {n:#x}: original {:x?}, rust {:x?}",
                &theirs[n..end],
                &mine[n..end]
            )
        }
    }
}

/// The whole `dispatch` of such a hook. `regions` is everything the function may write.
pub fn dispatch_state(
    hook: &RawHook,
    r: &mut Regs,
    call_original: CallOriginal,
    regions: &[Region],
    rust: Rust,
) {
    hook.count_call();

    if !shadow() {
        match rust(r) {
            Ok(left) => left.apply(r),
            Err(e) => unsafe { crate::fatal::raise::refused(hook, e, r, call_original) },
        }
        return;
    }

    if !shadow_due(hook) {
        unsafe { call_original(r) };
        return;
    }

    let before = capture(regions);
    let left = match rust(r) {
        Ok(left) => left,
        Err(e) => {
            restore(regions, &before);
            unsafe { crate::fatal::raise::refused(hook, e, r, call_original) };
            return;
        }
    };
    let mine = capture(regions);
    restore(regions, &before);

    let mut o = *r;
    unsafe { call_original(&mut o) };
    let theirs = capture(regions);

    if theirs != mine || !left.matches(r, &o) {
        crate::screens::harness::mismatch_unless_disturbed(hook, || {
            format!(
                "regs_ok={}; memory: {}",
                left.matches(r, &o),
                diff_summary(&theirs, &mine)
            )
        });
    }
    let esp = r.esp;
    *r = o;
    r.esp = esp;
}

/// `dispatch_state` outside shadow mode, without its hook (typed tables, 2026-10-22: the entry the menus' and draw lists' enums run, with no hook): the Rust version, the
/// registers it leaves applied. A refusal is the caller's; the Rust version has written
/// nothing then.
pub fn state_entry(r: &mut Regs, rust: Rust) -> Result<(), Refused> {
    rust(r)?.apply(r);
    Ok(())
}

/// `dispatch_state` for a function that takes the pending key. The key comes from another
/// thread at any moment, so in shadow mode the original runs first, on the real key; then the
/// variables are put back, the Rust version runs with that same key replayed, the two results
/// are compared, and the original's result is put in place. `regions` must not include the
/// pending key itself. (The first version restored the key byte and ran the original second:
/// on 2026-10-10 that lost one key press and missed another in about 700 calls, each logged
/// as a mismatch.)
pub fn dispatch_keyed_state(
    hook: &RawHook,
    r: &mut Regs,
    call_original: CallOriginal,
    regions: &[Region],
    rust: Rust,
) {
    if !shadow() || !notes_taken_keys() {
        // With the key hook off nothing can be replayed: the old way, with its race.
        dispatch_state(hook, r, call_original, regions, rust);
        return;
    }
    hook.count_call();

    let before = capture(regions);
    let mut o = *r;
    let ((), key) = noting_taken_key(|| unsafe { call_original(&mut o) });
    let theirs = capture(regions);

    restore(regions, &before);
    let result = replaying_key(key.unwrap_or(0), || rust(r));
    let mine = capture(regions);
    restore(regions, &theirs);

    // The original has come back, so it met no fatal error.
    if let Err(crate::errors::Refused::Fatal(error)) = result {
        crate::screens::harness::mismatch_unless_disturbed(hook, || {
            format!(
                "the Rust version reported fatal error {:#x} {:?}; the original returned",
                error.code, error.message
            )
        });
    }
    if let Ok(left) = result
        && (theirs != mine || !left.matches(r, &o))
    {
        crate::screens::harness::mismatch_unless_disturbed(hook, || {
            format!(
                "key={key:x?} before {before:x?}: original {theirs:x?}; rust {mine:x?} regs_ok={}",
                left.matches(r, &o)
            )
        });
    }
    let esp = r.esp;
    *r = o;
    r.esp = esp;
}

/// `dispatch_state` for a function with an effect outside memory (a log line, a saved file)
/// that must not happen twice. In shadow mode the original runs first, for real; the
/// variables are put back; `replay`, the Rust version without that effect, runs; the results
/// are compared and the original's are put in place. Outside shadow mode `rust` runs.
pub fn dispatch_replayed_state(
    hook: &RawHook,
    r: &mut Regs,
    call_original: CallOriginal,
    regions: &[Region],
    rust: Rust,
    replay: Rust,
) {
    if !shadow() {
        dispatch_state(hook, r, call_original, regions, rust);
        return;
    }
    hook.count_call();
    let before = capture(regions);
    let mut o = *r;
    unsafe { call_original(&mut o) };
    let theirs = capture(regions);

    restore(regions, &before);
    let result = replay(r);
    let mine = capture(regions);
    restore(regions, &theirs);

    // The original has come back, so it met no fatal error.
    if let Err(crate::errors::Refused::Fatal(error)) = result {
        crate::screens::harness::mismatch_unless_disturbed(hook, || {
            format!(
                "the Rust version reported fatal error {:#x} {:?}; the original returned",
                error.code, error.message
            )
        });
    }
    if let Ok(left) = result
        && (theirs != mine || !left.matches(r, &o))
    {
        crate::screens::harness::mismatch_unless_disturbed(hook, || {
            format!(
                "before {before:x?}: original {theirs:x?}; rust {mine:x?} regs_ok={}",
                left.matches(r, &o)
            )
        });
    }
    let esp = r.esp;
    *r = o;
    r.esp = esp;
}

/// One self-test case: `prepare` puts the inputs in place, the original runs, `prepare` runs
/// again, the Rust version runs, and what the two left in `regions` and the registers is
/// compared. The caller saves and restores what `prepare` writes.
pub fn check_state(
    t: &mut SelfTest,
    what: &str,
    call_original: CallOriginal,
    before: Regs,
    prepare: &dyn Fn(),
    regions: &[Region],
    rust: Rust,
) {
    check_state_with(
        t,
        what,
        call_original,
        before,
        prepare,
        regions,
        rust,
        &Vec::new,
    );
}

/// `check_state` with something more to compare: `observe` is called after each of the two
/// runs, for what cannot be compared byte for byte (a copy of a running clock, say).
#[allow(clippy::too_many_arguments)]
pub fn check_state_with(
    t: &mut SelfTest,
    what: &str,
    call_original: CallOriginal,
    before: Regs,
    prepare: &dyn Fn(),
    regions: &[Region],
    rust: Rust,
    observe: &dyn Fn() -> Vec<u8>,
) {
    prepare();
    let mut ro = before;
    unsafe { call_original(&mut ro) };
    let theirs = (capture(regions), observe());

    prepare();
    let mine = rust(&before);
    let state = (capture(regions), observe());

    let regs_ok = mine.as_ref().is_ok_and(|left| left.matches(&before, &ro));
    t.check(regs_ok && theirs == state, || {
        format!(
            "{what}: regs_ok={regs_ok} original eax={:#x} ebx={:#x} ecx={:#x} edx={:#x} \
             esi={:#x} edi={:#x} ebp={:#x} carry={}; rust {mine:x?}; memory: {}; observed: {}",
            ro.eax,
            ro.ebx,
            ro.ecx,
            ro.edx,
            ro.esi,
            ro.edi,
            ro.ebp,
            ro.carry(),
            diff_summary(&theirs.0, &state.0),
            diff_summary(&theirs.1, &state.1)
        )
    });
}

/// What the repair choice functions may write between them: both flag tables, the status and
/// the total.
pub fn repair_regions() -> [Region; 4] {
    [
        (game_addr(REPAIR_FLAGS_RVA) as u32, FLAG_TABLE_LEN),
        (game_addr(REPAIR_MARKS_RVA) as u32, FLAG_TABLE_LEN),
        at(STATUS_RVA),
        at(REPAIR_TOTAL_RVA),
    ]
}

/// What a repair choice self-test writes besides `repair_regions`: saved by `begin`, put back
/// by `end`.
pub struct RepairTest {
    regions: Vec<Region>,
    saved: Vec<u8>,
}

/// The sixteen repair times, `dword_75AB60[16]`: the tests give them known values.
const REPAIR_TIMES: Region = (0, 16 * 4);
const REPAIR_TIMES_RVA: usize = crate::framework::rva(0x75AB60);

impl RepairTest {
    pub fn begin() -> Self {
        let mut regions = repair_regions().to_vec();
        regions.push(at(MENU_ITEM_RVA));
        regions.push(at(TIME_BUDGET_RVA));
        regions.push((game_addr(REPAIR_TIMES_RVA) as u32, REPAIR_TIMES.1));
        let saved = capture(&regions);
        Self { regions, saved }
    }

    /// Time `100 * (n + 1)` for the n-th of the sixteen.
    pub fn set_known_times() {
        for n in 0..16u32 {
            unsafe {
                ((game_addr(REPAIR_TIMES_RVA) as *mut u32).add(n as usize))
                    .write_unaligned(100 * (n + 1))
            };
        }
    }

    pub fn end(self) {
        restore(&self.regions, &self.saved);
    }
}

/// Flag patterns for the tests: only items that have a repair time are ever set, because the
/// original reads through a null pointer for the others.
pub const FLAG_PATTERNS: [&[u32]; 4] = [
    &[],
    &[2, 5],
    &[7, 0x12, 0x13],
    &[2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 0x12, 0x13],
];

/// Sets the flags of `chosen` to 1 and all others to 0, and fills the second table with
/// `mark`.
pub fn set_flags(chosen: &[u32], mark: u32) {
    use crate::service::state::{ITEM_COUNT, repair_mark_addr, set_repair_flag};
    for item in 0..ITEM_COUNT {
        set_repair_flag(item, chosen.contains(&item) as u32);
        unsafe { (repair_mark_addr(item) as *mut u32).write_unaligned(mark) };
    }
}
