//! `gfx::copy_picture_to_surface` (0x4060E3): copies the picture buffer onto the surface, row
//! by row: the background of a menu frame.
//!
//! In:  nothing
//! Out: every register preserved
//! If the surface is not locked (`dword_4F3870` is 0) it is locked first, and nothing is
//! copied if that fails; it is then unlocked again afterwards. The copy takes
//! `dword_4F49A4` rows of `dword_4F49B4` dwords from the buffer, which it reads without
//! gaps, and after each row moves on `dword_4F49C4` more bytes on the surface. All of these
//! are read after the lock, which sets them (`sub_405726`).
//! Callers: `menu::frame_callback`, `menu::draw_frame` and a dozen more.
//! The lock and the unlock are the game's (`gfx::lock_surface` 0x4058AD,
//! `gfx::unlock_surface` 0x405949, DirectDraw calls); the Rust version reaches them through
//! `PictureSurface`, which is where a build without the game puts its own surface.
//! Shadow mode runs the original alone and, when the surface was locked at entry and so
//! still is, compares what is on it with what the Rust version copies.
//! Refuses a row count of 0 (the original counts through all 32 bits) and null pointers.
//! The name is a reading.

use crate::errors::BadInput;
use crate::framework::{
    RawHook, Regs, SelfTest, call_game, game_addr, game_read, hook_asm, log, rva, shadow,
};
use crate::gfx::picture::picture;
use crate::gfx::surface::BASE_RVA;
use crate::redirect::RedirectedCopy;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicUsize, Ordering};

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "gfx::copy_picture_to_surface",
    rva(0x4060E3),
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

const LOCKED_RVA: usize = rva(0x4F3870); // dword_4F3870: 1 while the surface is locked
const ROWS_RVA: usize = rva(0x4F49A4); // dword_4F49A4
const ROW_DWORDS_RVA: usize = rva(0x4F49B4); // dword_4F49B4
const ROW_GAP_RVA: usize = rva(0x4F49C4); // dword_4F49C4: surface bytes after a row's pixels
const LOCK_SURFACE: usize = rva(0x4058AD); // gfx::lock_surface
const UNLOCK_SURFACE: usize = rva(0x405949); // gfx::unlock_surface

// ---- Rust version ----

/// The surface the picture goes to, and the picture.
pub trait PictureSurface {
    fn locked(&self) -> bool;
    /// Returns whether the surface is now locked.
    fn lock(&mut self) -> bool;
    fn unlock(&mut self);
    /// What the copy reads, valid once the surface is locked.
    fn picture(&self) -> *const u8;
    fn pixels(&self) -> *mut u8;
    fn rows(&self) -> u32;
    fn row_dwords(&self) -> u32;
    fn row_gap(&self) -> u32;
}

/// The game's own.
pub struct GameSurface;

impl PictureSurface for GameSurface {
    fn locked(&self) -> bool {
        unsafe { game_read::<u32>(LOCKED_RVA) != 0 }
    }
    fn lock(&mut self) -> bool {
        let mut g = Regs::default();
        unsafe { call_game(&mut g, game_addr(LOCK_SURFACE)) };
        !g.carry()
    }
    fn unlock(&mut self) {
        let mut g = Regs::default();
        unsafe { call_game(&mut g, game_addr(UNLOCK_SURFACE)) };
    }
    fn picture(&self) -> *const u8 {
        picture()
    }
    fn pixels(&self) -> *mut u8 {
        unsafe { game_read::<u32>(BASE_RVA) as *mut u8 }
    }
    fn rows(&self) -> u32 {
        unsafe { game_read(ROWS_RVA) }
    }
    fn row_dwords(&self) -> u32 {
        unsafe { game_read(ROW_DWORDS_RVA) }
    }
    fn row_gap(&self) -> u32 {
        unsafe { game_read(ROW_GAP_RVA) }
    }
}

pub unsafe fn copy_picture_to_surface(surface: &mut dyn PictureSurface) -> Result<(), BadInput> {
    let was_locked = surface.locked();
    if !was_locked && !surface.lock() {
        return Ok(());
    }
    let (mut from, mut to) = (surface.picture(), surface.pixels());
    let (rows, row_bytes) = (surface.rows(), surface.row_dwords() as usize * 4);
    let refused = if from.is_null() || to.is_null() {
        Some(BadInput("no picture buffer or no surface"))
    } else if rows == 0 {
        Some(BadInput("a surface without rows"))
    } else {
        None
    };
    if let Some(why) = refused {
        // Nothing is written yet; the surface is left as it was found.
        if !was_locked {
            surface.unlock();
        }
        return Err(why);
    }
    let gap = surface.row_gap() as usize;
    for _ in 0..rows {
        unsafe {
            std::ptr::copy(from, to, row_bytes);
            from = from.add(row_bytes);
            to = to.wrapping_add(row_bytes).wrapping_add(gap);
        }
    }
    if !was_locked {
        surface.unlock();
    }
    Ok(())
}

// ---- hook ----

/// The number of the first row of a locked surface that is not the picture's.
unsafe fn first_wrong_row(surface: &dyn PictureSurface) -> Option<u32> {
    let row_bytes = surface.row_dwords() as usize * 4;
    let step = row_bytes.wrapping_add(surface.row_gap() as usize);
    (0..surface.rows()).find(|&row| unsafe {
        let from = surface.picture().add(row as usize * row_bytes);
        let to = surface.pixels().wrapping_add(row as usize * step);
        std::slice::from_raw_parts(from, row_bytes) != std::slice::from_raw_parts(to, row_bytes)
    })
}

static SAID_COMPARED: AtomicBool = AtomicBool::new(false);
static SAID_NOT_COMPARED: AtomicBool = AtomicBool::new(false);

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if !shadow() {
        if let Err(e) = unsafe { copy_picture_to_surface(&mut GameSurface) } {
            unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) };
        }
        return;
    }
    let was_locked = GameSurface.locked();
    let mut o = *r;
    unsafe { call_original(&mut o) };
    // Unlocked again, the surface's memory is no longer ours to read.
    let surface = GameSurface;
    let comparable = was_locked
        && !crate::gfx::picture::IN_A_TEST.load(Ordering::SeqCst)
        && surface.locked()
        && !surface.picture().is_null()
        && !surface.pixels().is_null();
    // Once a run each, for hook.log: that calls of both kinds have been seen.
    let said = if comparable {
        &SAID_COMPARED
    } else {
        &SAID_NOT_COMPARED
    };
    if !said.swap(true, Ordering::SeqCst) {
        log(&format!(
            "gfx::copy_picture_to_surface: a shadow call that is {} (locked at entry: {was_locked})",
            if comparable {
                "compared"
            } else {
                "not compared"
            }
        ));
    }
    let wrong = if comparable {
        unsafe { first_wrong_row(&surface) }
    } else {
        None
    };
    if wrong.is_some() || !o.preserved_except(r, 0) {
        HOOK.mismatch(|| {
            format!("after the original, row {wrong:?} of the surface is not the picture's")
        });
    }
}

// ---- self-test: a copy of the original whose lock and unlock are stand-ins and whose six
// variables are the test's, copying between two buffers of the test ----

static T_LOCKED: AtomicU32 = AtomicU32::new(0);
static T_PICTURE: AtomicU32 = AtomicU32::new(0);
static T_PIXELS: AtomicU32 = AtomicU32::new(0);
static T_ROWS: AtomicU32 = AtomicU32::new(0);
static T_ROW_DWORDS: AtomicU32 = AtomicU32::new(0);
static T_ROW_GAP: AtomicU32 = AtomicU32::new(0);
/// What the lock stand-in answers: 1 locks, 0 fails.
static T_LOCK_WORKS: AtomicU32 = AtomicU32::new(0);
static T_LOCKS: AtomicU32 = AtomicU32::new(0);
static T_UNLOCKS: AtomicU32 = AtomicU32::new(0);

/// As the original: carry set when the surface is not locked afterwards.
#[unsafe(naked)]
unsafe extern "C" fn lock_stand_in() {
    core::arch::naked_asm!(
        "inc dword ptr [{locks}]",
        "cmp dword ptr [{works}], 1",
        "ret",
        locks = sym T_LOCKS,
        works = sym T_LOCK_WORKS,
    )
}

#[unsafe(naked)]
unsafe extern "C" fn unlock_stand_in() {
    core::arch::naked_asm!(
        "inc dword ptr [{unlocks}]",
        "ret",
        unlocks = sym T_UNLOCKS,
    )
}

struct TestSurface;

impl PictureSurface for TestSurface {
    fn locked(&self) -> bool {
        T_LOCKED.load(Ordering::SeqCst) != 0
    }
    fn lock(&mut self) -> bool {
        T_LOCKS.fetch_add(1, Ordering::SeqCst);
        T_LOCK_WORKS.load(Ordering::SeqCst) == 1
    }
    fn unlock(&mut self) {
        T_UNLOCKS.fetch_add(1, Ordering::SeqCst);
    }
    fn picture(&self) -> *const u8 {
        T_PICTURE.load(Ordering::SeqCst) as *const u8
    }
    fn pixels(&self) -> *mut u8 {
        T_PIXELS.load(Ordering::SeqCst) as *mut u8
    }
    fn rows(&self) -> u32 {
        T_ROWS.load(Ordering::SeqCst)
    }
    fn row_dwords(&self) -> u32 {
        T_ROW_DWORDS.load(Ordering::SeqCst)
    }
    fn row_gap(&self) -> u32 {
        T_ROW_GAP.load(Ordering::SeqCst)
    }
}

const LEN: usize = 0x43;
const ENTRY_BYTES: [u8; 8] = [0x60, 0x8B, 0x2D, 0x70, 0x38, 0x4F, 0x00, 0x85];
const CALLS: [usize; 2] = [0x0B, 0x3C];

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let redirect = |target: usize| {
        Some(if target == LOCK_SURFACE {
            lock_stand_in as *const () as usize
        } else {
            unlock_stand_in as *const () as usize
        })
    };
    let copy = match unsafe {
        RedirectedCopy::new(
            game_addr(rva(0x4060E3)),
            LEN,
            &ENTRY_BYTES,
            &CALLS,
            &redirect,
        )
    } {
        Ok(copy) => copy,
        Err(why) => return t.skip(why),
    };
    // Where the copy names the six variables.
    let operands: [(usize, &AtomicU32); 6] = [
        (0x03, &T_LOCKED),
        (0x14, &T_PICTURE),
        (0x1A, &T_PIXELS),
        (0x1F, &T_ROW_GAP),
        (0x25, &T_ROW_DWORDS),
        (0x2B, &T_ROWS),
    ];
    for (operand, variable) in operands {
        unsafe { copy.patch_u32(operand, variable as *const AtomicU32 as u32) };
    }

    // (rows, dwords in a row, gap)
    for (rows, row_dwords, gap) in [(1u32, 1u32, 0u32), (4, 3, 8), (5, 0, 4), (3, 7, 0)] {
        // (locked at entry, the lock works)
        for (locked, lock_works) in [(1u32, 1u32), (0, 1), (0, 0)] {
            let picture: Vec<u8> = (0..rows * row_dwords * 4)
                .map(|n| (n * 7 + 1) as u8)
                .collect();
            let surface_len = (rows * (row_dwords * 4 + gap)) as usize;
            let run = |original: bool| {
                let mut pixels = vec![0xAAu8; surface_len + 16];
                T_LOCKED.store(locked, Ordering::SeqCst);
                T_LOCK_WORKS.store(lock_works, Ordering::SeqCst);
                T_PICTURE.store(picture.as_ptr() as u32, Ordering::SeqCst);
                T_PIXELS.store(pixels.as_mut_ptr() as u32, Ordering::SeqCst);
                T_ROWS.store(rows, Ordering::SeqCst);
                T_ROW_DWORDS.store(row_dwords, Ordering::SeqCst);
                T_ROW_GAP.store(gap, Ordering::SeqCst);
                T_LOCKS.store(0, Ordering::SeqCst);
                T_UNLOCKS.store(0, Ordering::SeqCst);
                let before = Regs::SENTINEL;
                let ok = if original {
                    let mut ro = before;
                    unsafe { call_game(&mut ro, copy.addr()) };
                    ro.preserved_except(&before, 0)
                } else {
                    unsafe { copy_picture_to_surface(&mut TestSurface) }.is_ok()
                };
                (
                    ok,
                    pixels,
                    T_LOCKS.load(Ordering::SeqCst),
                    T_UNLOCKS.load(Ordering::SeqCst),
                )
            };
            let theirs = run(true);
            let mine = run(false);
            t.check(theirs == mine && theirs.0, || {
                format!(
                    "rows={rows} row_dwords={row_dwords} gap={gap} locked={locked} lock_works={lock_works}: original ok={} locks={} unlocks={}; rust ok={} locks={} unlocks={} same pixels={}",
                    theirs.0,
                    theirs.2,
                    theirs.3,
                    mine.0,
                    mine.2,
                    mine.3,
                    theirs.1 == mine.1
                )
            });
        }
    }
    t.finish()
}
