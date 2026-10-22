//! `menu::play_films` (0x4161CD): the films: the two logos at the start, then the intro,
//! over and over with the game's own demonstration between, until a key.
//!
//! In:  eax = 0 at the game's start, 1 from the main menu when it has stood idle
//! Out: every register preserved
//! - `sub_40F826`, `sub_4059F4`, and a pending key is taken and thrown away.
//! - At the start only: nothing at all (straight to the end) with the setting `nofmv`;
//!   else `allogo.avi` if the screen has 16 bits (its extension swapped by
//!   `files::swap_missing_extension` if the file is missing), then `mflogo.mpg`, each
//!   through the player `sub_41AE39` with ebx = 0; then `sub_4059F4`.
//! - `sub_415515`; from the idle menu also `sub_4433A7`, `sub_44346B`, `nullsub_6`,
//!   `sub_443396`.
//! - Then round and round: without `nofmv`, and if `intro.mpg` is there, the intro
//!   through the player, which ends the round if it comes back with the zero flag
//!   clear; else `sub_463666` (ebp kept across it), `sub_40F826`, and a key ends the
//!   round.
//! - `sub_409B76`, `sub_4059F4`; from the idle menu also `sub_442AF6`, `sub_474E04`,
//!   `sub_442AF6`, `sub_40F780`.
//!
//! What the unnamed ones do has not been read. Callers: `menu::run_front_end` (with 0),
//! `menu::main_menu_loop`'s idle step (with 1), and 0x470593.
//! Every call goes through the game's address (hooks where ported), the registers handed
//! from one to the next as they come back; but `nullsub_6` is not called, and
//! `scene::set_up_lit_engine` (`sub_415515`) and `race::credits_race` (`sub_463666`) are
//! their Rust versions (2026-10-19). Ungated thunk: a film runs for a long time. Shadow mode runs
//! the original alone. Each real call writes a line to hook.log saying which version
//! runs. Manual patch with a private copy (0xC5 bytes, 23 calls relocated): a call
//! starts at the fourth byte.
//! Self-test: a redirected copy whose sixteen callees follow a script and whose two
//! settings are the test's.

use crate::framework::{RawHook, Regs, SelfTest, call_game, game_addr, hook_asm, log, rva, shadow};
use crate::install::stand_ins::{act, set_handler, stand_in};
use crate::redirect::RedirectedCopy;
use rc2k_core::menu::films::{self, Film, Flags, Step};
use std::sync::Mutex;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new("menu::play_films", rva(0x4161CD), thunk, &ORIGINAL)
    .private_copy(LEN)
    .relocate_calls(&CALLS)
    .manual_patch()
    .self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL,
    ungated
);

// ---- game data ----

const NO_FMV_RVA: usize = rva(0x5BE870); // g_noFmv
const SCREEN_BITS_RVA: usize = rva(0x4F4954); // g_screenBits
const AL_LOGO_RVA: usize = rva(0x529822); // ";3files\fmv\allogo.avi"
const MF_LOGO_RVA: usize = rva(0x52980B); // ";3files\fmv\mflogo.mpg"
const INTRO_RVA: usize = rva(0x5297F5); // ";3files\fmv\intro.mpg"
const ZERO_FLAG: u32 = 0x40;

/// The callees, in the order of the test's stand-ins.
const CALLEES: [usize; 16] = [
    0x40F826, 0x4059F4, 0x407F95, // input::take_key
    0x4032DB, // files::swap_missing_extension
    0x41AE39, // the player
    0x415515, 0x4433A7, 0x44346B, 0x44347E, 0x443396, 0x402EBE, // files::file_exists
    0x463666, 0x409B76, 0x442AF6, 0x474E04, 0x40F780,
];
const TAKE_KEY: usize = 0x407F95;
const SWAP_MISSING_EXTENSION: usize = 0x4032DB;
const PLAYER: usize = 0x41AE39;
const FILE_EXISTS: usize = 0x402EBE;
const DEMONSTRATION: usize = 0x463666;

// ---- Rust version ----

/// Calls the function at that address (as IDA shows it) with the registers, which come
/// back as it leaves them.
pub type CallAt<'a> = &'a mut dyn FnMut(usize, &mut Regs);

/// The game for `rc2k_core::menu::films`: the settings where `settings` says, the steps
/// at their addresses with the registers the original gives them (one set carried
/// through, `r`).
struct Game<'a, 'b> {
    r: &'a mut Regs,
    settings: [*const u32; 2],
    call: CallAt<'b>,
}

fn film(f: Film) -> u32 {
    game_addr(match f {
        Film::AlLogo => AL_LOGO_RVA,
        Film::MfLogo => MF_LOGO_RVA,
        Film::Intro => INTRO_RVA,
    }) as u32
}

impl films::Films for Game<'_, '_> {
    fn no_fmv(&mut self) -> bool {
        unsafe { self.settings[0].read_unaligned() != 0 }
    }

    fn screen_bits(&mut self) -> u32 {
        unsafe { self.settings[1].read_unaligned() }
    }

    fn step(&mut self, s: Step) -> Flags {
        let r = &mut *self.r;
        let at = match s {
            Step::StopMusic => 0x40F826,
            Step::Step4059F4 => 0x4059F4,
            Step::TakeKey => TAKE_KEY,
            Step::SwapMissingExtension(f) => {
                r.eax = film(f);
                SWAP_MISSING_EXTENSION
            }
            Step::Player(f) => {
                if let Some(f) = f {
                    r.eax = film(f);
                }
                r.ebx = 0;
                PLAYER
            }
            Step::LitEngine => 0x415515,
            Step::Render3dTidy => 0x4433A7,
            Step::ObjectsTidy => 0x44346B,
            Step::Nothing => 0x44347E,
            Step::ReleaseTextures => 0x443396,
            Step::FileExists(f) => {
                r.eax = film(f);
                FILE_EXISTS
            }
            Step::Demonstration => DEMONSTRATION,
            Step::Step409B76 => 0x409B76,
            Step::SetUpEngine => 0x442AF6,
            Step::FrontEndScene => 0x474E04,
            Step::Tune3 => 0x40F780,
        };
        if s == Step::Demonstration {
            // ebp is pushed and popped around this one.
            let kept = r.ebp;
            (self.call)(at, r);
            r.ebp = kept;
        } else {
            (self.call)(at, r);
        }
        Flags {
            carry: r.carry(),
            zero: r.eflags & ZERO_FLAG != 0,
        }
    }
}

/// `r` holds the registers at entry (eax the mode); `settings` are where `g_noFmv` and
/// `g_screenBits` are.
pub unsafe fn play_films(r: &mut Regs, settings: [*const u32; 2], call: CallAt) {
    let mode = r.eax;
    r.ebp = mode;
    // Converted: `rc2k_core::menu::films::play_films`.
    films::play_films(&mut Game { r, settings, call }, mode != 0);
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        log(&format!(
            "menu::play_films: the original runs (mode {})",
            r.eax
        ));
        unsafe { call_original(r) };
        return;
    }
    log(&format!(
        "menu::play_films: the Rust version runs (mode {})",
        r.eax
    ));
    // Between `pusha` and `popa`: the caller's registers stay as they are.
    let mut running = *r;
    let settings = [NO_FMV_RVA, SCREEN_BITS_RVA].map(|at| game_addr(at) as *const u32);
    unsafe {
        play_films(&mut running, settings, &mut |at, regs| match at {
            // `nullsub_6`.
            0x44347E => {}
            0x415515 => in_game_unless(at, regs, |r| {
                crate::scene::set_up_lit_engine::set_up_lit_engine_in(
                    r,
                    &mut crate::scene::set_up_lit_engine::lit_part,
                )
            }),
            DEMONSTRATION => in_game_unless(at, regs, |r| {
                crate::race::credits_race::credits_race_in(
                    r,
                    &mut crate::race::credits_race::credits_part,
                )
            }),
            SWAP_MISSING_EXTENSION => in_game_unless(at, regs, |r| {
                crate::files::swap_missing_extension::swap_missing_extension(r.eax as *mut u8)?;
                Ok(())
            }),
            FILE_EXISTS => in_game_unless(at, regs, |r| {
                let there = crate::files::file_exists::file_exists(r.eax as *const u8)?;
                r.set_carry(!there);
                r.set_zero(there);
                Ok(())
            }),
            _ => call_game(regs, game_addr(rva(at))),
        });
    }
    log("menu::play_films: the Rust version has come back");
}

/// A ported callee's Rust version; when it refuses, the callee at its hook (which refuses
/// again, logs it and runs its original), from the registers it was given.
fn in_game_unless(
    at: usize,
    regs: &mut Regs,
    rust: impl FnOnce(&mut Regs) -> Result<(), crate::errors::Refused>,
) {
    let entry = *regs;
    if rust(regs).is_err() {
        *regs = entry;
        unsafe { call_game(regs, game_addr(rva(at))) };
    }
}

// ---- self-test ----

const LEN: usize = 0xC5;
const ENTRY_BYTES: [u8; 8] = [0x60, 0x8B, 0xE8, 0xE8, 0x51, 0x96, 0xFF, 0xFF];
const CALLS: [usize; 23] = [
    0x3, 0x8, 0xD, 0x32, 0x39, 0x46, 0x4B, 0x50, 0x59, 0x5E, 0x63, 0x68, 0x7C, 0x85, 0x8D, 0x93,
    0x98, 0xA1, 0xA6, 0xAF, 0xB4, 0xB9, 0xBE,
];
/// How often the code names `g_noFmv` and `g_screenBits`.
const NAMED: [usize; 2] = [2, 1];

/// What a case's stand-ins do, and what was called.
struct Script {
    /// The intro's file is there.
    intro_exists: bool,
    /// The player comes back from the intro with the zero flag clear on this showing of
    /// it, from 1 (0: never).
    intro_ends_on: u32,
    /// A key is there on this poll after the first, from 1.
    key_on: u32,
    intros: u32,
    polls: u32,
    /// Each call: its place, eax and ebx where they are given, and ebp.
    calls: Vec<[u32; 4]>,
}

static SCRIPT: Mutex<Script> = Mutex::new(Script {
    intro_exists: false,
    intro_ends_on: 0,
    key_on: 0,
    intros: 0,
    polls: 0,
    calls: Vec::new(),
});

fn handler(place: u32, r: &mut Regs) {
    let mut script = SCRIPT.lock().unwrap();
    let given = match place {
        3 | 10 => [r.eax, 0],
        4 => [r.eax, r.ebx],
        _ => [0, 0],
    };
    script.calls.push([place, given[0], given[1], r.ebp]);
    match place {
        // The key: the first poll's is thrown away; then none until the script's.
        2 => {
            let key = script.polls != 0 && script.polls == script.key_on;
            script.polls += 1;
            r.set_zero(!key);
        }
        // The player: only the intro's zero flag is looked at.
        4 if r.eax == game_addr(INTRO_RVA) as u32 => {
            script.intros += 1;
            r.set_zero(script.intros != script.intro_ends_on);
        }
        4 => r.set_zero(false),
        10 => r.set_carry(!script.intro_exists),
        // The demonstration leaves ebp changed.
        11 => r.ebp = 0xBAD0_BAD0,
        _ => {}
    }
}

fn through_the_test(at: usize, r: &mut Regs) {
    let place = CALLEES.iter().position(|&known| known == at).unwrap() as u32;
    act(place, r);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let copy = unsafe {
        RedirectedCopy::new(
            game_addr(rva(0x4161CD)),
            LEN,
            &ENTRY_BYTES,
            &CALLS,
            &|callee| {
                CALLEES
                    .iter()
                    .position(|&known| rva(known) == callee)
                    .map(|place| stand_in(place as u32))
            },
        )
    };
    let copy = match copy {
        Ok(copy) => copy,
        Err(why) => return t.skip(why),
    };
    // The copy's two settings become the test's.
    let mut settings = [0u32; 2];
    unsafe {
        let code = std::slice::from_raw_parts(copy.addr() as *const u8, LEN);
        let real = [NO_FMV_RVA, SCREEN_BITS_RVA].map(|at| game_addr(at) as u32);
        let mut patches = Vec::new();
        let mut seen = [0usize; 2];
        let mut at = 0;
        while at + 4 <= LEN {
            let named = u32::from_le_bytes(code[at..at + 4].try_into().unwrap());
            match real.iter().position(|&address| address == named) {
                Some(n) => {
                    seen[n] += 1;
                    patches.push((at, settings.as_mut_ptr().add(n) as u32));
                    at += 4;
                }
                None => at += 1,
            }
        }
        if seen != NAMED {
            return t.skip("the original is not what the test was made from");
        }
        for (at, new) in patches {
            copy.patch_u32(at, new);
        }
    }
    let places: [*const u32; 2] = std::array::from_fn(|n| settings.as_ptr().wrapping_add(n));

    // (what, mode, nofmv, screen bits, the intro is there, the intro ends on, a key on)
    let cases: [(&str, u32, u32, u32, bool, u32, u32); 10] = [
        ("the start, 32 bits, the intro ends", 0, 0, 32, true, 1, 0),
        ("the start, 16 bits, the intro ends", 0, 0, 16, true, 1, 0),
        ("the start, no films", 0, 1, 16, true, 1, 1),
        ("the start, no intro file, a key", 0, 0, 32, false, 0, 2),
        (
            "the start, the intro three times, then a key",
            0,
            0,
            32,
            true,
            0,
            3,
        ),
        (
            "the start, the intro ends the third time",
            0,
            0,
            16,
            true,
            3,
            9,
        ),
        ("idle, the intro ends", 1, 0, 32, true, 1, 0),
        ("idle, no films, a key", 1, 1, 16, true, 0, 1),
        ("idle, no films, a key later", 7, 2, 32, false, 0, 4),
        ("idle, 16 bits, no intro file", 1, 0, 16, false, 0, 2),
    ];
    for (what, mode, no_fmv, bits, intro_exists, intro_ends_on, key_on) in cases {
        let before = Regs {
            eax: mode,
            ..Regs::SENTINEL
        };
        let mut run = |rust: bool| unsafe {
            std::ptr::write_volatile(&raw mut settings, [no_fmv, bits]);
            *SCRIPT.lock().unwrap() = Script {
                intro_exists,
                intro_ends_on,
                key_on,
                intros: 0,
                polls: 0,
                calls: Vec::new(),
            };
            set_handler(handler);
            let preserved = if rust {
                let mut running = before;
                play_films(&mut running, places, &mut through_the_test);
                true
            } else {
                let mut ro = before;
                call_game(&mut ro, copy.addr());
                ro.preserved_except(&before, 0)
            };
            (preserved, SCRIPT.lock().unwrap().calls.clone())
        };
        let (theirs, mine) = (run(false), run(true));
        t.check(theirs == mine && theirs.0, || {
            format!(
                "{what}: original preserved={} with {} calls; rust {} calls; first different call {:?} (original {:x?}, rust {:x?})",
                theirs.0,
                theirs.1.len(),
                mine.1.len(),
                theirs.1.iter().zip(&mine.1).position(|(a, b)| a != b),
                theirs.1.iter().zip(&mine.1).find(|(a, b)| a != b).map(|(a, _)| a),
                theirs.1.iter().zip(&mine.1).find(|(a, b)| a != b).map(|(_, b)| b),
            )
        });
    }
    t.finish()
}
