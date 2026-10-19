//! `gfx::find_3d_cards` (0x404270): which display cards can draw in 3D.
//!
//! In:  nothing
//! Out: every register preserved (`pusha` .. `popa`)
//! `dword_5AB4AC` (a 3D primary card) and `dword_5AB4B0` (a 3D secondary card) 0; unless
//! `dword_5AB94C` (a setting) is not 0: "Finding 3d hardware" (log level 1, a line), then
//! `DirectDrawEnumerateA(Callback, 0)`: the game's `Callback` (0x4042AE) makes DirectDraw
//! for each card, logs " dd:" its description and name, and for one with 3D ("3d:") sets
//! "primary card" (`dword_5AB4AC` 1, unless `dword_5AB950`) or "secondary card"
//! (`dword_5AB4B0` 1 and its GUID to `unk_5AB4B4`, unless `dword_5AB954`). Then a line
//! break in the log. The enumeration (and the callback, DirectDraw throughout) is the
//! platform's: in the game through the import's thunk (0x4D5E96); in the standalone program
//! nothing (no 3D card is found: its 3D is its platform's). One caller, `start` (0x401093).
//! Shadow mode runs the original alone (DirectDraw is made once per card). Self-test: a
//! copy whose three variables are the test's and whose calls go to stand-ins (the
//! enumeration's notes its arguments), against the Rust version, the setting 0 and not.

use crate::errors::Refused;
use crate::framework::{
    RawHook, Regs, SelfTest, call_game, game_addr, game_read, game_write, hook_asm, rva, shadow,
    standalone,
};
use crate::install::stand_ins::{act, set_handler, stand_in};
use std::sync::Mutex;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("gfx::find_3d_cards", rva(0x404270), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

const FINDING_TEXT_RVA: usize = rva(0x5C2CF7); // "Finding 3d hardware"
const CALLBACK_RVA: usize = rva(0x4042AE); // Callback
const ENUMERATE: usize = rva(0x4D5E96); // DirectDrawEnumerateA's thunk
/// The calls by place: 0 `logfile::log_write_l1_nl` (eax), 1 the enumeration, 2
/// `logfile::log_newline_l1`.
const CALLEES: [usize; 2] = [0x40142B, 0x4013DE];

pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;

/// Where the variables are (the game's, or a test's).
#[derive(Clone, Copy)]
pub struct FindVars {
    pub primary_3d: u32,
    pub secondary_3d: u32,
    /// dword_5AB94C: not 0, no search
    pub no_search: u32,
}

impl FindVars {
    pub fn game() -> Self {
        let a = |x: usize| game_addr(rva(x)) as u32;
        Self {
            primary_3d: a(0x5AB4AC),
            secondary_3d: a(0x5AB4B0),
            no_search: a(0x5AB94C),
        }
    }
}

// ---- Rust version ----

pub unsafe fn find_3d_cards(entry: &Regs, v: &FindVars, part: Part) -> Result<(), Refused> {
    unsafe {
        (v.primary_3d as *mut u32).write_unaligned(0);
        (v.secondary_3d as *mut u32).write_unaligned(0);
        if (v.no_search as *const u32).read_unaligned() == 0 {
            part(
                0,
                &mut Regs {
                    eax: game_addr(FINDING_TEXT_RVA) as u32,
                    ..*entry
                },
            )?;
            let mut r = *entry;
            part(1, &mut r)?;
        }
        let mut r = *entry;
        part(2, &mut r)
    }
}

/// In the game: the logs and the enumeration through the game; in the standalone program
/// the enumeration does nothing.
pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    match place {
        // DirectDraw's drivers: the standalone program's is its rasteriser, a 3D primary
        // card (as the enumeration's callback sets it, unless `dword_5AB950` says not to).
        1 if standalone() => unsafe {
            if game_read::<u32>(rva(0x5AB950)) == 0 {
                game_write(rva(0x5AB4AC), 1u32);
            }
        },
        1 => unsafe {
            let f: unsafe extern "system" fn(u32, u32) -> i32 =
                std::mem::transmute(game_addr(ENUMERATE));
            f(game_addr(CALLBACK_RVA) as u32, 0);
        },
        _ => unsafe { call_game(r, game_addr(rva(CALLEES[(place / 2) as usize]))) },
    }
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
    if let Err(e) = unsafe { find_3d_cards(&entry, &FindVars::game(), &mut game_part) } {
        unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) };
    }
}

// ---- self-test ----

/// The calls noted: (place, eax or the enumeration's callback, its context).
static NOTED: Mutex<Vec<(u32, u32, u32)>> = Mutex::new(Vec::new());

fn note(place: u32, r: &mut Regs) {
    let eax = if place == 0 { r.eax } else { 0 };
    NOTED.lock().unwrap().push((place, eax, 0));
}

extern "system" fn note_enumerate(callback: u32, context: u32) -> i32 {
    NOTED.lock().unwrap().push((1, callback, context));
    0
}

fn test_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    if place == 1 {
        note_enumerate(game_addr(CALLBACK_RVA) as u32, 0);
        return Ok(());
    }
    act(place, r);
    Ok(())
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let mut m = vec![0u32; 3];
    let at = m.as_mut_ptr() as u32;
    let v = FindVars {
        primary_3d: at,
        secondary_3d: at + 4,
        no_search: at + 8,
    };
    let c = match crate::audio::test_copy::make(
        &HOOK,
        &crate::audio::test_copy::Copy {
            address: 0x404270,
            len: 0x3E,
            entry: &[
                0x60, 0xC7, 0x05, 0xAC, 0xB4, 0x5A, 0x00, 0x00, 0x00, 0x00, 0x00,
            ],
            calls: &[0x25, 0x32, 0x37],
            jumps_out: &[],
        },
        &|callee| {
            if callee == ENUMERATE {
                return Some(note_enumerate as *const () as usize);
            }
            match callee {
                c if c == rva(CALLEES[0]) => Some(stand_in(0)),
                c if c == rva(CALLEES[1]) => Some(stand_in(2)),
                _ => None,
            }
        },
        &[
            (0x5AB4AC, v.primary_3d),
            (0x5AB4B0, v.secondary_3d),
            (0x5AB94C, v.no_search),
        ],
        3,
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    set_handler(note);
    for no_search in [0u32, 1, 0x8000_0000] {
        let run = |rust: bool| unsafe {
            let p = at as *mut u32;
            p.write_volatile(0xAAAA_AAAA);
            p.add(1).write_volatile(0xAAAA_AAAA);
            p.add(2).write_volatile(no_search);
            NOTED.lock().unwrap().clear();
            let before = Regs::SENTINEL;
            let (done, preserved) = if rust {
                (find_3d_cards(&before, &v, &mut test_part).is_ok(), true)
            } else {
                let mut ro = before;
                call_game(&mut ro, c.addr());
                (true, ro.preserved_except(&before, 0))
            };
            (
                done && preserved,
                std::mem::take(&mut *NOTED.lock().unwrap()),
                [p.read_volatile(), p.add(1).read_volatile()],
            )
        };
        let theirs = run(false);
        let mine = run(true);
        t.check(theirs == mine, || {
            format!("setting {no_search:#x}: original {theirs:x?}; rust {mine:x?}")
        });
    }
    drop(m);
    t.finish()
}
