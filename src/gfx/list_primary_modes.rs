//! `gfx::list_primary_modes` (0x4043DA): the primary card's screen modes listed into the
//! modes' table and marked for Direct3D. (`gfx::list_secondary_modes` is the same for the
//! secondary card; the two share what is here.)
//!
//! In:  nothing
//! Out: every register preserved (`pusha` .. `popa`)
//! The primary: `sub_4048EC` with eax 0 (DirectDraw made for the card, "VID: Open ddraw";
//! failing, fatal errors 0x80208511 / 0x80208512, raised there), `sub_404A31` with eax
//! 0x1000 (DirectDraw's `EnumDisplayModes` with `sub_404A56`, which puts the modes in the
//! table), `gfx::mark_3d_modes` with eax 0x1000 if `dword_5AB4AC` (a 3D primary card) is
//! 1, then `sub_404935` (DirectDraw closed). The DirectDraw calls are the platform's, through the game; in the standalone program
//! they do nothing (its screen modes are its platform's: `standalone.rs`). One caller,
//! `start` (0x401098). Manual patch with a private copy (0x27 bytes, the four calls
//! relocated): a call starts at +3. Shadow mode runs the original alone (DirectDraw is
//! opened once). Self-test: a copy whose variables are the test's and whose four calls go
//! to stand-ins noting eax, against the Rust version, both switches.

use crate::errors::Refused;
use crate::framework::{
    RawHook, Regs, SelfTest, call_game, game_addr, hook_asm, rva, shadow, standalone,
};
use crate::gfx::mark_3d_modes::{ModeVars, mark_3d_modes};
use crate::install::stand_ins::{act, set_handler, stand_in};
use std::sync::Mutex;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new("gfx::list_primary_modes", rva(0x4043DA), thunk, &ORIGINAL)
    .private_copy(0x27)
    .relocate_calls(&[0x03, 0x0D, 0x1B, 0x20])
    .manual_patch()
    .self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

/// The calls by place: 0 `sub_4048EC` (eax the GUID or 0), 1 `sub_404A31` (eax the kind),
/// 2 `gfx::mark_3d_modes` (eax the kind), 3 `sub_404935`.
const CALLEES: [usize; 4] = [0x4048EC, 0x404A31, 0x40442C, 0x404935];
pub const PRIMARY: u32 = 0x1000;

pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;

/// Where the variables are (the game's, or a test's).
#[derive(Clone, Copy)]
pub struct CardVars {
    /// dword_5AB4AC: a 3D primary card
    pub primary_3d: u32,
    /// dword_5AB4B0: a 3D secondary card
    pub secondary_3d: u32,
    /// unk_5AB4B4: the secondary card's GUID
    pub secondary_guid: u32,
}

impl CardVars {
    pub fn game() -> Self {
        let a = |x: usize| game_addr(rva(x)) as u32;
        Self {
            primary_3d: a(0x5AB4AC),
            secondary_3d: a(0x5AB4B0),
            secondary_guid: a(0x5AB4B4),
        }
    }
}

// ---- Rust version ----

pub unsafe fn get(at: u32) -> u32 {
    unsafe { (at as *const u32).read_unaligned() }
}

/// Each call with the entry's registers and eax as given (the callees keep them all).
pub fn call(entry: &Regs, place: u32, eax: u32, part: Part) -> Result<(), Refused> {
    part(place, &mut Regs { eax, ..*entry })
}

pub unsafe fn list_primary_modes(entry: &Regs, v: &CardVars, part: Part) -> Result<(), Refused> {
    call(entry, 0, 0, part)?;
    call(entry, 1, PRIMARY, part)?;
    if unsafe { get(v.primary_3d) } == 1 {
        call(entry, 2, PRIMARY, part)?;
    }
    call(entry, 3, PRIMARY, part)
}

/// In the game: DirectDraw through the game, the marking from the Rust version; in the
/// standalone program DirectDraw does nothing.
pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    if place == 2 {
        return unsafe {
            mark_3d_modes(
                r,
                &ModeVars::game(),
                &mut crate::gfx::mark_3d_modes::game_part,
            )
        };
    }
    if !standalone() {
        unsafe { call_game(r, game_addr(rva(CALLEES[place as usize]))) };
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
    if let Err(e) = unsafe { list_primary_modes(&entry, &CardVars::game(), &mut game_part) } {
        unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) };
    }
}

// ---- self-test ----

/// The calls noted: (place, eax).
pub static NOTED: Mutex<Vec<(u32, u32)>> = Mutex::new(Vec::new());

pub fn note(place: u32, r: &mut Regs) {
    NOTED.lock().unwrap().push((place, r.eax));
}

pub fn test_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    act(place, r);
    Ok(())
}

pub type Rust = unsafe fn(&Regs, &CardVars, Part) -> Result<(), Refused>;

/// Both switches, the copy against `rust`.
pub fn compare(
    t: &mut SelfTest,
    hook: &RawHook,
    copy: (usize, usize, &[u8], &[usize], usize),
    rust: Rust,
) {
    let mut m = vec![0u32; 6];
    let at = m.as_mut_ptr() as u32;
    let v = CardVars {
        primary_3d: at,
        secondary_3d: at + 4,
        secondary_guid: at + 8,
    };
    let (address, len, entry, calls, references) = copy;
    let c = match crate::audio::test_copy::make(
        hook,
        &crate::audio::test_copy::Copy {
            address,
            len,
            entry,
            calls,
            jumps_out: &[],
        },
        &|callee| {
            CALLEES
                .iter()
                .position(|&c| rva(c) == callee)
                .map(|n| stand_in(n as u32))
        },
        &[
            (0x5AB4AC, v.primary_3d),
            (0x5AB4B0, v.secondary_3d),
            (0x5AB4B4, v.secondary_guid),
        ],
        references,
    ) {
        Ok(c) => c,
        Err(why) => {
            t.check(false, || format!("no copy: {why}"));
            return;
        }
    };
    set_handler(note);
    for primary in [0u32, 1, 2] {
        for secondary in [0u32, 1, 2] {
            let run = |mine: bool| unsafe {
                let p = at as *mut u32;
                p.write_volatile(primary);
                p.add(1).write_volatile(secondary);
                NOTED.lock().unwrap().clear();
                let before = Regs::SENTINEL;
                let (done, preserved) = if mine {
                    (rust(&before, &v, &mut test_part).is_ok(), true)
                } else {
                    let mut ro = before;
                    call_game(&mut ro, c.addr());
                    (true, ro.preserved_except(&before, 0))
                };
                (
                    done && preserved,
                    std::mem::take(&mut *NOTED.lock().unwrap()),
                )
            };
            let theirs = run(false);
            let ours = run(true);
            t.check(theirs == ours, || {
                format!(
                    "primary {primary}, secondary {secondary}: original {theirs:x?}; rust {ours:x?}"
                )
            });
        }
    }
    drop(m);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    compare(
        &mut t,
        &HOOK,
        (
            0x4043DA,
            0x27,
            &[0x60, 0x33, 0xC0, 0xE8, 0x0A, 0x05, 0x00, 0x00],
            &[0x03, 0x0D, 0x1B, 0x20],
            1,
        ),
        list_primary_modes,
    );
    t.finish()
}
