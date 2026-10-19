//! `net::set_up` (0x41848E): what the network game needs before anything else: the
//! receiving thread's event and the mutex, and whether a lobby launched the game.
//!
//! In:  nothing
//! Out: every register preserved (`pusha` .. `popa`)
//! With `g_noticeKind` (a byte) 2, `dword_52ABD0` (signed) held to 4 at most. An event (auto reset,
//! not signalled: `threading::create_event` with eax 0, ebx 0) to `dword_52AB68`, none:
//! fatal error 0x80168501; a mutex (`threading::create_mutex`) to `dword_52AB6C`, none:
//! fatal error 0x80168502. Then `sub_418E23` (through `part`): DirectPlay's lobby asked
//! (`CoCreateInstance`) whether the game was launched by one ("NET: Lobby launch
//! detected", `dword_5AB290` 1); the platform's (DirectPlay), the game's in the game,
//! nothing in the standalone program (no lobby: `dword_5AB290` stays 0).
//! One caller, `start` (0x4010A2). Shadow mode runs the original alone (handles cannot be
//! made twice). Self-test: a copy whose five variables are the test's, whose three calls go
//! to stand-ins (handles of the test's, or none) and whose jumps to the fatal stubs land in
//! the test, against the Rust version.

use crate::errors::{FatalError, Refused};
use crate::framework::{
    RawHook, Regs, SelfTest, call_game, game_addr, hook_asm, rva, shadow, standalone,
};
use crate::install::stand_ins::{WENT_FATAL, act, fatal_landing, set_handler, stand_in};
use crate::threading::create_event::{create_event, handle_flags};
use crate::threading::create_mutex::create_mutex;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("net::set_up", rva(0x41848E), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

const NOTICE_KIND_RVA: usize = rva(0x4E07A0); // g_noticeKind
const PLAYERS_RVA: usize = rva(0x52ABD0); // dword_52ABD0: held to 4 with notice kind 2
pub const EVENT_RVA: usize = rva(0x52AB68); // dword_52AB68: the receiving thread's event
const MUTEX_RVA: usize = rva(0x52AB6C); // dword_52AB6C
/// The calls by place: 0 `threading::create_event` (eax, ebx -> eax, carry), 1
/// `threading::create_mutex` (-> eax, carry), 2 `sub_418E23` (the lobby).
const CALLEES: [usize; 3] = [0x41103C, 0x41107F, 0x418E23];
const NO_EVENT: FatalError = FatalError {
    code: 0x8016_8501,
    message: "",
};
const NO_MUTEX: FatalError = FatalError {
    code: 0x8016_8502,
    message: "",
};

pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;

/// Where the variables are (the game's, or a test's).
#[derive(Clone, Copy)]
pub struct NetVars {
    /// g_noticeKind, a byte
    pub notice_kind: u32,
    pub players: u32,
    pub event: u32,
    pub mutex: u32,
}

impl NetVars {
    pub fn game() -> Self {
        let a = |r| game_addr(r) as u32;
        Self {
            notice_kind: a(NOTICE_KIND_RVA),
            players: a(PLAYERS_RVA),
            event: a(EVENT_RVA),
            mutex: a(MUTEX_RVA),
        }
    }
}

// ---- Rust version ----

unsafe fn get(at: u32) -> u32 {
    unsafe { (at as *const u32).read_unaligned() }
}

unsafe fn set(at: u32, value: u32) {
    unsafe { (at as *mut u32).write_unaligned(value) };
}

pub unsafe fn set_up(v: &NetVars, part: Part) -> Result<(), Refused> {
    unsafe {
        if (v.notice_kind as *const u8).read() == 2 && get(v.players) as i32 > 4 {
            set(v.players, 4);
        }
        let mut r = Regs::default();
        part(0, &mut r)?;
        if r.carry() {
            return Err(NO_EVENT.into());
        }
        set(v.event, r.eax);
        part(1, &mut r)?;
        if r.carry() {
            return Err(NO_MUTEX.into());
        }
        set(v.mutex, r.eax);
        part(2, &mut r)?;
        Ok(())
    }
}

/// In the game: the handles from the Rust versions, the lobby through the game; in the
/// standalone program there is no lobby.
pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    match place {
        0 | 1 => {
            r.eax = if place == 0 {
                create_event(0, 0)
            } else {
                create_mutex()
            };
            r.set_carry(handle_flags(r.eax).0);
        }
        _ if standalone() => {}
        _ => unsafe { call_game(r, game_addr(rva(CALLEES[2]))) },
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
    if let Err(e) = unsafe { set_up(&NetVars::game(), &mut game_part) } {
        unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) };
    }
}

// ---- self-test ----

/// The calls noted: (place, eax, ebx).
static NOTED: Mutex<Vec<(u32, u32, u32)>> = Mutex::new(Vec::new());
/// Which call gives no handle (3: none).
static FAILS: AtomicU32 = AtomicU32::new(3);

fn note(place: u32, r: &mut Regs) {
    NOTED.lock().unwrap().push((place, r.eax, r.ebx));
    if place < 2 {
        let fails = FAILS.load(Ordering::SeqCst) == place;
        r.eax = if fails { 0 } else { 0x1234_0000 + place };
        r.set_carry(fails);
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
    let mut vars = vec![0u32; 4];
    let at = vars.as_mut_ptr() as u32;
    let v = NetVars {
        notice_kind: at,
        players: at + 4,
        event: at + 8,
        mutex: at + 12,
    };
    let c = match crate::audio::test_copy::make(
        &HOOK,
        &crate::audio::test_copy::Copy {
            address: 0x41848E,
            len: 0x48,
            entry: &[0x60, 0x80, 0x3D, 0xA0, 0x07, 0x4E, 0x00, 0x02],
            calls: &[0x21, 0x31, 0x41],
            jumps_out: &[],
        },
        &|callee| {
            CALLEES
                .iter()
                .position(|&c| rva(c) == callee)
                .map(|n| stand_in(n as u32))
        },
        &[
            (0x4E07A0, v.notice_kind),
            (0x52ABD0, v.players),
            (0x52AB68, v.event),
            (0x52AB6C, v.mutex),
        ],
        5,
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    unsafe {
        // The two `jb` to the fatal stubs (0F 82 at +0x26 and +0x36).
        c.retarget(0x28, fatal_landing as *const () as usize);
        c.retarget(0x38, fatal_landing as *const () as usize);
    }
    set_handler(note);
    for kind in [0u32, 2, 3] {
        for players in [0u32, 4, 5, 0xFFFF_FFFF, 8] {
            for fails in [3u32, 0, 1] {
                FAILS.store(fails, Ordering::SeqCst);
                let run = |rust: bool| unsafe {
                    let p = at as *mut u32;
                    p.write_volatile(kind);
                    p.add(1).write_volatile(players);
                    p.add(2).write_volatile(0xAAAA_AAAA);
                    p.add(3).write_volatile(0xAAAA_AAAA);
                    NOTED.lock().unwrap().clear();
                    WENT_FATAL.store(0, Ordering::SeqCst);
                    let (fatal, preserved) = if rust {
                        match set_up(&v, &mut test_part) {
                            Ok(()) => (0, true),
                            Err(Refused::Fatal(e)) => (e.code, true),
                            Err(_) => (1, false),
                        }
                    } else {
                        let before = Regs::SENTINEL;
                        let mut ro = before;
                        call_game(&mut ro, c.addr());
                        let fatal = match (WENT_FATAL.load(Ordering::SeqCst), fails) {
                            (0, _) => 0,
                            (_, 0) => NO_EVENT.code,
                            _ => NO_MUTEX.code,
                        };
                        (fatal, ro.preserved_except(&before, 0))
                    };
                    let state: Vec<u32> = (0..4).map(|n| p.add(n).read_volatile()).collect();
                    (
                        fatal,
                        preserved,
                        state,
                        std::mem::take(&mut *NOTED.lock().unwrap()),
                    )
                };
                let theirs = run(false);
                let mine = run(true);
                t.check(theirs == mine && theirs.1, || {
                    format!(
                        "kind {kind}, players {players:#x}, failing call {fails}: original {theirs:x?}; rust {mine:x?}"
                    )
                });
            }
        }
    }
    drop(vars);
    t.finish()
}
