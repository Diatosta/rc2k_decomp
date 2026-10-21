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
//! standalone); the log's call through `part`. Shadow mode runs the original alone (the box
//! is shown). Self-test: a copy whose variables are the test's and whose calls go to
//! noting stand-ins, against the Rust version: reported or not, both kinds of code.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, hook_asm, rva};
use crate::system::api_stand_ins::{Api, copy_with};
use crate::system::exit_parts::{LOG_2_NL, MESSAGE_BOX, Part, compare_exit, dispatch_exit};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "fatal::show_report_at_exit",
    rva(0x407855),
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

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_exit(&HOOK, r, call_original, in_game);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let mut mem = [0u8; 0x20];
    let base = mem.as_mut_ptr() as u32;
    let v = Vars {
        reported: base,
        code: base + 4,
        caption: base + 8,
        text: game_addr(rva(TEXT)) as u32,
    };
    // pusha; cmp g_fatalReported, 1
    let entry = [0x60, 0x80, 0x3D, 0xBC, 0x67, 0x4F, 0x00, 0x01];
    let copy = match copy_with(
        0x407855,
        0x42,
        &entry,
        &[0x2E, 0x3B],
        &[LOG_2_NL],
        &[(0x2E, MESSAGE_BOX, 4)],
        &[
            (REPORTED, 1, v.reported),
            (CODE, 4, v.code),
            (CAPTION, 6, v.caption),
        ],
        4,
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    for (reported, code) in [
        (0u8, 0x9000_0000u32),
        (1, 0x8010_0001),
        (1, 0x9020_1101),
        (2, 0),
    ] {
        let fill = || unsafe {
            let m = base as *mut u8;
            for k in 0..0x20 {
                m.add(k).write_volatile(b'A' + k as u8);
            }
            m.write_volatile(reported);
            ((base + 4) as *mut u32).write_volatile(code);
        };
        compare_exit(
            &mut t,
            &format!("reported {reported}, code {code:#x}"),
            &copy,
            Regs::SENTINEL,
            &fill,
            &[(base, 0x20)],
            &|r, part, api| show_report_at_exit_with(r, &v, part, api),
            false,
        );
    }
    let _ = &mut mem;
    t.finish()
}
