//! `gfx::clear_front_surface` (0x4059F4): the front (shown) surface made black.
//!
//! In:  nothing
//! Out: every register preserved (`pusha` .. `popa`)
//! Nothing without `g_frontSurface`. The surface locked (`sub_405996`: DirectDraw's `Lock`
//! into the description at `dword_4F3400`, tried again while the surface is lost; failing,
//! a log line and the carry): nothing more. Else `g_screenHeight` rows of
//! `g_screenRowDwords` zero dwords from the description's memory (`dword_4F3424`), each
//! row `dword_4F3410` (the pitch) on from the last, then unlocked (`sub_4059D5`: `Unlock`;
//! failing, fatal error 0x80208272, raised there). The lock and unlock are the platform's
//! (DirectDraw), through the game; in the standalone program the lock fails (its front
//! surface is SDL's window: `standalone.rs` would need a lock of its own for this to clear
//! it), so nothing is written. Refuses (before anything) a height of 0: the original's
//! `dec edx; jnz` would clear 2^32 rows. Six callers (the screen mode's set-up, the films,
//! the menus). Shadow mode runs the original alone (the lock is made once). Self-test: a
//! copy whose six variables are the test's and whose lock and unlock go to stand-ins (the
//! lock failing or not), on a buffer of the test's, against the Rust version.

use crate::errors::{BadInput, Refused};
use crate::framework::{
    RawHook, Regs, SelfTest, call_game, game_addr, hook_asm, rva, shadow, standalone,
};
use crate::install::stand_ins::{act, set_handler, stand_in};
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("gfx::clear_front_surface", rva(0x4059F4), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

/// The calls by place: 0 the lock (carry: failed), 1 the unlock.
const CALLEES: [usize; 2] = [0x405996, 0x4059D5];
pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;

/// Where the variables are (the game's, or a test's).
#[derive(Clone, Copy)]
pub struct FrontVars {
    /// g_frontSurface
    pub front: u32,
    /// dword_4F3424: the locked memory
    pub memory: u32,
    /// dword_4F3410: the pitch
    pub pitch: u32,
    /// g_screenRowBytes
    pub row_bytes: u32,
    /// g_screenHeight
    pub height: u32,
    /// g_screenRowDwords
    pub row_dwords: u32,
}

impl FrontVars {
    pub fn game() -> Self {
        let a = |x: usize| game_addr(rva(x)) as u32;
        Self {
            front: a(0x4F32B4),
            memory: a(0x4F3424),
            pitch: a(0x4F3410),
            row_bytes: a(0x4F495C),
            height: a(0x4F4950),
            row_dwords: a(0x4F4960),
        }
    }
}

// ---- Rust version ----

unsafe fn get(at: u32) -> u32 {
    unsafe { (at as *const u32).read_unaligned() }
}

/// The rows cleared: `rows` of `dwords` zero dwords from `memory`, each row's start `gap`
/// bytes after the last one's end.
pub unsafe fn clear_rows(memory: u32, dwords: u32, gap: u32, rows: u32) {
    let mut at = memory;
    for _ in 0..rows {
        for k in 0..dwords {
            unsafe { (at.wrapping_add(4 * k) as *mut u32).write_unaligned(0) };
        }
        at = at.wrapping_add(4 * dwords).wrapping_add(gap);
    }
}

pub unsafe fn clear_front_surface(entry: &Regs, v: &FrontVars, part: Part) -> Result<(), Refused> {
    unsafe {
        if get(v.front) == 0 {
            return Ok(());
        }
        let rows = get(v.height);
        if rows == 0 {
            return Err(BadInput("a screen height of 0 (2^32 rows)").into());
        }
        let mut r = *entry;
        part(0, &mut r)?;
        if r.carry() {
            return Ok(());
        }
        let gap = get(v.pitch).wrapping_sub(get(v.row_bytes));
        clear_rows(get(v.memory), get(v.row_dwords), gap, rows);
        let mut r = *entry;
        part(1, &mut r)?;
        Ok(())
    }
}

/// In the game: the lock and unlock through the game (DirectDraw). In the standalone
/// program the lock fails.
pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    if standalone() {
        r.set_carry(place == 0);
        return Ok(());
    }
    unsafe { call_game(r, game_addr(rva(CALLEES[place as usize]))) };
    Ok(())
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    let entry = *r;
    if let Err(e) = unsafe { clear_front_surface(&entry, &FrontVars::game(), &mut game_part) } {
        unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) };
    }
}

// ---- self-test ----

static NOTED: Mutex<Vec<u32>> = Mutex::new(Vec::new());
static LOCK_FAILS: AtomicBool = AtomicBool::new(false);

fn note(place: u32, r: &mut Regs) {
    NOTED.lock().unwrap().push(place);
    if place == 0 {
        r.set_carry(LOCK_FAILS.load(Ordering::SeqCst));
    }
}

fn test_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    act(place, r);
    Ok(())
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let mut vars = vec![0u32; 6];
    let at = vars.as_mut_ptr() as u32;
    let v = FrontVars {
        front: at,
        memory: at + 4,
        pitch: at + 8,
        row_bytes: at + 12,
        height: at + 16,
        row_dwords: at + 20,
    };
    let c = match crate::audio::test_copy::make(
        &HOOK,
        &crate::audio::test_copy::Copy {
            address: 0x4059F4,
            len: 0x3F,
            entry: &[0x60, 0x83, 0x3D, 0xB4, 0x32, 0x4F, 0x00, 0x00],
            calls: &[0x0A, 0x38],
            jumps_out: &[],
        },
        &|callee| {
            CALLEES
                .iter()
                .position(|&c| rva(c) == callee)
                .map(|n| stand_in(n as u32))
        },
        &[
            (0x4F32B4, v.front),
            (0x4F3424, v.memory),
            (0x4F3410, v.pitch),
            (0x4F495C, v.row_bytes),
            (0x4F4950, v.height),
            (0x4F4960, v.row_dwords),
        ],
        6,
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    set_handler(note);
    let mut buffer = vec![0xAAu8; 0x400];
    let memory = buffer.as_mut_ptr() as u32 + 0x10;
    // (front, lock fails, row dwords, row bytes, pitch, height)
    let cases = [
        (0u32, false, 4u32, 16u32, 16u32, 4u32),
        (1, true, 4, 16, 16, 4),
        (1, false, 4, 16, 16, 4),
        (1, false, 4, 16, 24, 5),
        (1, false, 5, 20, 32, 3),
        (1, false, 0, 0, 8, 6),
        (1, false, 6, 22, 30, 1),
        (1, false, 3, 12, 8, 4),
    ];
    for (front, fails, dwords, row_bytes, pitch, height) in cases {
        LOCK_FAILS.store(fails, Ordering::SeqCst);
        let run = |rust: bool| unsafe {
            let p = at as *mut u32;
            for (n, x) in [front, memory, pitch, row_bytes, height, dwords]
                .into_iter()
                .enumerate()
            {
                p.add(n).write_volatile(x);
            }
            let b = memory as *mut u8;
            std::ptr::write_bytes(b.sub(0x10), 0xAA, 0x400);
            NOTED.lock().unwrap().clear();
            let before = Regs::SENTINEL;
            let (done, preserved) = if rust {
                (
                    clear_front_surface(&before, &v, &mut test_part).is_ok(),
                    true,
                )
            } else {
                let mut ro = before;
                call_game(&mut ro, c.addr());
                (true, ro.preserved_except(&before, 0))
            };
            (
                done && preserved,
                std::mem::take(&mut *NOTED.lock().unwrap()),
                std::slice::from_raw_parts(b.sub(0x10), 0x400).to_vec(),
            )
        };
        let theirs = run(false);
        let mine = run(true);
        t.check(theirs == mine, || {
            format!(
                "front {front}, lock fails {fails}, {dwords} dwords, row {row_bytes}, pitch {pitch}, height {height}: original {} {:?}; rust {} {:?}; buffers {}",
                theirs.0,
                theirs.1,
                mine.0,
                mine.1,
                crate::service::harness::diff_summary(&theirs.2, &mine.2)
            )
        });
    }
    drop(buffer);
    drop(vars);
    t.finish()
}
