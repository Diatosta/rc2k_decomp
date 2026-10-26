//! `fatal::show_report_at_exit` (0x407855): at the game's end, if a fatal error was
//! reported (`g_fatalReported` 1), its text (`Text`, written by `fatal::report`) shown in
//! a message box (`MessageBoxA`, no window, caption `Caption`, type 0x2030: an
//! exclamation, task-modal); for a code with bit 28 the caption ("Rally: Guru Meditation")
//! is first cut to "Rally" (its byte 5, `byte_4F6702`, 0). Then "END PROCESS" logged.
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! One caller, `start`'s way out. `MessageBoxA` through `api` (the same function in the
//! standalone); the log's call through `part`.
//! Hook removed 2026-10-22 (direct calls): Rust callers run its `entry` (`calls::call`,
//! `routes.rs`); the registers above are the original's, for reference.

use crate::errors::Refused;
use crate::framework::{Regs, game_addr, rva};
use crate::system::api_stand_ins::Api;
use crate::system::exit_parts::{LOG_2_NL, MESSAGE_BOX, Part, exit_entry};

const REPORTED: usize = 0x4F67BC; // g_fatalReported (a byte)
const CODE: usize = 0x4F52A4; // g_fatalCode
const CAPTION: usize = 0x4F66FD; // Caption; byte_4F6702 its sixth byte
const TEXT: usize = 0x9526F8; // Text: the report
const END: usize = 0x4DA4A6; // "END PROCESS"

/// Where the variables are.
#[derive(Clone, Copy)]
pub struct Vars {
    pub reported: u32,
    pub code: u32,
    pub caption: u32,
    pub text: u32,
}

impl Vars {
    pub fn game() -> Self {
        let a = |x: usize| game_addr(rva(x)) as u32;
        Self {
            reported: a(REPORTED),
            code: a(CODE),
            caption: a(CAPTION),
            text: a(TEXT),
        }
    }
}

/// The Rust version on the variables in `v`.
pub fn show_report_at_exit_with(
    r: &mut Regs,
    v: &Vars,
    part: Part,
    api: Api,
) -> Result<(), Refused> {
    unsafe {
        if (v.reported as *const u8).read_volatile() == 1 {
            // Converted: `rc2k_core::fatal::caption_cut`.
            if rc2k_core::fatal::caption_cut((v.code as *const u32).read_volatile()) {
                ((v.caption + rc2k_core::fatal::CAPTION_CUT as u32) as *mut u8).write_volatile(0);
            }
            api(MESSAGE_BOX, &[0, v.text, v.caption, 0x2030]);
        }
    }
    let mut l = Regs {
        eax: game_addr(rva(END)) as u32,
        ..*r
    };
    part(LOG_2_NL, &mut l)
}

fn in_game(r: &mut Regs, part: Part, api: Api) -> Result<(), Refused> {
    show_report_at_exit_with(r, &Vars::game(), part, api)
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    exit_entry(r, in_game)
}
