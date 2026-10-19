//! `net::receive_thread` (0x4183C3): the network's receiving thread ("NETRXTDB"), the
//! routine of the third slot `threading::start_threads` starts (`stru_518714`), from its
//! start to `ExitThread`.
//!
//! In:  the registers the thread starter leaves (ecx, edx, edi, ebp handed on to the calls)
//! Out: does not return (`ExitThread(0)`)
//! Its slot's byte +0xC (running) 1 already: straight to the end. Else running;
//! "NETRXTDB:START" (log level 2); `dword_52A4B0` (done) not 0: the end. Else round and
//! round: `threading::wait_for_object` on the event `dword_52AB68` (`net::set_up`'s) for
//! 1000 ms; signalled (zero clear): `dword_52A918` (receiving) 1 and, with `dword_52A910`
//! (a network session) 1, `sub_4194CC` (DirectPlay's messages taken in: the platform's,
//! through the game; nothing in the standalone program, which has no network); then
//! `dword_52A918` 0; with `dword_52ACE4` 1 the call posted in `dword_52ACE8` (taken with an
//! `xchg`, eax its address) is made and its eax kept in `dword_52ACEC`, `dword_52ACF0` 1
//! (`sub_4184F8` posts them and waits); until `g_quitting` is 1. The end: done 1,
//! "NETRXTDB:QUIT", not running, `ExitThread(0)`.
//! The registers go on from call to call as in the original (the posted call's are kept).
//! The thunk is `ungated` (the thread is in it for the whole game); a turn's calls into the
//! game hold the gate, so a self-test's pause stops them. Shadow mode runs the original
//! alone. Self-test: a redirected copy whose thirteen variable references are the test's,
//! whose calls go to stand-ins (a wait that waits for nothing and says signalled or not by
//! a script, a quit after some turns, calls posted on some turns) and whose `ExitThread`
//! goes back to the test, against the Rust version, nine cases.

use crate::errors::Refused;
use crate::framework::{
    RawHook, Regs, SelfTest, ZERO_FLAG, call_game, game_addr, hook_asm, hook_enter, hook_leave,
    rva, shadow, standalone,
};
use crate::install::stand_ins::{act, set_handler, stand_in};
use std::sync::Mutex;
use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("net::receive_thread", rva(0x4183C3), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL,
    ungated
);

// ---- game data ----

const START_TEXT_RVA: usize = rva(0x4DACD2); // "NETRXTDB:START"
const QUIT_TEXT_RVA: usize = rva(0x4DACE1); // "NETRXTDB:QUIT"
/// The wait for the event, in milliseconds.
const WAIT: u32 = 1000;
/// The calls by place: 0 `logfile::log_write_l2_nl` (eax), 1 `threading::wait_for_object`
/// (eax, ebx -> the zero flag), 2 `sub_4194CC`, 3 the posted call (eax its address), 4
/// `ExitThread(0)` (its thunk, 0x4D5D10).
const CALLEES: [usize; 3] = [0x401439, 0x411063, 0x4194CC];
const EXIT_THREAD: usize = 0x4D5D10;
const CALLS: [usize; 5] = [0x18, 0x31, 0x4B, 0x9A, 0xAC];

pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;

/// Where the variables are (the game's, or a test's).
#[derive(Clone, Copy)]
pub struct ReceiveVars {
    /// stru_518714, the thread's slot.
    pub slot: u32,
    /// dword_52A4B0
    pub done: u32,
    /// dword_52AB68
    pub event: u32,
    /// dword_52A918
    pub receiving: u32,
    /// dword_52A910
    pub session: u32,
    /// dword_52ACE4
    pub posting: u32,
    /// dword_52ACE8
    pub posted: u32,
    /// dword_52ACEC
    pub result: u32,
    /// dword_52ACF0
    pub answered: u32,
    /// g_quitting
    pub quitting: u32,
}

impl ReceiveVars {
    pub fn game() -> Self {
        let a = |x: usize| game_addr(rva(x)) as u32;
        Self {
            slot: a(0x518714),
            done: a(0x52A4B0),
            event: a(0x52AB68),
            receiving: a(0x52A918),
            session: a(0x52A910),
            posting: a(0x52ACE4),
            posted: a(0x52ACE8),
            result: a(0x52ACEC),
            answered: a(0x52ACF0),
            quitting: a(0x51837C),
        }
    }
}

// ---- Rust version ----

unsafe fn get(at: u32) -> u32 {
    unsafe { (at as *const u32).read_volatile() }
}

unsafe fn set(at: u32, value: u32) {
    unsafe { (at as *mut u32).write_volatile(value) };
}

/// Runs to the `ExitThread` (place 4), which in the game does not come back.
pub unsafe fn receive_thread(entry: &Regs, v: &ReceiveVars, part: Part) -> Result<(), Refused> {
    unsafe {
        let mut c = Regs {
            esi: v.slot,
            ..*entry
        };
        let running = (v.slot + 0xC) as *mut u8;
        if running.read_volatile() != 1 {
            running.write_volatile(1);
            part(
                0,
                &mut Regs {
                    eax: game_addr(START_TEXT_RVA) as u32,
                    ..c
                },
            )?;
            if get(v.done) == 0 {
                loop {
                    c.eax = get(v.event);
                    c.ebx = WAIT;
                    part(1, &mut c)?;
                    if c.eflags & ZERO_FLAG == 0 {
                        set(v.receiving, 1);
                        if get(v.session) == 1 {
                            part(2, &mut c)?;
                        }
                    }
                    set(v.receiving, 0);
                    if get(v.posting) == 1 {
                        c.eax = AtomicU32::from_ptr(v.posted as *mut u32).swap(0, Ordering::SeqCst);
                        if c.eax != 0 {
                            part(3, &mut c)?;
                            set(v.result, c.eax);
                            set(v.answered, 1);
                        }
                    }
                    if get(v.quitting) == 1 {
                        break;
                    }
                }
            }
        }
        set(v.done, 1);
        part(
            0,
            &mut Regs {
                eax: game_addr(QUIT_TEXT_RVA) as u32,
                ..c
            },
        )?;
        c.esi = v.slot;
        running.write_volatile(0);
        part(4, &mut c)?;
    }
    Ok(())
}

/// In the game: the log and the calls through the game, the wait from the Rust version; a
/// turn's calls into the game hold the gate. In the standalone program there is no network
/// (place 2 does nothing).
pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    struct Gate;
    impl Drop for Gate {
        fn drop(&mut self) {
            hook_leave();
        }
    }
    match place {
        1 => {
            let (carry, zero) =
                crate::threading::wait_for_object::wait_for_object(r.eax, r.ebx).flags();
            r.set_carry(carry);
            r.set_zero(zero);
        }
        2 if standalone() => {}
        2 | 3 => {
            hook_enter();
            let _gate = Gate;
            let to = if place == 2 {
                game_addr(rva(CALLEES[2]))
            } else {
                r.eax as usize
            };
            unsafe { call_game(r, to) };
        }
        4 => unsafe { windows_sys::Win32::System::Threading::ExitThread(0) },
        _ => unsafe { call_game(r, game_addr(rva(CALLEES[0]))) },
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
    if let Err(e) = unsafe { receive_thread(&entry, &ReceiveVars::game(), &mut game_part) } {
        unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) };
    }
}

// ---- self-test ----

/// Each call: the place, eax, ebx, ecx, edx, esi.
static NOTED: Mutex<Vec<[u32; 6]>> = Mutex::new(Vec::new());
static TURNS: AtomicU32 = AtomicU32::new(0);
/// The turn after which the test's quit switch is set.
static QUIT_AFTER: AtomicU32 = AtomicU32::new(0);
/// Bit n: turn n's wait says signalled. Bit n of `POSTS`: a call is posted in turn n.
static SIGNALS: AtomicU32 = AtomicU32::new(0);
static POSTS: AtomicU32 = AtomicU32::new(0);
/// The test's variables (their first address: `ReceiveVars` in that order of dwords after
/// the slot's four).
static VARS: AtomicU32 = AtomicU32::new(0);

fn vars_at(at: u32) -> ReceiveVars {
    let a = |n: u32| at + 0x10 + 4 * n;
    ReceiveVars {
        slot: at,
        done: a(0),
        event: a(1),
        receiving: a(2),
        session: a(3),
        posting: a(4),
        posted: a(5),
        result: a(6),
        answered: a(7),
        quitting: a(8),
    }
}

fn note(place: u32, r: &mut Regs) {
    NOTED
        .lock()
        .unwrap()
        .push([place, r.eax, r.ebx, r.ecx, r.edx, r.esi]);
    let v = vars_at(VARS.load(Ordering::SeqCst));
    match place {
        1 => {
            let turn = TURNS.fetch_add(1, Ordering::SeqCst);
            r.set_zero(SIGNALS.load(Ordering::SeqCst) & (1 << turn) == 0);
            unsafe {
                if POSTS.load(Ordering::SeqCst) & (1 << turn) != 0 {
                    set(v.posted, stand_in(3) as u32);
                }
                if turn + 1 >= QUIT_AFTER.load(Ordering::SeqCst) {
                    set(v.quitting, 1);
                }
            }
        }
        3 => {
            // The posted call's answer, and a register of its own handed on.
            let turn = TURNS.load(Ordering::SeqCst);
            r.eax = 0x5000 + turn;
            r.ecx = 0xC000 + turn;
        }
        _ => {}
    }
}

fn test_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    act(place, r);
    Ok(())
}

/// The copy's entry registers (ebx, ecx, edx, edi, ebp) and the stack to go back to.
static ENTRY: [AtomicU32; 5] = [const { AtomicU32::new(0) }; 5];
static SAVED: AtomicU32 = AtomicU32::new(0);

/// Runs the copy at `addr` with `ENTRY`'s registers; it ends in `exit_thread`.
#[unsafe(naked)]
unsafe extern "C" fn run_copy(addr: u32) {
    core::arch::naked_asm!(
        "pushad",
        "pushfd",
        "mov [{saved}], esp",
        "mov eax, [esp + 40]",
        "mov ebx, [{entry}]",
        "mov ecx, [{entry} + 4]",
        "mov edx, [{entry} + 8]",
        "mov edi, [{entry} + 12]",
        "mov ebp, [{entry} + 16]",
        "call eax",
        "ud2",
        saved = sym SAVED,
        entry = sym ENTRY,
    )
}

/// `ExitThread` in the copy: noted as place 4 and back to `run_copy`'s caller.
#[unsafe(naked)]
unsafe extern "C" fn exit_thread() {
    core::arch::naked_asm!(
        "pushfd",
        "pushad",
        "mov eax, esp",
        "push 4",
        "push eax",
        "call {note}",
        "add esp, 8",
        "mov esp, [{saved}]",
        "popfd",
        "popad",
        "ret",
        note = sym note_exit,
        saved = sym SAVED,
    )
}

extern "C" fn note_exit(regs: *mut Regs, place: u32) {
    note(place, unsafe { &mut *regs });
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let mut m = vec![0u32; 13];
    let at = m.as_mut_ptr() as u32;
    VARS.store(at, Ordering::SeqCst);
    let v = vars_at(at);
    let c = match crate::audio::test_copy::make(
        &HOOK,
        &crate::audio::test_copy::Copy {
            address: 0x4183C3,
            len: 0xB1,
            entry: &[0xFC, 0x8D, 0x35, 0x14, 0x87, 0x51, 0x00],
            calls: &CALLS,
            jumps_out: &[],
        },
        &|callee| {
            if callee == rva(EXIT_THREAD) {
                return Some(exit_thread as *const () as usize);
            }
            CALLEES
                .iter()
                .position(|&c| rva(c) == callee)
                .map(|n| stand_in(n as u32))
        },
        &[
            (0x518714, v.slot),
            (0x52A4B0, v.done),
            (0x52AB68, v.event),
            (0x52A918, v.receiving),
            (0x52A910, v.session),
            (0x52ACE4, v.posting),
            (0x52ACE8, v.posted),
            (0x52ACEC, v.result),
            (0x52ACF0, v.answered),
            (0x51837C, v.quitting),
        ],
        13,
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    set_handler(note);
    // eax: `run_copy` enters the copy with `call eax`.
    let entry = Regs {
        eax: c.addr() as u32,
        ebx: 0xB0B0,
        ecx: 0xC0C0,
        edx: 0xD0D0,
        edi: 0xE0E0,
        ebp: 0xF0F0,
        ..Regs::default()
    };
    for (n, x) in [entry.ebx, entry.ecx, entry.edx, entry.edi, entry.ebp]
        .iter()
        .enumerate()
    {
        ENTRY[n].store(*x, Ordering::SeqCst);
    }
    // (running already, done already, session, posting, quit after, signals, posts)
    let cases = [
        (false, false, 0u32, 0u32, 1u32, 0u32, 0u32),
        (false, false, 1, 0, 3, 0b101, 0),
        (false, false, 0, 1, 3, 0b111, 0b010),
        (false, false, 1, 1, 5, 0b10110, 0b11001),
        (true, false, 1, 1, 3, 0b111, 0b111),
        (false, true, 1, 1, 3, 0b111, 0b111),
        (false, false, 2, 1, 4, 0b1111, 0b0110),
        (false, false, 1, 2, 4, 0b1010, 0b1111),
        (false, false, 1, 1, 6, 0b111111, 0b100001),
    ];
    for (running, done, session, posting, quit_after, signals, posts) in cases {
        let run = |rust: bool| {
            unsafe {
                let p = at as *mut u32;
                for n in 0..13 {
                    p.add(n).write_volatile(0xAAAA_AAAA);
                }
                (p as *mut u8).add(0xC).write_volatile(running as u8);
                set(v.done, done as u32);
                set(v.event, 0x1234_5678);
                set(v.session, session);
                set(v.posting, posting);
                set(v.posted, 0);
                set(v.quitting, 0);
            }
            TURNS.store(0, Ordering::SeqCst);
            QUIT_AFTER.store(quit_after, Ordering::SeqCst);
            SIGNALS.store(signals, Ordering::SeqCst);
            POSTS.store(posts, Ordering::SeqCst);
            NOTED.lock().unwrap().clear();
            let done = if rust {
                unsafe { receive_thread(&entry, &v, &mut test_part) }.is_ok()
            } else {
                unsafe { run_copy(c.addr() as u32) };
                true
            };
            (
                done,
                std::mem::take(&mut *NOTED.lock().unwrap()),
                unsafe { std::slice::from_raw_parts(at as *const u32, 13) }.to_vec(),
            )
        };
        let theirs = run(false);
        let mine = run(true);
        t.check(theirs.0 && theirs == mine, || {
            format!(
                "running {running}, done {done}, session {session}, posting {posting}, quit after {quit_after}, signals {signals:#b}, posts {posts:#b}: original {:x?} {:x?}; rust {:x?} {:x?}",
                theirs.1, theirs.2, mine.1, mine.2
            )
        });
    }
    drop(m);
    t.finish()
}
