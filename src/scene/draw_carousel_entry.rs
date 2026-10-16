//! `scene::draw_carousel_entry` (0x4906D5): one entry of the main menu's carousel at
//! its place in the row.
//!
//! In:  ebx = the entry, ecx = its place in the row (-1 before the front one, 0 the
//!      front one, 1 and 2 after it)
//! Out: every register preserved
//!
//! The object's variables for `scene::draw_menu_object`:
//! - x (`dword_63501C`) = the spacing `dword_75A664` times (place - 1), plus the high
//!   half of the spacing times the turn `g_carouselAngle` inverted (the turn as a
//!   fraction of 2^32);
//! - the angle about y (`dword_635040`) from the clock: its low sixteen bits times 8,
//!   as sixteen bits; the angle about x (`dword_63503C`) = 0;
//! - y and z (`dword_635020`, `dword_635024`) = `dword_75A668`, `dword_75A66C`; the
//!   front entry (place 0) is brought forward by the high half of `dword_75A670` times
//!   the turn inverted, the next one (place 1) by that of the turn itself: the one
//!   leaving the front goes back as the one arriving comes forward;
//! - `dword_623FB0` and `dword_623FB4` = 0;
//! - the entry's kind `dword_75A674[entry]`, kept in `dword_75A658`; a kind above
//!   0x258 has a size, a y and an angle about x of its own in the records of twelve
//!   bytes at `unk_75950E` (by the kind less 0x259);
//! - `g_objectNumber` = the first object of that kind (`scene::find_object_of_kind`
//!   0x422298: objects 1 .. 0xFF, else 0).
//!
//! Then `scene::draw_menu_object`.
//! Five call sites in `scene::draw_carousel`.
//! Refuses an entry of 0x100 or more and a kind past the records.
//! Shadow mode runs the original alone and compares the variables it set with the
//! Rust arithmetic.
//! Self-test: a redirected copy whose drawing notes everything set, 240 cases by lot.

use crate::errors::{BadInput, Refused};
use crate::framework::{
    RawHook, Regs, SelfTest, call_game, game_addr, game_read, game_write, hook_asm, rva, shadow,
};
use crate::install::stand_ins::{act, set_handler, stand_in};
use crate::redirect::RedirectedCopy;
use crate::scene::draw_menu_object::draw_menu_object_in_game;
use crate::service::harness::{Region, at, capture, restore};
use std::sync::Mutex;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "scene::draw_carousel_entry",
    rva(0x4906D5),
    thunk,
    &ORIGINAL,
)
.self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

pub(crate) const PLACE_RVA: usize = rva(0x63501C); // dword_63501C, dword_635020, dword_635024
pub(crate) const SIZE_RVA: usize = rva(0x635038); // dword_635038
pub(crate) const ANGLE_X_RVA: usize = rva(0x63503C); // dword_63503C
pub(crate) const ANGLE_Y_RVA: usize = rva(0x635040); // dword_635040
const CLEARED_RVA: usize = rva(0x623FB0); // dword_623FB0, dword_623FB4
const SPACING_RVA: usize = rva(0x75A664); // dword_75A664, then y, z and the way forward
pub(crate) const KINDS_RVA: usize = rva(0x75A674); // dword_75A674
pub(crate) const ENTRIES: u32 = 0x100;
pub(crate) const KIND_RVA: usize = rva(0x75A658); // dword_75A658
pub(crate) const OWN_RVA: usize = rva(0x75950E); // unk_75950E
pub(crate) const OWN: u32 = 0x88;
const ANGLE_RVA: usize = rva(0x75AA80); // g_carouselAngle
pub(crate) const CLOCK_RVA: usize = rva(0x613830); // g_sceneClock
pub(crate) const OBJECT_RVA: usize = rva(0x6214FC); // g_objectNumber
pub(crate) const OBJECT_KINDS_RVA: usize = rva(0x5CD27C); // g_objectKinds

// ---- Rust version ----

/// What the function sets before it draws.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Settings {
    place: [u32; 3],
    /// None: left as it is.
    size: Option<u32>,
    angle_x: u32,
    angle_y: u32,
    kind: u32,
    object: u32,
}

pub(crate) use crate::scene::find_object_of_kind::find_object_of_kind;

/// Reads only.
unsafe fn settings(entry: u32, row: u32) -> Result<Settings, BadInput> {
    unsafe {
        if entry >= ENTRIES {
            return Err(BadInput("a carousel entry past the table"));
        }
        let [spacing, y, z, forward]: [u32; 4] = game_read(SPACING_RVA);
        let turn: u32 = game_read(ANGLE_RVA);
        let high = |a: u32, b: u32| ((a as u64 * b as u64) >> 32) as u32;
        let x = high(spacing, !turn)
            .wrapping_add((spacing as i32).wrapping_mul(row.wrapping_sub(1) as i32) as u32);
        let z = match row {
            0 => z.wrapping_sub(high(!turn, forward)),
            1 => z.wrapping_sub(high(turn, forward)),
            _ => z,
        };
        let clock: u32 = game_read(CLOCK_RVA);
        let mut set = Settings {
            place: [x, y, z],
            size: None,
            angle_x: 0,
            angle_y: ((clock as u16) << 3) as u32,
            kind: game_read(KINDS_RVA + entry as usize * 4),
            object: 0,
        };
        if set.kind as i32 > 0x258 {
            let own = set.kind - 0x259;
            if own >= OWN {
                return Err(BadInput("a kind past the records of the kinds' own places"));
            }
            let [size, y, angle]: [u32; 3] = game_read(OWN_RVA + own as usize * 12);
            set.size = Some(size);
            set.place[1] = y;
            set.angle_x = angle;
        }
        set.object = find_object_of_kind(set.kind);
        Ok(set)
    }
}

pub unsafe fn draw_carousel_entry(
    r: &Regs,
    draw: &mut dyn FnMut(&mut Regs) -> Result<(), Refused>,
) -> Result<(), Refused> {
    unsafe {
        let set = settings(r.ebx, r.ecx)?;
        game_write(PLACE_RVA, set.place);
        if let Some(size) = set.size {
            game_write(SIZE_RVA, size);
        }
        game_write(ANGLE_X_RVA, set.angle_x);
        game_write(ANGLE_Y_RVA, set.angle_y);
        game_write(CLEARED_RVA, [0u32; 2]);
        game_write(KIND_RVA, set.kind);
        game_write(OBJECT_RVA, set.object);
        // The registers the drawing is entered with; they are put back afterwards.
        let mut inner = Regs {
            eax: set.object,
            ..*r
        };
        draw(&mut inner)
    }
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        let said = unsafe { settings(r.ebx, r.ecx) }.ok();
        let (entry, row) = (r.ebx, r.ecx);
        unsafe { call_original(r) };
        // What the drawing under it does not write again (the object's number it may).
        let left = unsafe {
            (
                game_read::<[u32; 3]>(PLACE_RVA),
                game_read::<u32>(SIZE_RVA),
                game_read::<u32>(ANGLE_X_RVA),
                game_read::<u32>(ANGLE_Y_RVA),
                game_read::<u32>(KIND_RVA),
            )
        };
        if let Some(said) = said {
            let mine = (
                said.place,
                said.size.unwrap_or(left.1),
                said.angle_x,
                said.angle_y,
                said.kind,
            );
            if mine != left {
                HOOK.mismatch(|| {
                    format!("entry {entry:#x}, row {row:#x}: original {left:x?}; rust {mine:x?}")
                });
            }
        }
        return;
    }
    if let Err(e) = unsafe { draw_carousel_entry(r, &mut |r| draw_menu_object_in_game(r)) } {
        unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) }
    }
}

// ---- self-test ----

const LEN: usize = 0xE3;
const ENTRY_BYTES: [u8; 11] = [0x60, 0xC7, 0x05, 0x3C, 0x50, 0x63, 0x00, 0, 0, 0, 0];
const CALLS: [usize; 2] = [0xD2, 0xDC];

fn written() -> [Region; 7] {
    [
        (game_addr(PLACE_RVA) as u32, 12),
        at(SIZE_RVA),
        at(ANGLE_X_RVA),
        at(ANGLE_Y_RVA),
        (game_addr(CLEARED_RVA) as u32, 8),
        at(KIND_RVA),
        at(OBJECT_RVA),
    ]
}

/// Each drawing: everything set by then, and eax as it was entered with.
static CALLED: Mutex<Vec<(Vec<u8>, u32)>> = Mutex::new(Vec::new());

fn handler(_: u32, r: &mut Regs) {
    CALLED.lock().unwrap().push((capture(&written()), r.eax));
    r.eax = 0x100;
    r.ebx ^= 0x1234;
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let copy = unsafe {
        RedirectedCopy::new(
            game_addr(rva(0x4906D5)),
            LEN,
            &ENTRY_BYTES,
            &CALLS,
            &|callee| (callee == rva(0x4907E7)).then(|| stand_in(0)),
        )
    };
    let copy = match copy {
        Ok(copy) => copy,
        Err(why) => return t.skip(why),
    };
    let regions = written();
    const ENTRY: u32 = 0xF7;
    let inputs: [Region; 5] = [
        (game_addr(SPACING_RVA) as u32, 16),
        at(ANGLE_RVA),
        at(CLOCK_RVA),
        at(KINDS_RVA + ENTRY as usize * 4),
        at(KINDS_RVA),
    ];
    let saved = (capture(&regions), capture(&inputs));
    let mut seed = 0x0CA4_0D5Eu32;
    let mut lot = |below: u32| {
        seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        (seed >> 8) % below
    };
    // Kinds the game has objects of, where it has any, among the ones tried.
    let loaded: Vec<u32> = (1..0x100usize)
        .map(|n| unsafe { game_read::<u32>(OBJECT_KINDS_RVA + n * 4) })
        .filter(|&kind| kind != 0)
        .take(4)
        .collect();
    for case in 0..240 {
        let entry = [0u32, ENTRY][lot(2) as usize];
        let row = [0xFFFF_FFFFu32, 0, 1, 2, 7][lot(5) as usize];
        let spacing = [0x0002_0000u32, 0xFFFE_0000, 0x1234_5678, 0][lot(4) as usize];
        let y = lot(0x10_0000);
        let z = lot(0x100_0000);
        let forward = [0x0008_0000u32, 0xFFFF_FFFF, 0, 0x8000_0000][lot(4) as usize];
        let turn = [0u32, 0xFFFF_FFFF, 0x8000_0000, 0x4000_0001, 0xC123_4567][lot(5) as usize];
        let clock = lot(0x00FF_FFFF) << 4 | lot(16);
        let kind = match lot(8) {
            0 => 0x258,
            1 => 0x259,
            2 => 0x259 + OWN - 1,
            3 => 0x25A + lot(8),
            4 => 0xFFFF_FFFF,
            n => loaded.get(n as usize - 5).copied().unwrap_or(0x230 + n),
        };
        let run = |rust: bool| unsafe {
            restore(&regions, &vec![0xAA; saved.0.len()]);
            game_write(SPACING_RVA, [spacing, y, z, forward]);
            game_write(ANGLE_RVA, turn);
            game_write(CLOCK_RVA, clock);
            game_write(KINDS_RVA + entry as usize * 4, kind);
            CALLED.lock().unwrap().clear();
            set_handler(handler);
            let before = Regs {
                ebx: entry,
                ecx: row,
                ..Regs::SENTINEL
            };
            let mut r = before;
            let kept = if rust {
                draw_carousel_entry(&r, &mut |r| {
                    act(0, r);
                    Ok(())
                })
                .is_ok()
            } else {
                call_game(&mut r, copy.addr());
                r.preserved_except(&before, 0)
            };
            (kept, capture(&regions), CALLED.lock().unwrap().clone())
        };
        let (theirs, mine) = (run(false), run(true));
        t.check(theirs == mine && theirs.0 && theirs.2.len() == 1, || {
            format!(
                "case {case}: entry {entry:#x}, row {row:#x}, spacing {spacing:#x}, y {y:#x}, z {z:#x}, forward {forward:#x}, turn {turn:#x}, clock {clock:#x}, kind {kind:#x}: original {theirs:x?}; rust {mine:x?}"
            )
        });
    }
    restore(&regions, &saved.0);
    restore(&inputs, &saved.1);
    t.finish()
}
