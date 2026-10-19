//! The replay viewer (`race::replay_viewer`, 0x41CF0C, IDA 2026-10-19) and its tree: the
//! variables they share, the `Part` their calls go through, and the self-tests' tools.
//!
//! The viewer plays a replay from `var\tmprpl` (the last race's, or a saved one copied
//! there first), block by block (`dword_5C2530`, 0 .. `dword_5BECD0`; 100 frames a block,
//! the frame in `dword_5C2534`), and edits it: two marks (`dword_96E478`, `dword_96E47C`),
//! cut, copy and paste of blocks (the clipboard `unk_96E488`, `unk_971CC8`), one undo
//! (`dword_9673F4` ..), the cameras of each block (`unk_963BB4`: 8 bytes a block, the
//! first 8 of the block's data), and saving the result as a replay slot.
//!
//! The tests: a copy of the original whose calls go to `install::stand_ins` (`copy`),
//! noted by `note` with eax, ebx, ecx and edx and answered by the test's `answer` (eax and
//! the flags, memory written); the Rust side calls `test_part`, which does the same. Then
//! `run_both` compares the registers, the flags the callers read, the calls and memory.

use crate::errors::Refused;
use crate::framework::{CARRY_FLAG, RawHook, Regs, SelfTest, ZERO_FLAG, game_addr, shadow};
use crate::physics::flow::{a, g, p};
use crate::service::harness::{Region, capture, diff_summary, restore};
use std::sync::Mutex;

/// A call by place.
pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;

// ---- the replay being viewed ----

pub const OPEN: usize = 0x5BEC88; // dword_5BEC88: `h` is the replay's file
pub const HANDLE: usize = 0x5BEC8C; // h
pub const AT_END: usize = 0x5BEC90; // dword_5BEC90: the last frame was reached
pub const HOLD_REVS: usize = 0x5BEC94; // g_holdCarRevs
pub const STOPPED: usize = 0x5BEC98; // dword_5BEC98: 1 = not playing
pub const MODE: usize = 0x5BEC9C; // dword_5BEC9C: the viewer's eax
pub const SLOT: usize = 0x5BECA0; // dword_5BECA0: the replay (222 the last race's)
pub const SOUND: usize = 0x5BECA4; // dword_5BECA4: the menu sound to play
pub const EDITING: usize = 0x5BECA8; // dword_5BECA8: the editing keys work
pub const INDEX: usize = 0x5BECC8; // "RCIB": the index, 0x3850 bytes
pub const BLOCKS: usize = 0x5BECD0; // dword_5BECD0: blocks in the replay
pub const PLACES: usize = 0x5BECD8; // lDistanceToMove: (offset, size) a block
pub const MOST_BLOCKS: u32 = 0x708;
pub const LAST_RACE: u32 = 0xDE; // the slot of `var\tmprpl` as it is
pub const BLOCK: usize = 0x5C2530; // dword_5C2530: the block shown
pub const FRAME: usize = 0x5C2534; // dword_5C2534: its frame, 0 .. 99
pub const PLAY_END: usize = 0x5C2538; // dword_5C2538: playing stops before this block
pub const PLAY_FROM_HERE: usize = 0x5C253C; // dword_5C253C
pub const SNAPSHOT: usize = 0x5C2540; // dword_5C2540: a picture of the next frame asked
pub const LAST_RACE_VIEWED: usize = 0x5C2544; // dword_5C2544: the slot is 222
pub const ONLY_SHOWN: usize = 0x5C2548; // dword_5C2548: modes 4 and 5
pub const KEYS_IGNORED: usize = 0x5C254C; // dword_5C254C: mode 5
pub const KEY_AFTER_PLAY: usize = 0x5C2550; // dword_5C2550
pub const WAIT_RELEASE: usize = 0x5C2554; // dword_5C2554: keys held after playing
pub const ESCAPES: usize = 0x5C2558; // dword_5C2558: Escape twice leaves
pub const MENU: usize = 0x5C255C; // unk_5C255C: the viewer's list (items at +0x18)
pub const RECORDING: usize = 0x5C2CCC; // g_replayRecording: the frames saved as pictures
pub const PICTURES_STARTED: usize = 0x5C2CD4; // dword_5C2CD4
pub const PICTURES_STARTED_2: usize = 0x5C2CD0; // dword_5C2CD0
pub const DEVELOPER_KEYS: usize = 0x5C2CEB; // dword_5C2CEB
pub const PICTURE_RATE: usize = 0x5C2CEF; // dword_5C2CEF: pictures a second, 1 .. 100
pub const PICTURE_TIME: usize = 0x5C2CF3; // dword_5C2CF3: ms a picture
pub const CAMERA_NAMES: usize = 0x5C291C; // word_5C291C: the cameras' text ids, 0x2F
pub const BLOCK_DATA: usize = 0x9613A4; // dword_9613A4: the block read, 0x2810 bytes
pub const BLOCK_CURSOR: usize = 0x9613A0; // dword_9613A0
pub const CAMERAS: usize = 0x963BB4; // unk_963BB4: 8 bytes a block
pub const UNDO_BLOCKS: usize = 0x9673F4; // dword_9673F4: 0 = nothing to undo
pub const UNDO_PLACES: usize = 0x9673F8; // dword_9673F8: 0x3840 bytes
pub const UNDO_CAMERAS: usize = 0x96AC38; // unk_96AC38: 0x3840 bytes
pub const TABLE_LEN: usize = 0x3840; // 0x708 blocks of 8 bytes
pub const MARK_IN: usize = 0x96E478; // dword_96E478
pub const MARK_OUT: usize = 0x96E47C; // dword_96E47C
pub const CAMERA_EDIT: usize = 0x96E480; // dword_96E480: 0, or the camera (1, 2) edited
pub const CLIP_BLOCKS: usize = 0x96E484; // dword_96E484: blocks on the clipboard
pub const CLIP_PLACES: usize = 0x96E488; // unk_96E488
pub const CLIP_CAMERAS: usize = 0x971CC8; // unk_971CC8
pub const HEADER: usize = 0x528504; // dword_528504: the file's header, 0x430
pub const CAR_HEADER: usize = 0x719B14; // dword_719B14: 0x400 bytes
pub const ONE_PLAYER: usize = 0x719B34; // dword_719B34 (the header's)
pub const FIRST_CAR: usize = 0x719B24; // dword_719B24 (the header's)

// ---- the list menu (`menu::list_*`) ----

pub const CHOICE: usize = 0x51D407; // dword_51D407: the item highlighted
pub const CHOICE_BEFORE: usize = 0x51D40B; // dword_51D40B
pub const REPEATS: usize = 0x51D40F; // dword_51D40F
pub const LAST_KEY: usize = 0x51D413; // byte_51D413
pub const LIST_SHOWN: usize = 0x51D415; // byte_51D415
pub const LIST_LEAVE: usize = 0x51D416; // byte_51D416: 1 leaves
pub const KEY_CLOCK: usize = 0x51D41B; // dword_51D41B
pub const FRAME_CLOCK: usize = 0x51D41F; // dword_51D41F
pub const MOVE_CLOCK: usize = 0x51D423; // dword_51D423
pub const CLOCK: usize = 0x518998; // dword_518998

/// Clamped to the game's tables: a block number as the original indexes by it.
pub fn place_of(block: u32) -> usize {
    PLACES + 8 * block as usize
}

pub fn camera_of(block: u32) -> usize {
    CAMERAS + 8 * block as usize
}

pub fn gb(x: usize) -> u8 {
    unsafe { (a(x) as *const u8).read() }
}

pub fn pb(x: usize, v: u8) {
    unsafe { (a(x) as *mut u8).write(v) }
}

/// `rep movsd` of `dwords` from `from` to `to` (IDA addresses), forwards.
pub fn move_dwords(to: usize, from: usize, dwords: usize) {
    unsafe {
        let (to, from) = (a(to) as *mut u32, a(from) as *const u32);
        for n in 0..dwords {
            to.add(n).write_unaligned(from.add(n).read_unaligned());
        }
    }
}

/// The flags of `cmp v, 1` as `jb` / `jz` read them.
pub fn cmp_one(r: &mut Regs, v: u32) {
    r.set_carry(v < 1);
    r.set_zero(v == 1);
}

/// A region of IDA address `x`, `len` bytes.
pub fn region(x: usize, len: usize) -> Region {
    (a(x), len)
}

pub fn dword(x: usize) -> Region {
    region(x, 4)
}

// ---- for the hooks: shadow comparison of a Rust version that returns the registers ----

/// The Rust version as the dispatchers run it: the registers it leaves, flags included.
pub type Rust<'a> = &'a dyn Fn(&Regs) -> Result<Regs, Refused>;

fn same(o: &Regs, m: &Regs, flags: u32) -> bool {
    let regs = |r: &Regs| [r.eax, r.ebx, r.ecx, r.edx, r.esi, r.edi, r.ebp];
    regs(o) == regs(m) && (o.eflags ^ m.eflags) & flags == 0
}

/// The whole `dispatch` of a hook whose Rust version only changes memory in `regions` and
/// the registers: the Rust version outside shadow mode; in it the Rust version first, its
/// memory put back, the original, and the registers, the flags in `flags` and `regions`
/// compared (the original's are kept).
pub fn dispatch_regs(
    hook: &RawHook,
    r: &mut Regs,
    call_original: crate::screens::harness::CallOriginal,
    regions: &[Region],
    flags: u32,
    rust: Rust,
) {
    hook.count_call();
    if !shadow() {
        match rust(r) {
            Ok(left) => *r = Regs { esp: r.esp, ..left },
            Err(e) => unsafe { crate::fatal::raise::refused(hook, e, r, call_original) },
        }
        return;
    }
    if !crate::service::harness::shadow_due(hook) {
        unsafe { call_original(r) };
        return;
    }
    let before = capture(regions);
    let mine = match rust(r) {
        Ok(left) => left,
        Err(e) => {
            restore(regions, &before);
            unsafe { crate::fatal::raise::refused(hook, e, r, call_original) };
            return;
        }
    };
    let state = capture(regions);
    restore(regions, &before);
    let mut o = *r;
    unsafe { call_original(&mut o) };
    let theirs = capture(regions);
    if theirs != state || !same(&o, &mine, flags) {
        crate::screens::harness::mismatch_unless_disturbed(hook, || {
            format!(
                "entry eax={:#x} ebx={:#x} ecx={:#x} edx={:#x} esi={:#x} edi={:#x}; regs_ok={}; memory: {}",
                r.eax,
                r.ebx,
                r.ecx,
                r.edx,
                r.esi,
                r.edi,
                same(&o, &mine, flags),
                diff_summary(&theirs, &state)
            )
        });
    }
    let esp = r.esp;
    *r = o;
    r.esp = esp;
}

/// The `dispatch` of a hook whose Rust version calls through `part` and whose calls have
/// effects that must not happen twice (files, frames, menus): the Rust version outside
/// shadow mode, the original alone in it.
pub fn dispatch_parts(
    hook: &RawHook,
    r: &mut Regs,
    call_original: crate::screens::harness::CallOriginal,
    rust: &dyn Fn(&Regs) -> Result<Regs, Refused>,
) {
    hook.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    match rust(r) {
        Ok(left) => *r = Regs { esp: r.esp, ..left },
        Err(e) => unsafe { crate::fatal::raise::refused(hook, e, r, call_original) },
    }
}

// ---- for the self-tests ----

/// One call noted: (place, eax, ebx, ecx, edx).
pub type Call = (u32, u32, u32, u32, u32);

static CALLED: Mutex<Vec<Call>> = Mutex::new(Vec::new());

/// What a stand-in does after it is noted: the test's answer by place.
pub type Answer = Box<dyn FnMut(u32, &mut Regs) + Send>;

static ANSWER: Mutex<Option<Answer>> = Mutex::new(None);

/// The stand-ins' handler: the call noted, then answered.
pub fn note(place: u32, r: &mut Regs) {
    CALLED
        .lock()
        .unwrap()
        .push((place, r.eax, r.ebx, r.ecx, r.edx));
    if let Some(answer) = ANSWER.lock().unwrap().as_mut() {
        answer(place, r);
    }
}

/// The Rust side's calls in the tests.
pub fn test_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    note(place, r);
    Ok(())
}

pub fn take_called() -> Vec<Call> {
    std::mem::take(&mut *CALLED.lock().unwrap())
}

/// The answer the stand-ins give from now on (`prepare` sets it again for each side).
pub fn answer(f: Answer) {
    *ANSWER.lock().unwrap() = Some(f);
}

/// Stand-ins that answer nothing (only note).
pub fn quiet() {
    *ANSWER.lock().unwrap() = None;
}

/// An answer for every place: the carry flag clear and nothing else.
pub fn clear_carry() -> Answer {
    Box::new(|_, r| r.set_carry(false))
}

/// A copy whose calls to `places`' callees (IDA addresses) go to stand-ins of those places;
/// other calls go where they went.
pub fn copy(
    address: usize,
    len: usize,
    entry: &[u8],
    calls: &[usize],
    places: &'static [(usize, u32)],
) -> Result<crate::redirect::RedirectedCopy, &'static str> {
    crate::install::stand_ins::set_handler(note);
    unsafe {
        crate::redirect::RedirectedCopy::new(
            game_addr(crate::framework::rva(address)),
            len,
            entry,
            calls,
            &|c| {
                places
                    .iter()
                    .find(|&&(k, _)| crate::framework::rva(k) == c)
                    .map(|&(_, p)| crate::install::stand_ins::stand_in(p))
            },
        )
    }
}

/// One case: `prepare`, the copy (or original) at `code` from `before`, `prepare`, the Rust
/// version from `before`; the registers, the flags in `flags`, the calls and `regions`
/// compared. A Rust refusal counts as a failure.
#[allow(clippy::too_many_arguments)] // the case: what, code, start, memory, flags, sides
pub fn run_both(
    t: &mut SelfTest,
    what: &str,
    code: usize,
    before: Regs,
    regions: &[Region],
    flags: u32,
    prepare: &dyn Fn(),
    rust: Rust,
) {
    prepare();
    take_called();
    let mut ro = before;
    unsafe { crate::framework::call_game(&mut ro, code) };
    let theirs = (take_called(), capture(regions));
    prepare();
    let mine = rust(&before);
    let state = (take_called(), capture(regions));
    let regs_ok = mine.as_ref().is_ok_and(|m| same(&ro, m, flags));
    t.check(regs_ok && theirs == state, || {
        format!(
            "{what}: regs_ok={regs_ok}: original eax={:#x} ebx={:#x} ecx={:#x} edx={:#x} \
             esi={:#x} edi={:#x} ebp={:#x} flags={:#x}; rust {}; calls original {:x?} rust \
             {:x?}; memory: {}",
            ro.eax,
            ro.ebx,
            ro.ecx,
            ro.edx,
            ro.esi,
            ro.edi,
            ro.ebp,
            ro.eflags & flags,
            match &mine {
                Ok(m) => format!(
                    "eax={:#x} ebx={:#x} ecx={:#x} edx={:#x} esi={:#x} edi={:#x} ebp={:#x} flags={:#x}",
                    m.eax,
                    m.ebx,
                    m.ecx,
                    m.edx,
                    m.esi,
                    m.edi,
                    m.ebp,
                    m.eflags & flags
                ),
                Err(e) => format!("refused {e:?}"),
            },
            theirs.0,
            state.0,
            diff_summary(&theirs.1, &state.1)
        )
    });
}

/// `run_both` against the hook's original itself (a function with no calls, or whose calls
/// are the Rust versions on either side).
#[allow(clippy::too_many_arguments)] // as `run_both`
pub fn check_original(
    t: &mut SelfTest,
    what: &str,
    call_original: crate::screens::harness::CallOriginal,
    before: Regs,
    regions: &[Region],
    flags: u32,
    prepare: &dyn Fn(),
    rust: Rust,
) {
    prepare();
    let mut ro = before;
    unsafe { call_original(&mut ro) };
    let theirs = capture(regions);
    prepare();
    let mine = rust(&before);
    let state = capture(regions);
    let regs_ok = mine.as_ref().is_ok_and(|m| same(&ro, m, flags));
    t.check(regs_ok && theirs == state, || {
        format!(
            "{what}: regs_ok={regs_ok}: original eax={:#x} ebx={:#x} ecx={:#x} edx={:#x} \
             esi={:#x} edi={:#x} flags={:#x}; rust {}; memory: {}",
            ro.eax,
            ro.ebx,
            ro.ecx,
            ro.edx,
            ro.esi,
            ro.edi,
            ro.eflags & flags,
            match &mine {
                Ok(m) => format!(
                    "eax={:#x} ebx={:#x} ecx={:#x} edx={:#x} esi={:#x} edi={:#x} flags={:#x}",
                    m.eax,
                    m.ebx,
                    m.ecx,
                    m.edx,
                    m.esi,
                    m.edi,
                    m.eflags & flags
                ),
                Err(e) => format!("refused {e:?}"),
            },
            diff_summary(&theirs, &state)
        )
    });
}

pub const CF: u32 = CARRY_FLAG;
pub const ZF: u32 = ZERO_FLAG;

/// Saved and put back over a test.
pub struct Saved(Vec<Region>, Vec<u8>);

impl Saved {
    pub fn new(regions: &[Region]) -> Self {
        Self(regions.to_vec(), capture(regions))
    }
}

impl Drop for Saved {
    fn drop(&mut self) {
        restore(&self.0, &self.1);
    }
}

/// A recognisable byte pattern for test tables.
pub fn fill(x: usize, len: usize, seed: u32) {
    unsafe {
        let at = a(x) as *mut u8;
        for n in 0..len {
            at.add(n)
                .write((n as u32).wrapping_mul(0x9E37).wrapping_add(seed) as u8 ^ (n >> 8) as u8);
        }
    }
}

/// `g` and `p` for the hooks here.
pub fn get(x: usize) -> u32 {
    g(x)
}

pub fn put(x: usize, v: u32) {
    p(x, v)
}

/// Every register of `r` as a `Left` (for the harnesses that take one), no flags.
pub fn left_of(r: &Regs) -> crate::screens::harness::Left {
    use crate::framework::{EAX, EBP, EBX, ECX, EDI, EDX, ESI};
    crate::screens::harness::Left::nothing()
        .with(EAX, r.eax)
        .with(EBX, r.ebx)
        .with(ECX, r.ecx)
        .with(EDX, r.edx)
        .with(ESI, r.esi)
        .with(EDI, r.edi)
        .with(EBP, r.ebp)
}
