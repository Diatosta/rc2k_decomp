//! The game's small functions that send the audio thread one command and wait for it:
//! what they share, for their hooks and self-tests.
//!
//! Each is `pusha`, some registers set, a sender of one command (0xB, the music, or 9),
//! `sound::wait_for_commands`, `popa`; the tunes go through `sound::play_music` for it.
//! A file for one of them holds what differs: the registers the command is sent with.

use crate::framework::{
    RawHook, Regs, SelfTest, call_game, game_addr, game_read, game_write, rva, shadow,
};
use crate::install::stand_ins::{act, set_handler, stand_in};
use crate::redirect::RedirectedCopy;
use crate::screens::harness::CallOriginal;
use crate::service::harness::{at, capture, restore};
use crate::sound::queue::{Queue, send_command, sent};
use crate::sound::wait_for_commands::{Game, wait_for_commands};
use std::sync::Mutex;

/// One such function.
pub struct Sender {
    /// Its address, length and first bytes, as IDA shows them.
    pub address: usize,
    pub len: usize,
    pub entry: [u8; 8],
    /// The offsets of its calls.
    pub calls: &'static [usize],
    /// The command it sends.
    pub command: u32,
    /// From the registers at entry: the registers the command is sent (and waited) with,
    /// or None if it sends nothing.
    pub sends: fn(&Regs) -> Option<Regs>,
    /// The game's variables it reads, each with values for the self-test to try.
    pub reads: &'static [(usize, &'static [u32])],
}

/// The senders of commands 0xB and 9, `sound::wait_for_commands`, and the two functions
/// some go through.
const SEND_B: usize = 0x409BF6;
const SEND_9: usize = 0x409BEB;
const WAIT: usize = 0x409C1E;
const PLAY_MUSIC: usize = 0x40F80E;
const STOP_MUSIC: usize = 0x40F826;

/// The `dispatch` of one of them: the original alone in shadow mode (a command must not
/// be sent twice).
pub fn dispatch_sender(hook: &RawHook, r: &mut Regs, call_original: CallOriginal, sender: &Sender) {
    hook.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    if let Some(with) = (sender.sends)(r) {
        unsafe { send_command(&Queue::game(), sender.command, sent(&with)) };
        wait_for_commands(&mut Game(sent(&with)));
    }
}

/// Each call of a run: its place (0 command 0xB, 1 the wait, 2 command 9) and the seven
/// registers a command holds.
static CALLED: Mutex<Vec<(u32, [u32; 7])>> = Mutex::new(Vec::new());

fn handler(place: u32, r: &mut Regs) {
    CALLED.lock().unwrap().push((place, sent(r)));
}

/// A copy of a function whose senders and wait go to the stand-ins, and whose calls of
/// `play_music` and `stop_music` go to such copies of those.
unsafe fn copy_of(
    address: usize,
    len: usize,
    entry: &[u8],
    calls: &[usize],
    inner: &[(usize, usize)],
) -> Result<RedirectedCopy, &'static str> {
    unsafe {
        RedirectedCopy::new(game_addr(rva(address)), len, entry, calls, &|callee| {
            if callee == rva(SEND_B) {
                Some(stand_in(0))
            } else if callee == rva(WAIT) {
                Some(stand_in(1))
            } else if callee == rva(SEND_9) {
                Some(stand_in(2))
            } else {
                inner
                    .iter()
                    .find(|&&(known, _)| rva(known) == callee)
                    .map(|&(_, copy)| copy)
            }
        })
    }
}

/// The self-test of one of them: its copy against `sends`, with several sets of
/// registers and every combination of its variables' values.
pub fn check_sender(hook: &RawHook, sender: &Sender) -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(hook) else {
        return (0, 0);
    };
    let copies = unsafe {
        let play = copy_of(
            PLAY_MUSIC,
            0x18,
            &[0x60, 0x0F, 0xB6, 0xDC, 0x0F, 0xB6, 0xC0, 0xBA],
            &[0x0C, 0x11],
            &[],
        );
        let stop = copy_of(
            STOP_MUSIC,
            0x18,
            &[0x60, 0x33, 0xC0, 0x33, 0xDB, 0x33, 0xC9, 0xBA],
            &[0x0C, 0x11],
            &[],
        );
        play.and_then(|play| stop.map(|stop| (play, stop)))
    };
    let (play, stop) = match copies {
        Ok(copies) => copies,
        Err(why) => return t.skip(why),
    };
    let copy = unsafe {
        copy_of(
            sender.address,
            sender.len,
            &sender.entry,
            sender.calls,
            &[(PLAY_MUSIC, play.addr()), (STOP_MUSIC, stop.addr())],
        )
    };
    let copy = match copy {
        Ok(copy) => copy,
        Err(why) => return t.skip(why),
    };
    let inputs: Vec<_> = sender.reads.iter().map(|&(var, _)| at(rva(var))).collect();
    let saved = capture(&inputs);
    // Every combination of the variables' values.
    let mut combinations: Vec<Vec<u32>> = vec![Vec::new()];
    for &(_, values) in sender.reads {
        combinations = combinations
            .into_iter()
            .flat_map(|sofar| {
                values.iter().map(move |&value| {
                    let mut next = sofar.clone();
                    next.push(value);
                    next
                })
            })
            .collect();
    }
    for values in &combinations {
        for eax in [0u32, 0x1234_5678, 0xFFFF_FFFF, 0x40, 0x0000_0A08] {
            let before = Regs {
                eax,
                ebx: 0x2222_0002,
                ecx: 0x3333_0003,
                edx: 0x4444_0004,
                ..Regs::SENTINEL
            };
            let run = |rust: bool| unsafe {
                for (&(var, _), &value) in sender.reads.iter().zip(values) {
                    game_write(rva(var), value);
                }
                CALLED.lock().unwrap().clear();
                set_handler(handler);
                let preserved = if rust {
                    if let Some(with) = (sender.sends)(&before) {
                        let place = if sender.command == 9 { 2 } else { 0 };
                        act(place, &mut { with });
                        act(1, &mut { with });
                    }
                    true
                } else {
                    let mut ro = before;
                    call_game(&mut ro, copy.addr());
                    ro.preserved_except(&before, 0)
                };
                (preserved, CALLED.lock().unwrap().clone())
            };
            let (theirs, mine) = (run(false), run(true));
            t.check(theirs == mine && theirs.0, || {
                format!("eax {eax:#x}, variables {values:x?}: original {theirs:x?}; rust {mine:x?}")
            });
        }
    }
    restore(&inputs, &saved);
    t.finish()
}

/// For the `sends` functions: a game variable's value.
pub fn variable(at: usize) -> u32 {
    unsafe { game_read(rva(at)) }
}

/// What `sound::play_music` sends for a tune, a second tune and a volume, from the
/// registers its caller has.
pub fn music(entry: &Regs, tune: u32, second: u32, volume: u32) -> Regs {
    Regs {
        eax: tune,
        ebx: second,
        ecx: volume,
        edx: 2,
        ..*entry
    }
}
