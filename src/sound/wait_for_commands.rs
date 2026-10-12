//! `sound::wait_for_commands` (0x409C1E): waits until the audio thread has taken every
//! command.
//!
//! In:  nothing (the registers go into the two commands it sends)
//! Out: every register preserved
//! Two "end of turn" commands (0xA) are queued; then, while commands are waiting
//! (`sub_409C01`): a millisecond's sleep, the window's messages (`system::pump_messages`),
//! and the quit switch at 1 ends the wait.
//! Fourteen callers.
//! Shadow mode runs the original alone. Manual patch with a private copy (0x27 bytes,
//! five calls relocated): a call starts at the second byte.
//! Self-test: a redirected copy whose four kinds of call follow a script.

use crate::framework::{
    RawHook, Regs, SelfTest, call_game, game_addr, game_read, hook_asm, rva, shadow,
};
use crate::install::stand_ins::{act, set_handler, stand_in};
use crate::redirect::RedirectedCopy;
use crate::sound::queue::{END_OF_TURN, Queue, pending, send_command, sent};
use std::sync::Mutex;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("sound::wait_for_commands", rva(0x409C1E), thunk, &ORIGINAL)
        .private_copy(LEN)
        .relocate_calls(&CALLS)
        .manual_patch()
        .self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

const QUIT_RVA: usize = rva(0x51837C); // dword_51837C

// ---- Rust version ----

/// What the wait is made of.
pub trait Parts {
    /// Queues an "end of turn" command.
    fn end_turn(&mut self);
    /// Whether commands are still waiting.
    fn pending(&mut self) -> bool;
    fn sleep_a_millisecond(&mut self);
    fn pump_messages(&mut self);
    /// The quit switch is 1.
    fn quitting(&mut self) -> bool;
}

pub fn wait_for_commands(parts: &mut dyn Parts) {
    parts.end_turn();
    parts.end_turn();
    while parts.pending() {
        parts.sleep_a_millisecond();
        parts.pump_messages();
        if parts.quitting() {
            break;
        }
    }
}

// ---- hook ----

/// The parts in the game: the queue's by the Rust functions, the window's messages by
/// the game.
pub struct Game(pub [u32; 7]);

impl Parts for Game {
    fn end_turn(&mut self) {
        unsafe { send_command(&Queue::game(), END_OF_TURN, self.0) };
    }

    fn pending(&mut self) -> bool {
        unsafe { pending(&Queue::game()) }
    }

    fn sleep_a_millisecond(&mut self) {
        crate::system::sleep::sleep(1);
    }

    fn pump_messages(&mut self) {
        unsafe { call_game(&mut Regs::default(), game_addr(rva(0x410535))) };
    }

    fn quitting(&mut self) -> bool {
        unsafe { game_read::<u32>(QUIT_RVA) == 1 }
    }
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    wait_for_commands(&mut Game(sent(r)));
}

// ---- self-test ----

const LEN: usize = 0x27;
const ENTRY_BYTES: [u8; 8] = [0x60, 0xE8, 0xBC, 0xFF, 0xFF, 0xFF, 0xE8, 0xB7];
/// Twice the "end of turn" sender, `sub_409C01`, `sleep_1ms`, `system::pump_messages`.
const CALLS: [usize; 5] = [0x01, 0x06, 0x0B, 0x12, 0x17];
const CALLEES: [usize; 4] = [0x409BE0, 0x409C01, 0x410FFE, 0x410535];
const QUIT_OPERAND: usize = 0x1E;

/// What a case's stand-ins do, and what was called.
struct Script {
    /// How many times commands are still waiting.
    waits: u32,
    /// The quit switch is set during this sleep, from 1 (0: never).
    quit_on: u32,
    quit: usize,
    asked: u32,
    sleeps: u32,
    calls: Vec<u32>,
}

static SCRIPT: Mutex<Script> = Mutex::new(Script {
    waits: 0,
    quit_on: 0,
    quit: 0,
    asked: 0,
    sleeps: 0,
    calls: Vec::new(),
});

fn handler(place: u32, r: &mut Regs) {
    let mut script = SCRIPT.lock().unwrap();
    script.calls.push(place);
    match place {
        // Zero set = nothing waits.
        1 => {
            script.asked += 1;
            r.set_zero(script.asked > script.waits);
        }
        2 => {
            script.sleeps += 1;
            if script.sleeps == script.quit_on {
                unsafe { (script.quit as *mut u32).write_volatile(1) };
            }
        }
        _ => {}
    }
}

/// The Rust version's parts, through the same handler.
struct Tested(*const u32);

impl Tested {
    fn at(place: u32) -> Regs {
        let mut r = Regs::default();
        act(place, &mut r);
        r
    }
}

impl Parts for Tested {
    fn end_turn(&mut self) {
        Self::at(0);
    }

    fn pending(&mut self) -> bool {
        Self::at(1).eflags & 0x40 == 0
    }

    fn sleep_a_millisecond(&mut self) {
        Self::at(2);
    }

    fn pump_messages(&mut self) {
        Self::at(3);
    }

    fn quitting(&mut self) -> bool {
        unsafe { self.0.read_volatile() == 1 }
    }
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let copy = unsafe {
        RedirectedCopy::new(
            game_addr(rva(0x409C1E)),
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
    let mut quit = 0u32;
    unsafe {
        let code = std::slice::from_raw_parts(copy.addr() as *const u8, LEN);
        let named = u32::from_le_bytes(code[QUIT_OPERAND..QUIT_OPERAND + 4].try_into().unwrap());
        if named != game_addr(QUIT_RVA) as u32 {
            return t.skip("the original is not what the test was made from");
        }
        copy.patch_u32(QUIT_OPERAND, &raw mut quit as u32);
    }
    // (what, how often commands still wait, the quit switch to begin with, set on)
    for (what, waits, quit_at_first, quit_on) in [
        ("nothing waits", 0u32, 0u32, 0u32),
        ("three rounds", 3, 0, 0),
        ("quitting from the first", 5, 1, 0),
        ("quit during the second sleep", 9, 0, 2),
        ("a quit switch of 2", 2, 2, 0),
    ] {
        let before = Regs::SENTINEL;
        let mut run = |rust: bool| unsafe {
            std::ptr::write_volatile(&raw mut quit, quit_at_first);
            *SCRIPT.lock().unwrap() = Script {
                waits,
                quit_on,
                quit: &raw mut quit as usize,
                asked: 0,
                sleeps: 0,
                calls: Vec::new(),
            };
            set_handler(handler);
            let preserved = if rust {
                wait_for_commands(&mut Tested(&raw const quit));
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
            format!("{what}: original {theirs:?}; rust {mine:?}")
        });
    }
    t.finish()
}
