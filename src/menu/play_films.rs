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
//! their Rust versions (2026-10-19). Each real call writes a line to hook.log saying which
//! version runs.
//! Hook removed 2026-10-22 (direct calls): Rust callers run its `entry` (`calls::call`,
//! `routes.rs`); the registers above are the original's, for reference.

use crate::framework::{Regs, game_addr, log, rva};
use rc2k_core::menu::films::{self, Film, Flags, Step};

// ---- game data ----

const NO_FMV_RVA: usize = rva(0x5BE870); // g_noFmv
const SCREEN_BITS_RVA: usize = rva(0x4F4954); // g_screenBits
const AL_LOGO_RVA: usize = rva(0x529822); // ";3files\fmv\allogo.avi"
const MF_LOGO_RVA: usize = rva(0x52980B); // ";3files\fmv\mflogo.mpg"
const INTRO_RVA: usize = rva(0x5297F5); // ";3files\fmv\intro.mpg"
const ZERO_FLAG: u32 = 0x40;

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

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
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
            _ => crate::calls::call(rva(at), regs),
        });
    }
    log("menu::play_films: the Rust version has come back");
    Ok(())
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
        crate::calls::call(rva(at), regs);
    }
}
