//! `system::main_loop` (0x415383): what `start` calls once everything is set up: it waits
//! for the window, runs the game (or the installer) and saves on the way out.
//!
//! In:  nothing
//! Out: nothing (its one caller looks at no register)
//! - If `dword_518390` is not 0 the loop is being entered a second time, by the way out
//!   at 0x4154B5 (`mov esp, dword_518390; jmp` here, taken by two fatal stubs about the
//!   CD-ROM): `threading::request_quit`, the log line "Leaving main loop...", and back to
//!   `start`. Else the stack pointer is kept there.
//! - An empty log line and "Entering main loop...".
//! - It sleeps a second at a time until the window is the active one (`dword_518374`)
//!   and the key thread has run (`dword_518998`); the quit switch (`dword_51837C`) at 1
//!   ends the wait and the loop, without `request_quit`.
//! - `dword_51838C` and `dword_518080` = 0; `sub_404823`, `sub_40EFF3`, `sub_4075F3`,
//!   `sub_4165F1` (not ported).
//! - If the installer's script is set up (`word_51D752` is 0xBD): the installer
//!   `sub_412DE0`, then the way out. Else, unless installing, `install::runs_from_source`
//!   (its answer is not used); `g_installing`, `dword_51B9CC` and `dword_51B9D0` = 0.
//! - `sub_40FCAA`; `files::is_cd_drive` (0x403310) on the base directory's first
//!   character, whose answer this build's nine `nop`s throw away.
//! - If `dword_5AB290` is 1: `sub_418DF4`, then the way out. Else `menu::run_front_end`,
//!   `sub_40F826`, and unless the quit switch is 1 by then (which leaves at once)
//!   `saves::save_game_data` and `saves::free_best_times`.
//! - The way out: `request_quit` and "Leaving main loop...".
//!
//! One caller, `start`.
//! Ungated thunk: the whole game runs inside it. Shadow mode runs the original alone.
//! Each real call writes a line to hook.log saying which version runs.
//! Self-test: a redirected copy whose variables are the test's and whose seventeen kinds
//! of call follow a script.
//! Refuses what its parts refuse; such a refusal comes with the work half done.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, call_game, game_addr, hook_asm, log, rva, shadow};
use crate::install::stand_ins::{act, set_handler, stand_in};
use crate::redirect::RedirectedCopy;
use std::sync::Mutex;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("system::main_loop", rva(0x415383), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL,
    ungated
);

// ---- game data ----

/// The variables the loop reads and writes, in the order of `Vars`'s fields.
const VARS: [usize; 12] = [
    0x518390, // dword_518390: the stack pointer at entry, 0 before
    0x51837C, // dword_51837C: the quit switch
    0x518374, // dword_518374: the window is the active one
    0x518998, // dword_518998: counted up by the key thread
    0x51838C, // dword_51838C
    0x518080, // dword_518080
    0x51D752, // word_51D752: 0xBD when the installer's script is set up
    0x51B9E0, // g_installing
    0x51B9CC, // dword_51B9CC
    0x51B9D0, // dword_51B9D0
    0x5189A8, // g_baseDirectory
    0x5AB290, // dword_5AB290
];
/// How often the code names each of them.
const NAMED: [usize; 12] = [2, 2, 1, 1, 1, 1, 1, 2, 1, 1, 1, 1];
const ENTERING_RVA: usize = rva(0x4DAA80); // "Entering main loop..."
const LEAVING_RVA: usize = rva(0x4DAA96); // "Leaving main loop..."
const INSTALLER_MARK: u16 = 0xBD;

/// Where the loop's variables are: the game's own, or a test's.
#[derive(Clone, Copy)]
pub struct Vars(pub [*mut u32; 12]);

impl Vars {
    fn game() -> Self {
        Self(VARS.map(|at| game_addr(rva(at)) as *mut u32))
    }

    unsafe fn get(&self, n: usize) -> u32 {
        unsafe { self.0[n].read_unaligned() }
    }

    unsafe fn set(&self, n: usize, value: u32) {
        unsafe { self.0[n].write_unaligned(value) }
    }
}

// ---- Rust version ----

/// What the loop calls, in the order of the test's stand-ins.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Call {
    /// A line of the game's log: the text's address, or 0 for the empty line.
    Log(u32),
    Sleep(u32),
    Sub404823,
    Sub40EFF3,
    Sub4075F3,
    Sub4165F1,
    RunsFromSource,
    Installer,
    Sub40FCAA,
    /// `files::is_cd_drive` on this character.
    IsCdDrive(u8),
    Sub418DF4,
    RunFrontEnd,
    Sub40F826,
    SaveGameData,
    FreeBestTimes,
    RequestQuit,
}

pub type Outside<'a> = &'a mut dyn FnMut(Call) -> Result<(), Refused>;

/// `stack` is the stack pointer at the loop's entry.
pub unsafe fn main_loop(vars: &Vars, stack: u32, outside: Outside) -> Result<(), Refused> {
    unsafe {
        let leaving = |outside: Outside| outside(Call::Log(game_addr(LEAVING_RVA) as u32));
        let quit_and_leave = |outside: Outside| {
            outside(Call::RequestQuit)?;
            outside(Call::Log(game_addr(LEAVING_RVA) as u32))
        };
        if vars.get(0) != 0 {
            return quit_and_leave(outside);
        }
        vars.set(0, stack);
        outside(Call::Log(0))?;
        outside(Call::Log(game_addr(ENTERING_RVA) as u32))?;
        loop {
            outside(Call::Sleep(1000))?;
            if vars.get(1) == 1 {
                return leaving(outside);
            }
            if vars.get(2) != 0 && vars.get(3) != 0 {
                break;
            }
        }
        vars.set(4, 0);
        vars.set(5, 0);
        outside(Call::Sub404823)?;
        outside(Call::Sub40EFF3)?;
        outside(Call::Sub4075F3)?;
        outside(Call::Sub4165F1)?;
        // The standalone in the download folder installs from it (`install::portable`); and
        // with `-u` (`dword_51B9CC`) it uninstalls where it is: its record is in the
        // installed folder, so the uninstall runs from there (the original's ran from the
        // CD, through the update's mark).
        if (vars.0[6] as *const u16).read_unaligned() == INSTALLER_MARK
            || (crate::framework::standalone()
                && (crate::install::portable::is_source_folder() || vars.get(8) == 1))
        {
            outside(Call::Installer)?;
            return quit_and_leave(outside);
        }
        if vars.get(7) != 1 {
            outside(Call::RunsFromSource)?;
        }
        vars.set(7, 0);
        vars.set(8, 0);
        vars.set(9, 0);
        outside(Call::Sub40FCAA)?;
        outside(Call::IsCdDrive((vars.0[10] as *const u8).read()))?;
        if vars.get(11) == 1 {
            outside(Call::Sub418DF4)?;
            return quit_and_leave(outside);
        }
        outside(Call::RunFrontEnd)?;
        outside(Call::Sub40F826)?;
        if vars.get(1) == 1 {
            return leaving(outside);
        }
        outside(Call::SaveGameData)?;
        outside(Call::FreeBestTimes)?;
        quit_and_leave(outside)
    }
}

// ---- hook ----

/// The calls: the Rust versions of what is ported, the game for the rest (the front end
/// through its own hook).
fn in_game(call: Call) -> Result<(), Refused> {
    let game = |at: usize, eax: u32| unsafe {
        call_game(
            &mut Regs {
                eax,
                ..Regs::default()
            },
            game_addr(rva(at)),
        )
    };
    unsafe {
        match call {
            Call::Log(0) => crate::menu::text_entry::log_l1(c"".as_ptr() as *const u8, true),
            Call::Log(text) => crate::menu::text_entry::log_l1(text as *const u8, true),
            Call::Sleep(milliseconds) => crate::system::sleep::sleep(milliseconds),
            Call::Sub404823 => game(0x404823, 0),
            Call::Sub40EFF3 => game(0x40EFF3, 0),
            Call::Sub4075F3 => game(0x4075F3, 0),
            Call::Sub4165F1 => game(0x4165F1, 0),
            Call::RunsFromSource => game(0x412DA3, 0),
            Call::Installer => crate::install::run_installer::run_installer(
                &Regs::default(),
                &mut crate::install::installer::game_part,
            )
            .map(|_| ())?,
            Call::Sub40FCAA => game(0x40FCAA, 0),
            // Its answer is not read.
            Call::IsCdDrive(drive) => {
                crate::files::is_cd_drive::is_cd_drive(drive);
            }
            // `net::lobby_launched_game`'s Rust version (a case it refuses through its hook).
            Call::Sub418DF4 => {
                crate::race::loadlevel::or_in_game(0x418DF4, &mut Regs::default(), |r| {
                    *r = crate::net::lobby_launched_game::lobby_launched_game(
                        r,
                        &mut crate::net::lobby::game_part,
                    )?;
                    Ok(())
                })?
            }
            Call::RunFrontEnd => game(0x47486C, 0),
            Call::Sub40F826 => game(0x40F826, 0),
            Call::SaveGameData => game(0x411499, 0),
            Call::FreeBestTimes => game(0x47C627, 0),
            Call::RequestQuit => game(0x410BF9, 0),
        }
    }
    Ok(())
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        log("system::main_loop: the original runs");
        unsafe { call_original(r) };
        return;
    }
    log("system::main_loop: the Rust version runs");
    // The thunk has pushed the flags since the entry: the stack pointer was 4 higher.
    let stack = r.esp.wrapping_add(4);
    match unsafe { main_loop(&Vars::game(), stack, &mut in_game) } {
        Ok(()) => log("system::main_loop: the Rust version has come back"),
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

// ---- self-test ----

const LEN: usize = 0x10E;
const ENTRY_BYTES: [u8; 8] = [0x83, 0x3D, 0x90, 0x83, 0x51, 0x00, 0x00, 0x0F];
const CALLS: [usize; 18] = [
    0x13, 0x1F, 0x2A, 0x62, 0x67, 0x6C, 0x71, 0x8A, 0x91, 0xB6, 0xC0, 0xD7, 0xDE, 0xE3, 0xF1, 0xF6,
    0xFB, 0x107,
];
/// The callees and the place each goes to (the two log functions share place 0).
const CALLEES: [(usize, u32); 17] = [
    (0x4013DE, 0), // logfile::log_newline_l1
    (0x40142B, 0), // logfile::log_write_l1_nl
    (0x41100C, 1), // system::sleep
    (0x404823, 2),
    (0x40EFF3, 3),
    (0x4075F3, 4),
    (0x4165F1, 5),
    (0x412DA3, 6), // install::runs_from_source
    (0x412DE0, 7), // the installer
    (0x40FCAA, 8),
    (0x403310, 9), // files::is_cd_drive
    (0x418DF4, 10),
    (0x47486C, 11), // menu::run_front_end
    (0x40F826, 12),
    (0x411499, 13), // saves::save_game_data
    (0x47C627, 14), // saves::free_best_times
    (0x410BF9, 15), // threading::request_quit
];

/// What a case's stand-ins do, and what was called.
struct Script {
    /// The test's variables, for the stand-ins that change them.
    vars: usize,
    /// After this many sleeps the window is active and the key thread has run.
    ready_after: u32,
    /// After this many sleeps the quit switch is set (0: never).
    quit_after: u32,
    /// The front end leaves the quit switch set.
    quits_in_front_end: bool,
    sleeps: u32,
    calls: Vec<(u32, u32)>,
}

static SCRIPT: Mutex<Script> = Mutex::new(Script {
    vars: 0,
    ready_after: 0,
    quit_after: 0,
    quits_in_front_end: false,
    sleeps: 0,
    calls: Vec::new(),
});

fn handler(place: u32, r: &mut Regs) {
    let mut script = SCRIPT.lock().unwrap();
    let vars = script.vars as *mut u32;
    // The empty log line comes without a text in eax; the drive is eax's low byte.
    let texts = [ENTERING_RVA, LEAVING_RVA].map(|text| game_addr(text) as u32);
    let noted = match place {
        0 if texts.contains(&r.eax) => r.eax,
        1 => r.eax,
        9 => r.eax & 0xFF,
        _ => 0,
    };
    script.calls.push((place, noted));
    unsafe {
        match place {
            1 => {
                script.sleeps += 1;
                if script.sleeps == script.quit_after {
                    vars.add(1).write(1);
                }
                if script.sleeps >= script.ready_after {
                    vars.add(2).write(1);
                    vars.add(3).write(7);
                }
            }
            11 if script.quits_in_front_end => vars.add(1).write(1),
            _ => {}
        }
    }
}

fn through_the_test(call: Call) -> Result<(), Refused> {
    let (place, eax) = match call {
        Call::Log(text) => (0, text),
        Call::Sleep(milliseconds) => (1, milliseconds),
        Call::Sub404823 => (2, 0),
        Call::Sub40EFF3 => (3, 0),
        Call::Sub4075F3 => (4, 0),
        Call::Sub4165F1 => (5, 0),
        Call::RunsFromSource => (6, 0),
        Call::Installer => (7, 0),
        Call::Sub40FCAA => (8, 0),
        Call::IsCdDrive(drive) => (9, drive as u32),
        Call::Sub418DF4 => (10, 0),
        Call::RunFrontEnd => (11, 0),
        Call::Sub40F826 => (12, 0),
        Call::SaveGameData => (13, 0),
        Call::FreeBestTimes => (14, 0),
        Call::RequestQuit => (15, 0),
    };
    act(
        place,
        &mut Regs {
            eax,
            ..Regs::default()
        },
    );
    Ok(())
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let copy = unsafe {
        RedirectedCopy::new(
            game_addr(rva(0x415383)),
            LEN,
            &ENTRY_BYTES,
            &CALLS,
            &|callee| {
                CALLEES
                    .iter()
                    .find(|&&(known, _)| rva(known) == callee)
                    .map(|&(_, place)| stand_in(place))
            },
        )
    };
    let copy = match copy {
        Ok(copy) => copy,
        Err(why) => return t.skip(why),
    };
    // The copy's variables become the test's.
    let mut vars = [0u32; 12];
    unsafe {
        let code = std::slice::from_raw_parts(copy.addr() as *const u8, LEN);
        let mut patches = Vec::new();
        let mut seen = [0usize; 12];
        let mut at = 0;
        while at + 4 <= LEN {
            let named = u32::from_le_bytes(code[at..at + 4].try_into().unwrap());
            match VARS
                .iter()
                .position(|&var| game_addr(rva(var)) as u32 == named)
            {
                Some(n) => {
                    seen[n] += 1;
                    patches.push((at, vars.as_mut_ptr().add(n) as u32));
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
    let test_vars = Vars(std::array::from_fn(|n| vars.as_mut_ptr().wrapping_add(n)));

    // (what, the variables to begin with, ready after, quit after, quits in the front end)
    let fresh = |entered: u32, mark: u32, installing: u32, other: u32| -> [u32; 12] {
        [
            entered,
            0,
            0,
            0,
            0xAAAA,
            0xBBBB,
            mark,
            installing,
            0xCCCC,
            0xDDDD,
            b'd' as u32,
            other,
        ]
    };
    let cases: [(&str, [u32; 12], u32, u32, bool); 9] = [
        ("a normal run", fresh(0, 0, 0, 0), 1, 0, false),
        ("a wait of four seconds", fresh(0, 0, 0, 0), 4, 0, false),
        ("quit while waiting", fresh(0, 0, 0, 0), 9, 3, false),
        ("quit in the front end", fresh(0, 0, 0, 0), 2, 0, true),
        ("the installer's script", fresh(0, 0xBD, 0, 0), 1, 0, false),
        ("installing", fresh(0, 0x1_00BC, 1, 0), 1, 0, false),
        ("the other game", fresh(0, 0, 0, 1), 1, 0, false),
        (
            "entered a second time",
            fresh(0x12_3456, 0, 0, 0),
            1,
            0,
            false,
        ),
        (
            "the mark's low word only",
            fresh(0, 0xFFFF_00BD, 2, 2),
            1,
            0,
            false,
        ),
    ];
    for (what, begin, ready_after, quit_after, quits_in_front_end) in cases {
        let before = Regs::SENTINEL;
        let mut run = |rust: bool| unsafe {
            std::ptr::write_volatile(&raw mut vars, begin);
            *SCRIPT.lock().unwrap() = Script {
                vars: (&raw mut vars) as usize,
                ready_after,
                quit_after,
                quits_in_front_end,
                sleeps: 0,
                calls: Vec::new(),
            };
            set_handler(handler);
            let ended = if rust {
                main_loop(&test_vars, 0x7777_0000, &mut through_the_test).is_err() as u32
            } else {
                let mut ro = before;
                call_game(&mut ro, copy.addr());
                0
            };
            let mut left = std::ptr::read_volatile(&raw const vars);
            // The stack pointer kept is each side's own: only whether one was kept.
            if begin[0] == 0 {
                left[0] = (left[0] != 0) as u32;
            }
            (ended, left, SCRIPT.lock().unwrap().calls.clone())
        };
        let (theirs, mine) = (run(false), run(true));
        t.check(theirs == mine && theirs.0 == 0, || {
            format!(
                "{what}: original left {:x?} after {} calls; rust ended {} left {:x?} after {} calls; first different call {:?}",
                theirs.1,
                theirs.2.len(),
                mine.0,
                mine.1,
                mine.2.len(),
                theirs.2.iter().zip(&mine.2).position(|(a, b)| a != b),
            )
        });
    }
    t.finish()
}
